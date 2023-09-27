use anyhow::Result;

use super::{Device, DeviceList};
use once_cell::sync::Lazy;

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = std::include_str!("../../../regexes/device/cameras.yml");
    DeviceList::from_file(contents).expect("loading cameras.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "camera")
}
