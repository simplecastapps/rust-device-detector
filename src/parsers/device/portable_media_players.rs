use anyhow::Result;

use super::{Device, DeviceList};
use once_cell::sync::Lazy;

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = std::include_str!("../../../regexes/device/portable_media_player.yml");
    DeviceList::from_file(contents).expect("loading portable_media_player.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "portable media player")
}
