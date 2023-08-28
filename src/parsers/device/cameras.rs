use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/cameras.yml");
        DeviceList::from_file(contents).expect("loading cameras.yml")
    };
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "camera")
}
