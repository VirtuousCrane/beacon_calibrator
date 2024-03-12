pub mod data_types;
pub mod program_logic;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use tokio::sync::Mutex;

    use crate::{data_types::*, program_logic::{get_beacon_diff, get_new_beacon_diff}};

    #[test]
    fn json_deserialize_test() {
        let input_string = "{\"beacons\":[{\"macAddress\":\"7c:87:ce:49:29:2a\",\"rssi\":-63}]}";
        let data_struct: Result<BeaconList, serde_json::Error> = serde_json::from_str(&input_string);

        let expected_struct = BeaconList {
            beacons: vec![Beacon { mac_address: "7c:87:ce:49:29:2a".into(), rssi: -63 }],
        };

        assert_eq!(expected_struct.beacons, data_struct.unwrap().beacons);
    }
    
    #[test]
    fn json_deserialize_multiple_test() {
        let input_string = "{\"beacons\":[{\"macAddress\":\"34:ab:95:73:5a:9a\",\"rssi\":-67},{\"macAddress\":\"7c:87:ce:49:2b:82\",\"rssi\":-42}]}";
        
        let data_struct: Result<BeaconList, serde_json::Error> = serde_json::from_str(&input_string);

        let beacon_1 = Beacon { mac_address: "34:ab:95:73:5a:9a".into(), rssi: -67 };
        let beacon_2 = Beacon { mac_address: "7c:87:ce:49:2b:82".into(), rssi: -42 };
        let expected_struct = BeaconList {
            beacons: vec![beacon_1, beacon_2],
        };

        assert_eq!(expected_struct.beacons, data_struct.unwrap().beacons)
    }

    #[test]
    fn get_new_beacon_diff_test() {
        // Setup
        let beacon = Beacon { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69 };
        let old_diff = Some(BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -40, count: 5, diff: 0 });

        // Execute
        let result = get_new_beacon_diff(&beacon, old_diff);
        
        // Verify
        let expected = BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -44, count: 6, diff: 29 };
        assert_eq!(expected, result);
    }

    #[test]
    fn get_new_beacon_diff_test_old_diff_none() {
        // Setup
        let beacon = Beacon { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69 };
        let old_diff = None;

        // Execute
        let result = get_new_beacon_diff(&beacon, old_diff);
        
        // Verify
        let expected = BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69, count: 1, diff: 0 };
        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn get_beacon_diff_test() {
        // Setup
        let mut map: HashMap<String, BeaconDiff> = HashMap::new();
        map.insert("FF:FF:FF:FF:FF:FF".into(), BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -40, count: 1, diff: 0 });

        let map_arc = Arc::new(Mutex::new(map));
        let beacon = Beacon { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69 };
        let beacon_list = BeaconList { beacons: vec![beacon] };

        // Execute
        let result = get_beacon_diff(map_arc, &beacon_list).await;

        // Verify
        let expected = vec![ BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -54, count: 2, diff: 29} ];
        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn get_beacon_diff_test_empty_map() {
        // Setup
        let map: HashMap<String, BeaconDiff> = HashMap::new();
        let map_arc = Arc::new(Mutex::new(map));
        let beacon = Beacon { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69 };
        let beacon_list = BeaconList { beacons: vec![beacon] };

        // Execute
        let result = get_beacon_diff(map_arc, &beacon_list).await;

        // Verify
        let expected = vec![ BeaconDiff { mac_address: "FF:FF:FF:FF:FF:FF".into(), rssi: -69, count: 1, diff: 0} ];
        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn get_beacon_diff_test_empty_list() {
        // Setup
        let map: HashMap<String, BeaconDiff> = HashMap::new();
        let map_arc = Arc::new(Mutex::new(map));
        let beacon_list = BeaconList { beacons: Vec::new() };

        // Execute
        let result = get_beacon_diff(map_arc, &beacon_list).await;

        // Verify
        let expected: Vec<BeaconDiff> = Vec::new();
        assert_eq!(expected, result);
    }
}