use std::{collections::HashMap, sync::Arc};

use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

pub type BeaconDiffMap = HashMap<(String, String), BeaconDiff>;
pub type BeaconDiffMapArc = Arc<Mutex<BeaconDiffMap>>;

#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct BeaconList {
    pub device_identifier: String,
    pub beacons: Vec<Beacon>,
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Debug)]
pub struct Beacon {
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    pub rssi: i32
}

#[derive(PartialEq, Clone, Debug)]
pub struct BeaconDiff {
    pub mac_address: String,
    pub rssi: i32,
    pub count: i32,
    pub diff: i32,
}
