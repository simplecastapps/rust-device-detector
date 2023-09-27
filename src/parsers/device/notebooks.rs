use anyhow::Result;

use super::{Device, DeviceList};
use once_cell::sync::Lazy;

use crate::parsers::utils::{static_user_agent_match, SafeRegex as Regex};

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = std::include_str!("../../../regexes/device/notebooks.yml");
    DeviceList::from_file(contents).expect("loading notebooks.yml")
});

static NOTEBOOK: Lazy<Regex> = static_user_agent_match!(r#"FBMD/"#);

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    if !NOTEBOOK.is_match(ua)? {
        return Ok(None);
    }

    DEVICE_LIST.lookup(ua, "notebook")
}
