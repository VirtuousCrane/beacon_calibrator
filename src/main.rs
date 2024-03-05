use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::join_all;
use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use tokio::sync::Mutex;
use beacon_calibrator::data_types::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mqttoptions = MqttOptions::new("beacon_calibrator_sdfgjiko", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("LOLICON/BEACON", QoS::AtMostOnce)
        .await
        .unwrap();
    let client_handle: Arc<Mutex<AsyncClient>> = Arc::new(Mutex::new(client));
    
    let beacon_map: Arc<Mutex<HashMap<String, BeaconDiff>>> = Arc::new(Mutex::new(HashMap::new()));
    
    while let Ok(notification) = eventloop.poll().await {
        let packet = match notification {
            rumqttc::Event::Incoming(pck) => pck,
            rumqttc::Event::Outgoing(_) => continue,
        };

        if let Packet::Publish(msg) = packet {
            let payload = msg.payload;
            if !payload.is_ascii() {
                continue;
            }
            
            let data = String::from_utf8_lossy(&payload);
            let data_struct: BeaconList = match serde_json::from_str(&data) {
                Ok(b) => b,
                Err(e) => {
                    println!("Invalid Format: {}", e.to_string());
                    continue;
                } 
            };
            
            let map_arc = Arc::clone(&beacon_map);
            let client_handle_arc = Arc::clone(&client_handle);

            tokio::spawn(async move {
                let mut beacon_processor_handle = Vec::new();

                for beacon_data in data_struct.beacons.iter() {
                    let data_diff = get_diff_from_map_arc(map_arc.clone(), &beacon_data.mac_address).await;
                    let process_handle = process_beacon_data(beacon_data, data_diff, map_arc.clone(), client_handle_arc.clone());
                    beacon_processor_handle.push(process_handle);
                }
                let _result = join_all(beacon_processor_handle).await;
            });
        };
    };

    Ok(())
}

async fn process_beacon_data(beacon_data: &Beacon, old_diff: Option<BeaconDiff>, map_arc: Arc<Mutex<HashMap<String, BeaconDiff>>>, mqtt_client_arc: Arc<Mutex<AsyncClient>>) {
    let beacon_diff = get_beacon_diff(beacon_data, old_diff, map_arc).await;
    
    if beacon_diff.count < 5 {
        return;
    } 
    
    let handle = mqtt_client_arc.lock().await;
    let text = format!("{{ \"macAddress\": \"{}\", \"rssi\": {}, \"diff\": {} }}", beacon_diff.mac_address, beacon_diff.rssi, beacon_diff.diff);
    let _publish_handle = handle.publish("LOLICON/BEACON/CALIBRATION", QoS::AtLeastOnce, false, text).await;
}

async fn get_beacon_diff(beacon_data: &Beacon, old_diff: Option<BeaconDiff>, map_arc: Arc<Mutex<HashMap<String, BeaconDiff>>>) -> BeaconDiff {
    let new_diff = match old_diff {
        Some(s) => {
            let rssi_product = s.rssi * s.count;
            let new_count = s.count + 1;
            let avg_rssi = (rssi_product + beacon_data.rssi) / new_count;
            let diff = s.rssi.abs_diff(beacon_data.rssi);
            let diff: i32 = if avg_rssi.abs() < beacon_data.rssi.abs() { diff as i32 } else { -1 * diff as i32 };
            let new_struct = BeaconDiff { mac_address: beacon_data.mac_address.clone(), rssi: avg_rssi, count: new_count, diff };
            
            new_struct
        },
        None => {
            BeaconDiff { mac_address: beacon_data.mac_address.clone(), rssi: beacon_data.rssi, count: 1, diff: 0 }
        }
    };
    
    let mut map_guard = map_arc.lock().await;
    map_guard.insert(beacon_data.mac_address.clone(), new_diff.clone());
    
    return new_diff;
}

async fn get_diff_from_map_arc(map_arc: Arc<Mutex<HashMap<String, BeaconDiff>>>, key: &String) -> Option<BeaconDiff> {
    let map_lock = map_arc.lock().await;
    match map_lock.get(key) {
        Some(obj) => Some(obj.clone()),
        None => None
    }
}