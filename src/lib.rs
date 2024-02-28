pub mod data_types;

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
}