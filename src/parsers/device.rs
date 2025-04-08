use anyhow::Result;

use serde::{Deserialize, Serialize};

use serde_yaml::Value;

use once_cell::sync::Lazy;

use version_compare::{self, Version};

use super::vendor_fragments;

use std::borrow::Cow;
use get_size::GetSize;
use crate::client_hints::ClientHint;
use crate::parsers::client::{Client, ClientType};
use crate::parsers::oss::OS;

use crate::parsers::utils::{
    lazy_user_agent_match, static_user_agent_match, LazyRegex, SafeRegex as Regex,
};

pub mod cameras;
pub mod car_browsers;
pub mod consoles;
pub mod mobiles;
pub mod notebooks;
pub mod portable_media_players;
pub mod shell_tvs;
pub mod televisions;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, GetSize)]
pub enum DeviceType {
    #[serde(rename = "desktop")]
    Desktop,
    #[serde(rename = "smartphone")]
    SmartPhone,
    #[serde(rename = "feature phone")]
    FeaturePhone,
    #[serde(rename = "tablet")]
    Tablet,
    #[serde(rename = "phablet")]
    Phablet,
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "portable media player")]
    PortableMediaPlayer,
    #[serde(rename = "car browser")]
    CarBrowser,
    #[serde(rename = "television")]
    Television,
    #[serde(rename = "smart display")]
    SmartDisplay,
    #[serde(rename = "smart speaker")]
    SmartSpeaker,
    #[serde(rename = "camera")]
    Camera,
    #[serde(rename = "notebook")]
    Notebook,
    #[serde(rename = "wearable")]
    Wearable,
    #[serde(rename = "peripheral")]
    Peripheral,
}

static APPLE_OS_NAMES: [&str; 5] = ["iPadOS", "tvOS", "watchOS", "iOS", "Mac"];

impl DeviceType {
    // these are used basically entirely for tests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SmartPhone => "smartphone",
            Self::FeaturePhone => "feature phone",
            Self::Tablet => "tablet",
            Self::Phablet => "phablet",
            Self::Console => "console",
            Self::PortableMediaPlayer => "portable media player",
            Self::CarBrowser => "car browser",
            Self::Television => "tv",
            Self::SmartDisplay => "smart display",
            Self::SmartSpeaker => "smart speaker",
            Self::Camera => "camera",
            Self::Notebook => "notebook",
            Self::Wearable => "wearable",
            Self::Peripheral => "peripheral",
            Self::Desktop => "desktop",
        }
    }
    pub fn from_str(name: &str) -> DeviceType {
        match name {
            "desktop" => Self::Desktop,
            "smartphone" => Self::SmartPhone,
            "feature phone" => Self::FeaturePhone,
            "tablet" => Self::Tablet,
            "phablet" => Self::Phablet,
            "console" => Self::Console,
            "portable media player" => Self::PortableMediaPlayer,
            "car browser" => Self::CarBrowser,
            "tv" => Self::Television,
            "smart display" => Self::SmartDisplay,
            "smart speaker" => Self::SmartSpeaker,
            "camera" => Self::Camera,
            "notebook" => Self::Notebook,
            "wearable" => Self::Wearable,
            "peripheral" => Self::Peripheral,
            _ => panic!("Unknown device type {}", name),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, GetSize)]
pub struct Device {
    #[serde(rename = "type")]
    pub device_type: Option<DeviceType>,
    pub brand: Option<String>,
    pub model: Option<String>,

    // Can be gotten from headers, and can solely determine whether
    // this is a mobile device or not regardless of everything else.
    #[serde(skip)]
    pub(crate) mobile_client_hint: bool,
    #[serde(skip)]
    pub(crate) touch_enabled: bool,
}

#[derive(Debug)]
pub struct DeviceList {
    devices: Vec<(String, DeviceEntry)>,
}

#[derive(Debug)]
struct ModelEntry {
    regex: Option<LazyRegex>,
    device: Option<DeviceType>,
    model: String,
    brand: Option<String>,
}

#[derive(Debug)]
struct DeviceEntry {
    device: Option<String>,
    regex: LazyRegex,
    models: Vec<ModelEntry>,
}
#[derive(Debug)]
struct DeviceMatchResult {
    device: Option<DeviceType>,
    model: Option<ModelMatchResult>,
}

#[derive(Debug)]
struct ModelMatchResult {
    model: String,
    device: Option<DeviceType>,
    brand: Option<String>,
}

//This is prominently featured in the php code, but I can find no examples in which
//it is hit.
//pub fn has_desktop_fragment(ua: &str) -> bool {
//    lazy_static! {
//        static R1: Lazy<Regex> =
//            static_user_agent_match!(r#"(?:Windows (?:NT|IoT)|X11; Linux x86_64)"#);
//        static R2: Lazy<Regex> =
//            static_user_agent_match!(r#"CE-HTML");
//        static R3: Lazy<Regex> = static_user_agent_match!(
//            r#" Mozilla/|Andr[o0]id|Tablet|Mobile|iPhone|Windows Phone|ricoh|OculusBrowser"#
//        );
//        static R4: Lazy<Regex> =
//            static_user_agent_match!(r#"Lenovo|compatible; MSIE|Trident/|Tesla/|XBOX|FBMD/|ARM; ?([^)]+)"#);
//    }
//
//    R1.is_match(ua).unwrap() && !R2.is_match(ua) && !R3.is_match(ua).unwrap() && !R4.is_match(ua).unwrap()
//}

pub fn lookup(
    ua: &str,
    client: Option<&Client>,
    client_hints: Option<&ClientHint>,
    os_info: Option<&OS>,
) -> Result<Option<Device>> {
    static ANDROID_10_MODEL: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(Android 10[.\d]*; K)(?: Build/|[;)])").unwrap());

    // this sets the model into the device string so that further
    // user agent level device detection will find it.
    let ua: Cow<'_, str> = 'ua: {
        if let Some(client_hints) = &client_hints {
            if let Some(model) = &client_hints.model {
                if let Some(captures) = ANDROID_10_MODEL.captures(ua)? {
                    let os_version: &str =
                        os_info.and_then(|os| os.version.as_deref()).unwrap_or("10");
                    let replacement = format!("Android {}; {}", os_version, model);
                    let res = ua.replace(&captures[1], &replacement);

                    break 'ua Cow::Owned(res);
                }
            }
        }

        Cow::Borrowed(ua)
    };

    let mut device = 'dev: {
        if let Some(res) = televisions::lookup(&ua)? {
            break 'dev res;
        }

        if let Some(res) = shell_tvs::lookup(&ua)? {
            break 'dev res;
        }
        if let Some(res) = notebooks::lookup(&ua)? {
            break 'dev res;
        }

        if let Some(res) = consoles::lookup(&ua)? {
            break 'dev res;
        }

        if let Some(res) = car_browsers::lookup(&ua)? {
            break 'dev res;
        }

        if let Some(res) = cameras::lookup(&ua)? {
            break 'dev res;
        }
        if let Some(res) = portable_media_players::lookup(&ua)? {
            break 'dev res;
        }

        if let Some(res) = mobiles::lookup(&ua)? {
            break 'dev res;
        }

        Device {
            device_type: None,
            model: None,
            brand: None,
            ..Default::default()
        }
    };

    static TOUCH: Lazy<Regex> = static_user_agent_match!(r#"Touch"#);

    if TOUCH.is_match(&ua)? {
        device.touch_enabled = true;
    }

    if let Some(client_hints) = client_hints {
        if device.model.is_none() && client_hints.model.is_some() {
            device.model = client_hints.model.clone();
        }

        if client_hints.mobile {
            device.mobile_client_hint = true;
        }
    }

    if device.brand.is_none() {
        if let Some(brand) = vendor_fragments::lookup(&ua)? {
            device.brand = Some(brand.to_owned());
        }
    }

    if let Some(os) = &os_info {
        if let Some(brand) = &device.brand {
            if brand == "Apple" && !APPLE_OS_NAMES.iter().any(|x| *x == os.name) {
                device.device_type = None;
                device.brand = None;
                device.model = None;
            }
        }

        if device.brand.is_none() {
            if APPLE_OS_NAMES.iter().any(|x| *x == os.name) {
                device.brand = Some("Apple".to_owned());
            }
        }
    }

    static APAD_TABLET: Lazy<Regex> = static_user_agent_match!(r#"Pad/APad"#);
    static ANDROID_TABLET: Lazy<Regex> =
        static_user_agent_match!(r#"Android( [\.0-9]+)?; Tablet;|Tablet(?! PC)|.*\-tablet$"#);
    static ANDROID_MOBILE: Lazy<Regex> =
        static_user_agent_match!(r#"Android( [\.0-9]+)?; Mobile;|.*\-mobile$"#);
    static ANDROID_VR: Lazy<Regex> =
        static_user_agent_match!(r#"Android( [\.0-9]+)?; Mobile VR;| VR "#);
    static OPERA_TABLET: Lazy<Regex> = static_user_agent_match!(r#"Opera Tablet"#);

    if device.device_type.is_none() && ANDROID_VR.is_match(&ua)? {
        device.device_type = Some(DeviceType::Wearable);
    }

    if let Some(os) = &os_info {
        if device.device_type.is_none() {
            static CHROME: Lazy<Regex> = static_user_agent_match!(r#"Chrome/[\.0-9]*"#);
            static SAFARI_PHONE: Lazy<Regex> = static_user_agent_match!(r#"(?:Mobile|eliboM)"#);
            static SAFARI_TAB: Lazy<Regex> = static_user_agent_match!(r#"(?!Mobile )Safari"#);
            if let Some(family) = &os.family {
                if family == "Android" && CHROME.is_match(&ua)? {
                    if SAFARI_PHONE.is_match(&ua)? {
                        device.device_type = Some(DeviceType::SmartPhone);
                    } else if SAFARI_TAB.is_match(&ua)? {
                        device.device_type = Some(DeviceType::Tablet);
                    }
                }
            }
        }
    }

    if device.device_type == Some(DeviceType::SmartPhone) && APAD_TABLET.is_match(&ua)? {
        device.device_type = Some(DeviceType::Tablet);
    }

    if device.device_type.is_none()
        && (ANDROID_TABLET.is_match(&ua)? || OPERA_TABLET.is_match(&ua)?)
    {
        device.device_type = Some(DeviceType::Tablet);
    }

    if device.device_type.is_none() && ANDROID_MOBILE.is_match(&ua)? {
        device.device_type = Some(DeviceType::SmartPhone);
    }

    if let Some(os) = &os_info {
        static V2: Lazy<Version> = Lazy::new(|| Version::from("2.0").unwrap());
        static V3: Lazy<Version> = Lazy::new(|| Version::from("3.0").unwrap());
        static V4: Lazy<Version> = Lazy::new(|| Version::from("4.0").unwrap());
        static V8: Lazy<Version> = Lazy::new(|| Version::from("8.0").unwrap());

        if device.device_type.is_none() && os.name == "Android" {
            if let Some(os_version) = os.version.as_ref() {
                if let Some(os_version) = Version::from(os_version) {
                    if os_version < *V2 {
                        device.device_type = Some(DeviceType::SmartPhone);
                    } else if os_version >= *V3 && os_version < *V4 {
                        device.device_type = Some(DeviceType::Tablet);
                    }
                }
            }
        }
        if let Some(device_type) = &device.device_type {
            if *device_type == DeviceType::FeaturePhone {
                if let Some(family) = &os.family {
                    if family == "Android" {
                        device.device_type = Some(DeviceType::SmartPhone);
                    }
                }
            }
        }

        if device.device_type.is_none() && os.name == "Java ME" {
            device.device_type = Some(DeviceType::FeaturePhone);
        }

        if device.device_type.is_none() {
            if os.name == "Windows RT" {
                device.device_type = Some(DeviceType::Tablet);
            }
            if let Some(os_version) = os.version.as_ref() {
                if let Some(os_version) = Version::from(os_version) {
                    if os.name == "Windows RT"
                        || (os.name == "Windows" && os_version >= *V8 && is_touch(&ua)?)
                    {
                        device.device_type = Some(DeviceType::Tablet);
                    }
                }
            }
        }
    }

    static OPERA: Lazy<Regex> = static_user_agent_match!(r#"Opera TV Store| OMI/"#);
    static ANDR0ID: Lazy<Regex> =
        static_user_agent_match!(r#"Andr0id|(?:Android(?: UHD)?|Google) TV|\(lite\) TV|BRAVIA'"#);
    static TIZEN: Lazy<Regex> = static_user_agent_match!(r#"SmartTV|Tizen.+ TV .+$"#);
    static GENERIC_TV: Lazy<Regex> = static_user_agent_match!(r#"\(TV;"#);

    if OPERA.is_match(&ua)? {
        device.device_type = Some(DeviceType::Television);
    }
    if ANDR0ID.is_match(&ua)? {
        device.device_type = Some(DeviceType::Television);
    }
    if device.device_type.is_none() && TIZEN.is_match(&ua)? {
        device.device_type = Some(DeviceType::Television);
    }

    if let Some(client) = client {
        if [
            "Kylo",
            "Espial TV Browser",
            "LUJO TV Browser",
            "LogicUI TV Browser",
            "Open TV Browser",
            "Seraphic Sraf",
            "Opera Devices",
            "Crow Browser",
            "Vewd Browser",
            "TiviMate",
            "Quick Search TV",
            "QJY TV Browser",
            "TV Bro",
        ]
        .iter()
        .any(|x| *x == client.name)
        {
            device.device_type = Some(DeviceType::Television);
        }
    }

    if device.device_type.is_none() && GENERIC_TV.is_match(&ua)? {
        device.device_type = Some(DeviceType::Television);
    }

    static DESKTOP_FRAGMENT: Lazy<Regex> =
        static_user_agent_match!(r#"Desktop(?: (x(?:32|64)|WOW64))?;"#);

    if let Some(device_type) = &device.device_type {
        if *device_type != DeviceType::Desktop
            && ua.contains("Desktop")
            && DESKTOP_FRAGMENT.is_match(&ua)?
        {
            device.device_type = Some(DeviceType::Desktop);
        }
    }
    if device.device_type.is_none() && DESKTOP_FRAGMENT.is_match(&ua)? {
        device.device_type = Some(DeviceType::Desktop);
    }

    if device.device_type.is_none() && is_desktop(os_info, client) {
        device.device_type = Some(DeviceType::Desktop);
    }

    if device.device_type.is_none() && device.brand.is_none() && device.model.is_none() {
        Ok(None)
    } else {
        Ok(Some(device))
    }
}

fn is_desktop(os: Option<&OS>, client: Option<&Client>) -> bool {
    // TODO FIXME if name is "Unknown" or some variant?
    if os.is_none() {
        return false;
    }

    if let Some(client) = &client {
        if uses_mobile_browser(client) {
            return false;
        }
    }

    if let Some(os) = &os {
        return os.desktop;
    }

    false
}

pub(crate) fn uses_mobile_browser(client: &Client) -> bool {
    if client.r#type == ClientType::Browser {
        if let Some(browser) = &client.browser {
            return browser.mobile_only;
        }
    }

    false
}

fn is_touch(ua: &str) -> Result<bool> {
    static TOUCH: Lazy<Regex> = static_user_agent_match!(r#"Touch"#);

    let res = TOUCH.is_match(ua)?;
    Ok(res)
}

impl DeviceList {
    fn lookup(&self, ua: &str, _type: &str) -> Result<Option<Device>> {
        for (name, device) in self.devices.iter() {
            if let Some(match_result) = device.lookup(ua)? {
                static TD: Lazy<Regex> = Lazy::new(|| Regex::new(r#" [Tt][Dd]$"#).unwrap());

                let mut model: Option<String> =
                    match match_result.model.as_ref().map(|x| x.model.as_str()) {
                        None => None,
                        Some("") => None,
                        Some(model) => Some(
                            TD.replace_all(model.replace('_', " ").trim(), "")
                                .into_owned(),
                        ),
                    };

                if let Some(m) = &model {
                    if m == "Build" {
                        model = None;
                    }
                }

                let device_type: Option<DeviceType> = match_result
                    .model
                    .as_ref()
                    .and_then(|model| model.device.as_ref())
                    .or(match_result.device.as_ref())
                    .cloned();

                let mut brand = match_result
                    .model
                    .as_ref()
                    .and_then(|model| model.brand.as_ref())
                    .or(Some(name))
                    .cloned();

                if let Some(b) = &brand {
                    if b == "Unknown" {
                        brand = None;
                    }
                }

                let dev = Device {
                    device_type,
                    model,
                    brand,
                    ..Default::default()
                };

                return Ok(Some(dev));
            }
        }

        Ok(None)
    }

    fn from_file(file_contents: &str) -> Result<DeviceList> {
        #[derive(Debug, Deserialize)]
        #[serde(try_from = "Value")]
        struct YamlModelEntry {
            regex: Option<String>,
            device: Option<String>,
            model: String,
            brand: Option<String>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<ModelEntry> for YamlModelEntry {
            fn into(self) -> ModelEntry {
                ModelEntry {
                    regex: self.regex.map(|x| lazy_user_agent_match(&x)),
                    device: self.device.map(|device| DeviceType::from_str(&device)),
                    model: self.model,
                    brand: self.brand,
                }
            }
        }

        #[derive(Debug, Deserialize)]
        struct YamlDeviceEntry {
            device: Option<String>,
            regex: String,
            #[serde(default)]
            model: Option<YamlModelEntry>,
            #[serde(default)]
            models: Vec<YamlModelEntry>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<DeviceEntry> for YamlDeviceEntry {
            fn into(self) -> DeviceEntry {
                let mut models = Vec::with_capacity(self.models.len() + 1);

                if let Some(model) = self.model {
                    models.push(model.into());
                }

                models.extend(self.models.into_iter().map(|x| x.into()));

                DeviceEntry {
                    regex: lazy_user_agent_match(&self.regex),
                    device: self.device,
                    models,
                }
            }
        }

        #[derive(Debug, Deserialize)]
        #[serde(transparent)]
        struct YamlDeviceList {
            // The php library relies in the case of devices for the various
            // device yaml files to be in order to get the right result, so
            // we also have to do that.
            devices: indexmap::IndexMap<String, YamlDeviceEntry>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<DeviceList> for YamlDeviceList {
            fn into(self) -> DeviceList {
                let mut devices = Vec::with_capacity(self.devices.len());
                for (k, v) in self.devices.into_iter() {
                    devices.push((k, v.into()));
                }

                // There are some regexes in the device yml that match multiple devices,
                // such as Intex matches Aqua, and the php lib depends on order
                // to get the right answer.
                // devices.sort_by(|a, b| a.0.cmp(&b.0));

                DeviceList { devices }
            }
        }

        impl TryFrom<Value> for YamlModelEntry {
            type Error = anyhow::Error;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                let model = match value {
                    Value::String(str) => YamlModelEntry {
                        regex: None,
                        device: None,
                        brand: None,
                        model: str,
                    },
                    Value::Mapping(mappings) => {
                        let model = mappings["model"]
                            .as_str()
                            .ok_or(anyhow::anyhow!("invalid model"))?;

                        let regex = mappings["regex"].as_str();
                        let device = mappings.get("device").and_then(|x| x.as_str());
                        let brand = mappings.get("brand").and_then(|x| x.as_str());

                        YamlModelEntry {
                            regex: regex.map(|x| x.to_owned()),
                            device: device.map(|x| x.to_owned()),
                            brand: brand.map(|x| x.to_owned()),
                            model: model.to_owned(),
                        }
                    }
                    err => Err(anyhow::anyhow!("Invalid model {:?}", err))?,
                };
                Ok(model)
            }
        }

        let res: YamlDeviceList = serde_yaml::from_str(file_contents)?;
        Ok(res.into())
    }
}

impl DeviceEntry {
    fn lookup(&self, ua: &str) -> Result<Option<DeviceMatchResult>> {
        let res = if let Some(captures) = self.regex.captures(ua)? {
            if let Some(mut model) = self.model_match(ua)? {
                let mut m = "".to_owned();

                // php will treat $1[0-9]+ as $1 and so that bug constantly
                // creeps into the php code because tests don't catch it.
                // The fix inevitably comes down when someone reports it, but
                // we can just fix the most common case here to side step the
                // issue 99.999% of the time.
                if model.model.contains("$10") {
                  model.model = model.model.replace("$1", "${1}");
                }

                captures.expand(&model.model, &mut m);
                model.model = m;

                Some(DeviceMatchResult {
                    model: Some(model),
                    device: self
                        .device
                        .as_ref()
                        .map(|device| DeviceType::from_str(device)),
                })
            } else {
                Some(DeviceMatchResult {
                    model: None,
                    device: self
                        .device
                        .as_ref()
                        .map(|device| DeviceType::from_str(device)),
                })
            }
        } else {
            None
        };

        Ok(res)
    }

    fn model_match(&self, ua: &str) -> Result<Option<ModelMatchResult>> {
        for model in self.models.iter() {
            let res = model_match(model, ua)?;
            if res.is_some() {
                return Ok(res);
            }
        }

        Ok(None)
    }
}

fn model_match(model: &ModelEntry, ua: &str) -> Result<Option<ModelMatchResult>> {
    let res = match &model.regex {
        Some(regex) => match regex.captures(ua)? {
            Some(caps) => {
                let mut m = "".to_owned();

                crate::parsers::utils::expand(&model.model, &mut m, &caps);
                // caps.expand(&model.model, &mut m);

                Some(ModelMatchResult {
                    model: m,
                    device: model.device.as_ref().map(|x| x.to_owned()),
                    brand: model.brand.as_ref().map(|x| x.to_owned()),
                })
            }
            _ => None,
        },
        None => Some(ModelMatchResult {
            model: model.model.clone(),
            device: model.device.as_ref().map(|x| x.to_owned()),
            brand: None,
        }),
    };

    Ok(res)
}
