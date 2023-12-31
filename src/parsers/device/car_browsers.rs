use anyhow::Result;

use super::{Device, DeviceList};
use once_cell::sync::Lazy;

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/device/car_browsers.yml"
    ));
    DeviceList::from_file(contents).expect("loading car_browsers.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "car browser")
}
