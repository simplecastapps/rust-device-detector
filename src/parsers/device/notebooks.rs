use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

use crate::parsers::utils::user_agent_match;
use crate::parsers::utils::LazyRegex;

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/notebooks.yml");
        DeviceList::from_file(contents).expect("loading notebooks.yml")
    };
    static ref NOTEBOOK: LazyRegex = user_agent_match(r#"FBMD/"#);
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    if !NOTEBOOK.is_match(ua)? {
        return Ok(None);
    }

    DEVICE_LIST.lookup(ua, "notebook")
}
