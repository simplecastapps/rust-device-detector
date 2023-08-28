use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

use super::DeviceType;
use crate::parsers::utils::user_agent_match;
use crate::parsers::utils::LazyRegex;

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/televisions.yml");
        DeviceList::from_file(contents).expect("loading televisions.yml")
    };
    static ref HBTV: LazyRegex = user_agent_match(r#"HbbTV/([1-9]{1}(?:\.[0-9]{1}){1,2})"#);
}

pub fn is_hbbtv(ua: &str) -> Result<bool> {
    HBTV.is_match(ua)
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    if !is_hbbtv(ua)? {
        return Ok(None);
    }

    let res = DEVICE_LIST.lookup(ua, "tv")?.map(|mut res| {
        res.device_type = Some(DeviceType::Television);
        res
    });

    // always set device type to tv for hbtvs
    let res = res.or_else(|| {
        Some(Device {
            device_type: Some(DeviceType::Television),
            ..Default::default()
        })
    });

    Ok(res)
}
