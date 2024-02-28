use std::{collections::HashMap, sync::Arc, time::Duration};

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
    
    let beacon_map: Arc<Mutex<HashMap<String, BeaconCount>>> = Arc::new(Mutex::new(HashMap::new()));
    
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
                for beacon_data in data_struct.beacons {
                    let mut map = map_arc.lock().await;
                    let data_count = map.get(&beacon_data.mac_address);

                    let result = match data_count {
                        Some(s) => {
                            let rssi_product = s.rssi * s.count;
                            let new_count = s.count + 1;
                            let avg_rssi = (rssi_product + beacon_data.rssi) / new_count;
                            let diff = s.rssi.abs_diff(beacon_data.rssi);
                            let diff: i32 = if avg_rssi.abs() < beacon_data.rssi.abs() { diff as i32 } else { -1 * diff as i32 };
                            let new_struct = BeaconCount { rssi: avg_rssi, count: new_count, diff };
                            
                            map.insert(
                                beacon_data.mac_address.clone(),
                                new_struct.clone()
                            );
                            new_struct
                        },
                        None => {
                            let new_struct = BeaconCount { rssi: beacon_data.rssi, count: 1, diff: 0 };
                            map.insert(
                                beacon_data.mac_address.clone(),
                                new_struct.clone()
                            );
                            new_struct
                        }
                    };
                    
                    if result.count < 5 {
                        return;
                    } 
                    
                    let handle = client_handle_arc.lock().await;
                    let text = format!("{{ \"macAddress\": \"{}\", \"rssi\": {}, \"diff\": {} }}", beacon_data.mac_address, result.rssi, result.diff);
                    let _publish_result = handle.publish("LOLICON/BEACON/CALIBRATION", QoS::AtLeastOnce, false, text).await;
                }
            });
        };
    };

    Ok(())
}
