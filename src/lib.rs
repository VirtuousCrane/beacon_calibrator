pub mod data_types;
pub mod program_logic;

#[cfg(test)]
mod tests {
    use crate::data_types::*;

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
}