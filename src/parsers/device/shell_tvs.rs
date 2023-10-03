use anyhow::Result;

use once_cell::sync::Lazy;

use super::{Device, DeviceList};

use super::DeviceType;
use crate::parsers::utils::{static_user_agent_match, SafeRegex as Regex};

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/regexes/device/shell_tv.yml"));
    DeviceList::from_file(contents).expect("loading shell_tv.yml")
});

static SHELL_TV: Lazy<Regex> =
    static_user_agent_match!(r#"[a-z]+[ _]Shell[ _]\w{6}|tclwebkit(\d+[\.\d]*)"#);

pub fn is_shell_tv(ua: &str) -> Result<bool> {
    let res = SHELL_TV.is_match(ua)?;
    Ok(res)
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
