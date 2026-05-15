use anyhow::Result;

use once_cell::sync::Lazy;

use super::{Device, DeviceList};

use super::DeviceType;
use crate::parsers::utils::{static_user_agent_match, SafeRegex as Regex};

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/device/televisions.yml"
    ));
    DeviceList::from_file(contents).expect("loading televisions.yml")
});
// Matches PHP HbbTv.php isHbbTv(): checks for HbbTV/ OR SmartTvA/ (case-insensitive via SafeRegex)
static HBTV: Lazy<Regex> =
    static_user_agent_match!(r#"(?:HbbTV|SmartTvA)/([1-9]{1}(?:\.[0-9]{1}){1,2})"#);

pub fn is_hbbtv(ua: &str) -> Result<bool> {
    let res = HBTV.is_match(ua)?;
    Ok(res)
}

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    // Only parse UAs containing HbbTV or SmartTvA fragments (matches PHP HbbTv.php behavior)
    // CE-HTML alone does NOT trigger this parser — those UAs fall through to mobiles.yml
    if !is_hbbtv(ua)? {
        return Ok(None);
    }

    let res = DEVICE_LIST.lookup(ua, "tv")?.map(|mut res| {
        // Only set device type to Television if not already set (e.g., could be Peripheral)
        if res.device_type.is_none() {
            res.device_type = Some(DeviceType::Television);
        }
        res
    });

    // For HbbTV/SmartTvA, always return a TV device even if no brand was found in the list.
    if res.is_none() {
        return Ok(Some(Device {
            device_type: Some(DeviceType::Television),
            ..Default::default()
        }));
    }

    Ok(res)
}
