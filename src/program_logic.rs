use std::{collections::HashMap, sync::Arc};

use futures::future::join_all;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::Mutex;

use crate::data_types::{Beacon, BeaconDiff, BeaconList};

/// Takes a Vector of BeaconDiff objects and send it to an MQTT Stream as JSON
pub async fn send_beacon_data(beacon_diff_list: Vec<BeaconDiff>, mqtt_client_arc: Arc<AsyncClient>) {
    let mut mqtt_send_handle = Vec::new();

    for beacon_diff in beacon_diff_list.iter() {
        if beacon_diff.count < 5 {
            continue;
        }

        let text = format!("{{ \"macAddress\": \"{}\", \"rssi\": {}, \"diff\": {} }}", beacon_diff.mac_address, beacon_diff.rssi, beacon_diff.diff);
        let publish_handle = mqtt_client_arc.publish("LOLICON/BEACON/CALIBRATION", QoS::AtLeastOnce, false, text);
        mqtt_send_handle.push(publish_handle);
    }
    
    join_all(mqtt_send_handle).await;
}

/// Creates a new BeaconDiff Object based on a Beacon's current and old data
pub fn get_new_beacon_diff(beacon_data: &Beacon, old_diff: Option<BeaconDiff>) -> BeaconDiff {
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
    
    new_diff
}

/// Gets a set of new BeaconDiff Object for Beacons in the BeaconList Object
pub async fn get_beacon_diff(map_arc: Arc<Mutex<HashMap<String, BeaconDiff>>>, beacon_list: &BeaconList) -> Vec<BeaconDiff> {
    let lock = map_arc.lock();
    let mut map = lock.await;
    let mut new_beacon_diff: Vec<BeaconDiff> = Vec::new();
    
    for beacon in beacon_list.beacons.iter() {
        let old_diff = match map.get(&beacon.mac_address) {
            Some(obj) => Some(obj.clone()),
            None => None
        };
        
        let new_diff = get_new_beacon_diff(beacon, old_diff);
        new_beacon_diff.push(new_diff.clone());
        map.insert(beacon.mac_address.clone(), new_diff);
    }

    return new_beacon_diff;
}