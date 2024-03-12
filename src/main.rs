use std::{collections::HashMap, sync::Arc, time::Duration};

use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use tokio::sync::Mutex;
use beacon_calibrator::{data_types::*, program_logic::{get_beacon_diff, send_beacon_data}};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mqttoptions = MqttOptions::new("beacon_calibrator_sdfgjiko", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("LOLICON/BEACON", QoS::AtMostOnce)
        .await
        .unwrap();
    let client_handle: Arc<AsyncClient> = Arc::new(client);
    
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
                let beacon_diff = get_beacon_diff(map_arc.clone(), &data_struct).await;
                send_beacon_data(beacon_diff, client_handle_arc.clone()).await;
            });
        };
    };

    Ok(())
}
