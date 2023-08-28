use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/portable_media_player.yml");
        DeviceList::from_file(contents).expect("loading portable_media_player.yml")
    };
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "portable media player")
}
