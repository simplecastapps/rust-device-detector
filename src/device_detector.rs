use anyhow::Result;

use serde::Serialize;

use crate::client_hints::ClientHint;
use crate::parsers::client::ClientType;
use crate::parsers::device::DeviceType;
use crate::parsers::{bot, client, device, oss};

#[cfg(feature = "cache")]
use moka::sync::Cache;

pub use bot::Bot;

// TODO we should Box KnownDevice as it is much larger than Bot
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Detection {
    Known(KnownDevice),
    Bot(Bot),
}

#[derive(Clone, Debug, Serialize)]
pub struct KnownDevice {
    pub client: Option<client::Client>,
    pub device: Option<device::Device>,
    pub os: Option<oss::OS>,
}

impl Detection {
    pub fn get_bot(&self) -> Option<&Bot> {
        match self {
            Self::Bot(bot) => Some(bot),
            _ => None,
        }
    }

    pub fn get_known_device(&self) -> Option<&KnownDevice> {
        match self {
            Self::Known(known) => Some(known),
            _ => None,
        }
    }

    /// Did we detect a bot? If not, then it is a known device.
    pub fn is_bot(&self) -> bool {
        matches!(self, Self::Bot(_))
    }
    /// This is purely to aid in generating test cases, you should not rely on this for
    /// actual production usage. Only useful for normal stuff, not bots, etc.
    pub fn to_test_case(self, ua: &str) -> String {
        let browser_family: String = match &self {
            Self::Known(known) => known
                .client
                .as_ref()
                .and_then(|x| x.browser.as_ref())
                .and_then(|x| x.family.as_deref())
                .unwrap_or("Unknown")
                .to_owned(),
            _ => "Unknown".to_owned(),
        };

        // if let Self::Known(known) = &self {
        //     dbg!(&known.device);
        // }

        let val = self.to_value();

        // php renders an empty os as an array, which in php is indistinguishable from an empty
        // map, so we have to do the same here
        let os_name = val
            .get("os")
            .and_then(|x| x.get("name"))
            .and_then(|x| x.as_str())
            .unwrap_or("\"\"");
        let os_version = val
            .get("os")
            .and_then(|x| x.get("version"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let os_platform = val
            .get("os")
            .and_then(|x| x.get("platform"))
            .and_then(|x| x.as_str())
            .unwrap_or("\"\"");

        let os = if os_name == "\"\"" && os_version.is_empty() && os_platform == "\"\"" {
            "os: []".to_owned()
        } else {
            format!(
                r#"os:
    name: {}
    version: "{}"
    platform: {}"#,
                os_name, os_version, os_platform
            )
        };

        let client_type = val
            .get("client")
            .and_then(|x| x.get("type"))
            .and_then(|x| x.as_str())
            .unwrap_or("\"\"");

        let client_name = val
            .get("client")
            .and_then(|x| x.get("name"))
            .and_then(|x| x.as_str())
            .unwrap_or("\"\"");

        let client_version = val
            .get("client")
            .and_then(|x| x.get("version"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let client_engine = val
            .get("client")
            .and_then(|x| x.get("engine"))
            .and_then(|x| x.as_str())
            .unwrap_or("\"\"");
        let client_engine_version = val
            .get("client")
            .and_then(|x| x.get("engine_version"))
            .and_then(|x| x.as_str())
            .unwrap_or("");

        let client = if client_type == "\"\""
            && client_name == "\"\""
            && client_version.is_empty()
            && client_engine == "\"\""
            && client_engine_version.is_empty()
        {
            "client: null".to_owned()
        } else {
            format!(
                r#"client:
    type: {}
    name: {}
    version: "{}"
    engine: {}
    engine_version: "{}""#,
                client_type, client_name, client_version, client_engine, client_engine_version
            )
        };

        format!(
            r#"-
  user_agent: '{}'
  {}
  {}
  device:
    type: {}
    brand: {}
    model: '{}'
  os_family: {}
  browser_family: {}
"#,
            ua.trim(),
            os,
            client,
            val.get("device")
                .and_then(|x| x.get("type"))
                .and_then(|x| x.as_str())
                .map(|x| {
                    if x == "television" {
                        "tv"
                    } else {
                        x
                    }
                })
                .unwrap_or("\"\""),
            val.get("device")
                .and_then(|x| x.get("brand"))
                .and_then(|x| x.as_str())
                .unwrap_or("\"\""),
            val.get("device")
                .and_then(|x| x.get("model"))
                .and_then(|x| x.as_str())
                .and_then(|x| {
                    if x.is_empty() {
                        None
                    } else {
                        Some(x)
                    }
                })
                .unwrap_or(""),
            val.get("os")
                .and_then(|x| x.get("family"))
                .and_then(|x| x.as_str())
                .unwrap_or("Unknown"),
            browser_family
        )
    }

    pub fn to_value(self) -> serde_json::Value {
        match self {
            Detection::Known(known) => {
                let is = serde_json::json!({
                    "desktop": known.is_desktop(),
                    "mobile": known.is_mobile(),
                    "touch_enabled": known.is_touch_enabled(),
                    "smart_phone": known.is_smart_phone(),
                    "feature_phone": known.is_feature_phone(),
                    "browser": known.is_browser(),
                    "camera": known.is_camera(),
                    // TODO rename from car
                    "car_browser": known.is_car_browser(),
                    "feed_reader": known.is_feed_reader(),
                    "console": known.is_console(),
                    "library": known.is_library(),
                    "media_player": known.is_media_player(),
                    // TODO rename from portable_mp3
                    "portable_media_player": known.is_portable_media_player(),
                    "mobile_app": known.is_mobile_app(),
                    "television": known.is_television(),
                    "smart_display": known.is_smart_display(),
                    "tablet": known.is_tablet(),
                    "smart_speaker": known.is_smart_speaker(),
                    "pim": known.is_pim(),
                    "peripheral": known.is_peripheral(),
                    "wearable": known.is_wearable(),
                    "phablet": known.is_phablet(),
                    "robot": false,

                });

                let mut val = serde_json::to_value(known).unwrap();
                val["is"] = is;
                val
            }
            Detection::Bot(bot) => {
                serde_json::json!({
                    "bot": serde_json::to_value(bot).unwrap()
                })
            }
        }
    }
}

impl KnownDevice {
    pub fn is_mobile(&self) -> bool {
        if let Some(device) = &self.device {
            if device.mobile_client_hint {
                return true;
            }

            if let Some(device_type) = &device.device_type {
                if [
                    DeviceType::FeaturePhone,
                    DeviceType::SmartPhone,
                    DeviceType::Tablet,
                    DeviceType::Phablet,
                    DeviceType::Camera,
                    DeviceType::PortableMediaPlayer,
                ]
                .contains(device_type)
                {
                    return true;
                }

                if [
                    DeviceType::Television,
                    DeviceType::SmartDisplay,
                    DeviceType::Console,
                ]
                .contains(device_type)
                {
                    return false;
                }
            }
        }

        if let Some(client) = &self.client {
            if device::uses_mobile_browser(client) {
                return true;
            }
        }

        if self.os.is_none() {
            return false;
        }

        !self.is_desktop()
    }

    pub fn is_touch_enabled(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| device.touch_enabled)
            .unwrap_or(false)
    }

    pub fn is_bot(&self) -> bool {
        false
    }

    pub fn is_pim(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::Pim)
            .unwrap_or(false)
    }
    pub fn is_feed_reader(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::FeedReader)
            .unwrap_or(false)
    }

    pub fn is_mobile_app(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::MobileApp)
            .unwrap_or(false)
    }

    pub fn is_media_player(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::MediaPlayer)
            .unwrap_or(false)
    }

    pub fn is_browser(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::Browser)
            .unwrap_or(false)
    }

    pub fn is_library(&self) -> bool {
        self.client
            .as_ref()
            .map(|x| x.r#type == ClientType::Library)
            .unwrap_or(false)
    }

    pub fn is_desktop(&self) -> bool {
        // the php library duplicates logic but as far as I can
        // tell it should be equivalent to this.
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Desktop)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_console(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Console)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_car_browser(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::CarBrowser)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
    pub fn is_camera(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Camera)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
    pub fn is_portable_media_player(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::PortableMediaPlayer)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
    pub fn is_notebook(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Notebook)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_television(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Television)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_smart_display(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::SmartDisplay)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_feature_phone(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::FeaturePhone)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_smart_phone(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::SmartPhone)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_tablet(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Tablet)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_phablet(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Phablet)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_smart_speaker(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::SmartSpeaker)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_peripheral(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Peripheral)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn is_wearable(&self) -> bool {
        self.device
            .as_ref()
            .map(|device| {
                device
                    .device_type
                    .as_ref()
                    .map(|x| *x == DeviceType::Wearable)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

// use std::alloc::System;

#[cfg(feature = "cache")]
type DetectionCache = Cache<String, Detection>;

#[derive(Clone)]
pub struct DeviceDetector {
    #[cfg(feature = "cache")]
    caching: bool,
    #[cfg(feature = "cache")]
    cache: DetectionCache,
}

impl DeviceDetector {
    #[cfg(not(feature = "cache"))]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "cache")]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            caching: false,
            cache: Cache::new(0),
        }
    }

    #[cfg(feature = "cache")]
    pub fn new_with_cache(entries: u64) -> Self {
        Self {
            caching: true,
            cache: Cache::new(entries),
        }
    }


    pub fn parse(&self, ua: &str, headers: Option<Vec<(String, String)>>) -> Result<Detection> {

        let parse = || {
            if let Some(bot) = bot::lookup_bot(ua)? {
                return Ok(Detection::Bot(bot));
            }

            let client_hints = match headers {
                Some(headers) => Some(ClientHint::from_headers(headers)?),
                None => None,
            };

            let os = oss::lookup(ua, client_hints.as_ref())?;

            let client = client::lookup(ua, client_hints.as_ref())?;

            let device = device::lookup(ua, client.as_ref(), client_hints.as_ref(), os.as_ref())?;

            let known = Detection::Known(KnownDevice { client, device, os });

            Ok::<_, anyhow::Error>(known)
        };

        #[cfg(feature = "cache")]
        {
            if !self.caching {
                return Ok(parse()?);
            }

            if let Some(res) = self.cache.get(ua) {
                return Ok(res);
            };

            let known = parse()?;

            self.cache.insert(ua.to_owned(), known.clone());

            Ok(known)
        }

        #[cfg(not(feature = "cache"))]
        parse()
    }
}
