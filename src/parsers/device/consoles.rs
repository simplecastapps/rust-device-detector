use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/consoles.yml");
        DeviceList::from_file(contents).expect("loading consoles.yml")
    };
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "console")
}
