#![allow(clippy::missing_safety_doc)]

use libc::c_char;
use std::ffi::{CStr, CString};
use std::ptr::{null, null_mut};

use crate::device_detector::{Detection, DeviceDetector};
use crate::parsers::bot::Bot;
use crate::parsers::client::Client;
use crate::parsers::device::Device;
use crate::parsers::oss::OS;

pub struct RDDDeviceDetector {
    dd: DeviceDetector,
}

impl RDDDeviceDetector {
    #[allow(unused)]
    pub fn new(cache_size: u64) -> RDDDeviceDetector {
        RDDDeviceDetector {
            #[cfg(feature = "cache")]
            dd: DeviceDetector::new_with_cache(cache_size),
            #[cfg(not(feature = "cache"))]
            dd: DeviceDetector::new(),
        }
    }
}

pub struct RDDDetection {
    dt: Option<Detection>,
}

#[derive(Debug)]
#[repr(C)]
pub struct RDDClient<'a> {
    client: &'a Option<Client>,
}

#[derive(Debug)]
#[repr(C)]
pub struct RDDDevice<'a> {
    device: &'a Option<Device>,
}

#[derive(Debug)]
#[repr(C)]
pub struct RDDOS<'a> {
    os: &'a Option<OS>,
}

#[derive(Debug)]
#[repr(C)]
pub struct RDDBot<'a> {
    bot: Option<&'a Bot>,
}

#[no_mangle]
pub extern "C" fn rdd_device_detector_new(cache_size: u64) -> *mut RDDDeviceDetector {
    Box::into_raw(Box::new(RDDDeviceDetector::new(cache_size)))
}

#[no_mangle]
pub unsafe extern "C" fn rdd_lookup(
    rdd: *const RDDDeviceDetector,
    ua: *const c_char,
) -> *mut RDDDetection {
    let rdd = unsafe { &*rdd };
    let ua = unsafe { CStr::from_ptr(ua) };

    let rdd = match ua.to_str() {
        Err(_invalid_utf8) => {
            println!("invalid utf8");
            RDDDetection { dt: None }
        }
        Ok(ua) => {
            match rdd.dd.parse(ua, None) {
                Ok(dd) => {
                    // println!("dd: {:#?}", dd);
                    RDDDetection { dt: Some(dd) }
                }
                Err(err) => {
                    println!("error: {:?}", err);
                    RDDDetection { dt: None }
                }
            }
        }
    };

    Box::into_raw(Box::new(rdd))
}

#[no_mangle]
pub unsafe extern "C" fn rdd_client<'a>(rdd: *const RDDDetection) -> *const RDDClient<'a> {
    let rdd = unsafe { &*rdd };
    match &rdd.dt {
        Some(Detection::Known(dev)) => {
            let rdd = RDDClient {
                client: &dev.client,
            };
            Box::into_raw(Box::new(rdd))
        }
        _ => null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_client_name(client: *const RDDClient) -> *mut c_char {
    let client = unsafe { &*client };

    client
        .client
        .as_ref()
        .map(|client| client.name.as_str())
        .map(|name| CString::new(name).unwrap().into_raw())
        .unwrap_or(null_mut())
}

// TODO this could just be a number, which could be converted into a static str.
#[no_mangle]
pub unsafe extern "C" fn rdd_client_type(client: *const RDDClient) -> *mut c_char {
    let client = unsafe { &*client };

    client
        .client
        .as_ref()
        .map(|client| client.r#type.as_str())
        .map(|r#type| CString::new(r#type).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_client_version(client: *const RDDClient) -> *mut c_char {
    let client = unsafe { &*client };

    client
        .client
        .as_ref()
        .and_then(|client| client.version.as_deref())
        .map(|version| CString::new(version).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_client_browser_engine(client: *const RDDClient) -> *mut c_char {
    let client = unsafe { &*client };

    client
        .client
        .as_ref()
        .and_then(|client| client.engine.as_deref())
        .map(|engine| CString::new(engine).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_client_browser_version(client: *const RDDClient) -> *mut c_char {
    let client = unsafe { &*client };

    client
        .client
        .as_ref()
        .and_then(|client| client.engine_version.as_deref())
        .map(|engine_version| CString::new(engine_version).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_device<'a>(rdd: *const RDDDetection) -> *const RDDDevice<'a> {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => {
            let rdd = RDDDevice {
                device: &dev.device,
            };
            Box::into_raw(Box::new(rdd))
        }
        _ => null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_device_brand(device: *const RDDDevice) -> *mut c_char {
    let device = unsafe { &*device };

    device
        .device
        .as_ref()
        .and_then(|d| d.brand.as_ref())
        .map(|b| CString::new(b.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_device_model(device: *const RDDDevice) -> *mut c_char {
    let device = unsafe { &*device };

    device
        .device
        .as_ref()
        .and_then(|d| d.model.as_ref())
        .map(|m| CString::new(m.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_device_type(device: *const RDDDevice) -> *mut c_char {
    let device = unsafe { &*device };

    device
        .device
        .as_ref()
        .and_then(|d| d.device_type.as_ref())
        .map(|t| CString::new(t.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn rdd_os<'a>(rdd: *const RDDDetection) -> *const RDDOS<'a> {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => {
            let rdd = RDDOS { os: &dev.os };
            Box::into_raw(Box::new(rdd))
        }
        _ => null(),
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_os_name(os: *const RDDOS) -> *mut c_char {
    let os = unsafe { &*os };

    os.os
        .as_ref()
        .map(|os| CString::new(&*os.name).unwrap().into_raw())
        .unwrap_or(null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn rdd_os_version(os: *const RDDOS) -> *mut c_char {
    let os = unsafe { &*os };

    os.os
        .as_ref()
        .and_then(|os| os.version.as_ref())
        .map(|version| CString::new(version.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn rdd_os_platform(os: *const RDDOS) -> *mut c_char {
    let os = unsafe { &*os };

    os.os
        .as_ref()
        .and_then(|os| os.platform.as_ref())
        .map(|platform| CString::new(platform.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_os_family(os: *const RDDOS) -> *mut c_char {
    let os = unsafe { &*os };

    os.os
        .as_ref()
        .and_then(|os| os.family.as_ref())
        .map(|family| CString::new(family.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

// pub struct Bot {
//     pub name: String,
//     pub category: Option<String>,
//     pub url: Option<String>,
//     pub producer: Option<BotProducer>,
// }

// pub struct BotProducer {
//     pub name: Option<String>,
//     pub url: Option<String>,
// }

#[no_mangle]
pub unsafe extern "C" fn rdd_bot<'a>(rdd: *const RDDDetection) -> *const RDDBot<'a> {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Bot(bot)) => {
            let rdd = RDDBot { bot: Some(bot) };
            Box::into_raw(Box::new(rdd))
        }
        _ => null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_bot_name(bot: *const RDDBot) -> *mut c_char {
    let bot = unsafe { &*bot };

    bot.bot
        .as_ref()
        .map(|bot| CString::new(&*bot.name).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_bot_category(bot: *const RDDBot) -> *mut c_char {
    let bot = unsafe { &*bot };

    bot.bot
        .as_ref()
        .and_then(|bot| bot.category.as_ref())
        .map(|category| CString::new(category.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_bot_url(bot: *const RDDBot) -> *mut c_char {
    let bot = unsafe { &*bot };

    bot.bot
        .as_ref()
        .and_then(|bot| bot.url.as_ref())
        .map(|url| CString::new(url.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_bot_producer_name(bot: *const RDDBot) -> *mut c_char {
    let bot = unsafe { &*bot };

    bot.bot
        .as_ref()
        .and_then(|bot| bot.producer.as_ref())
        .and_then(|producer| producer.name.as_ref())
        .map(|name| CString::new(name.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_bot_producer_url(bot: *const RDDBot) -> *mut c_char {
    let bot = unsafe { &*bot };

    bot.bot
        .as_ref()
        .and_then(|bot| bot.producer.as_ref())
        .and_then(|producer| producer.url.as_ref())
        .map(|url| CString::new(url.as_str()).unwrap().into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_bot(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    rdd.dt.as_ref().map(|x| x.is_bot()).unwrap_or(false)
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_mobile(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_mobile(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_touch_enabled(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_touch_enabled(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_pim(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_pim(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_feed_reader(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_feed_reader(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_mobile_app(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_mobile_app(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_media_player(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_media_player(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_browser(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_browser(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_library(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_library(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_desktop(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_desktop(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_console(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_console(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_is_car_browser(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_car_browser(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_camera(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_camera(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_portable_media_player(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_portable_media_player(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_notebook(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_notebook(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_television(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_television(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_smart_display(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_smart_display(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_feature_phone(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_feature_phone(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_smart_phone(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_smart_phone(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_tablet(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_tablet(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_phablet(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_phablet(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_smart_speaker(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_smart_speaker(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_peripheral(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_peripheral(),
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_is_wearable(rdd: *const RDDDetection) -> bool {
    let rdd = unsafe { &*rdd };

    match &rdd.dt {
        Some(Detection::Known(dev)) => dev.is_wearable(),
        _ => false,
    }
}
#[no_mangle]
pub unsafe extern "C" fn rdd_free_device_detector(rdd: *mut RDDDeviceDetector) {
    unsafe {
        drop(Box::from_raw(rdd));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_free_detection(rdd: *mut RDDDetection) {
    unsafe {
        drop(Box::from_raw(rdd));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rdd_free_string(rdd: *mut c_char) {
    let cstr = unsafe { CString::from_raw(rdd) };
    drop(cstr);
}
