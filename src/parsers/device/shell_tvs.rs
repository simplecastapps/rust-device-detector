use anyhow::Result;

use lazy_static::lazy_static;

use super::{Device, DeviceList};

use super::DeviceType;
use crate::parsers::utils::user_agent_match;
use crate::parsers::utils::LazyRegex;

lazy_static! {
    static ref DEVICE_LIST: DeviceList = {
        let contents = std::include_str!("../../../regexes/device/shell_tv.yml");
        DeviceList::from_file(contents).expect("loading shell_tv.yml")
    };
    static ref SHELL_TV: LazyRegex =
        user_agent_match(r#"[a-z]+[ _]Shell[ _]\w{6}|tclwebkit(\d+[\.\d]*)"#);
}

pub fn is_shell_tv(ua: &str) -> Result<bool> {
    SHELL_TV.is_match(ua)
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    if !is_shell_tv(ua)? {
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
