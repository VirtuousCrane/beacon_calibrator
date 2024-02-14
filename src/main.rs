use std::{collections::HashMap, sync::Arc, time::Duration};

use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Beacon {
    #[serde(rename = "macAddress")]
    mac_address: String,
    rssi: i32
}

#[derive(Clone)]
struct BeaconCount {
    rssi: i32,
    count: i32,
    diff: i32,
}

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
            let data_struct: Beacon = match serde_json::from_str(&data) {
                Ok(b) => b,
                Err(e) => {
                    println!("Invalid Format: {}", e.to_string());
                    continue;
                } 
            };
            
            let map_arc = Arc::clone(&beacon_map);
            let client_handle_arc = Arc::clone(&client_handle);

            tokio::spawn(async move {
                let mut map = map_arc.lock().await;
                let data_count = map.get(&data_struct.mac_address);

                let result = match data_count {
                    Some(s) => {
                        let rssi_product = s.rssi * s.count;
                        let new_count = s.count + 1;
                        let avg_rssi = (rssi_product + data_struct.rssi) / new_count;
                        let diff = s.rssi.abs_diff(data_struct.rssi);
                        let diff: i32 = if avg_rssi.abs() < data_struct.rssi.abs() { diff as i32 } else { -1 * diff as i32 };
                        let new_struct = BeaconCount { rssi: avg_rssi, count: new_count, diff };
                        
                        map.insert(
                            data_struct.mac_address.clone(),
                            new_struct.clone()
                        );
                        new_struct
                    },
                    None => {
                        let new_struct = BeaconCount { rssi: data_struct.rssi, count: 1, diff: 0 };
                        map.insert(
                            data_struct.mac_address.clone(),
                            new_struct.clone()
                        );
                        new_struct
                    }
                };
                
                if result.count < 5 {
                    return;
                } 
                
                let handle = client_handle_arc.lock().await;
                let text = format!("{{ \"macAddress\": \"{}\", \"rssi\": {}, \"diff\": {} }}", data_struct.mac_address, result.rssi, result.diff);
                let _publish_result = handle.publish("LOLICON/BEACON/CALIBRATION", QoS::AtLeastOnce, false, text).await;
            });
        };
    };

    Ok(())
}
