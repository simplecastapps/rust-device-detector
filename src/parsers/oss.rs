use anyhow::Result;

use once_cell::sync::Lazy;
use serde_yaml::Value;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::client_hints::ClientHint;
use crate::known_oss::AvailableOSs;
use crate::parsers::utils::{
    lazy_user_agent_match, static_user_agent_match, LazyRegex, SafeRegex as Regex,
};


static OS_LIST: Lazy<OSList> = Lazy::new(|| {
    let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/regexes/oss.yml"));
    OSList::from_file(contents).expect("loading oss.yml")
});
static CLIENT_HINT_MAPPING: Lazy<Vec<(String, Vec<String>)>> = Lazy::new(|| {
    [("GNU/Linux", vec!["Linux"]), ("Mac", vec!["MacOS"])]
        .into_iter()
        .map(|(k, v)| {
            let oss = v.into_iter().map(|s| s.to_owned()).collect();
            (k.to_owned(), oss)
        })
        .collect::<Vec<(String, Vec<String>)>>()
});
static AVAILABLE_OSSES: Lazy<AvailableOSs> = Lazy::new(AvailableOSs::default);

#[derive(Clone, Debug, Default, Serialize)]
pub struct OS {
    pub name: String,
    pub version: Option<String>,
    pub platform: Option<String>,
    pub family: Option<String>,

    #[serde(skip)]
    pub(crate) desktop: bool,
}

pub fn lookup(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<OS>> {
    let mut os_from_hints: Option<OS> = client_hints.and_then(|client_hints| {
        if let Some(platform) = client_hints.platform.as_ref() {
            let hint = CLIENT_HINT_MAPPING
                .iter()
                .find(|(_k, v)| {
                    v.iter()
                        .any(|v| *v.to_lowercase() == platform.to_lowercase())
                })
                .map(|x| &x.0)
                .or(Some(platform));

            if let Some(os) = hint.and_then(|hint| AVAILABLE_OSSES.search_by_name(hint)) {
                let mut version = client_hints.platform_version.clone();

                if let Some(platform_version) = &client_hints.platform_version {
                    if os.name == "Windows" {
                        if let Some(major_version) = platform_version
                            .split('.')
                            .next()
                            .and_then(|x| x.parse::<u32>().ok())
                        {
                            if major_version > 0 && major_version < 11 {
                                version = Some("10".to_owned());
                            } else if major_version > 10 {
                                version = Some("11".to_owned());
                            }
                        }
                    }
                }

                Some(OS {
                    name: os.name.clone(),
                    version,
                    platform: None,
                    family: os.family.clone(),
                    desktop: os.desktop,
                })
            } else {
                None
            }
        } else {
            None
        }
    });

    let os_from_ua: Option<OS> = OS_LIST.lookup(ua)?;

    // various occasional overrides of client hint information based on ua.
    if let Some(ref mut os_from_hints) = &mut os_from_hints {
        if let Some(os_from_ua) = &os_from_ua {
            // use version from user agent if none was provided in client hints if the os family matches
            if os_from_hints.version.is_none()
                && os_from_ua.version.is_some()
                && os_from_hints.family == os_from_ua.family
            {
                os_from_hints.version = os_from_ua.version.clone();
            }

            // if OS name detected from client hints matches OS family of user agent but the user
            // agent os name is another, we use the one from the user agent as it might be more
            // detailed
            if let Some(ua_family) = &os_from_ua.family {
                if *ua_family == os_from_hints.name {
                    os_from_hints.name = os_from_ua.name.clone();

                    if os_from_hints.name == "HarmonyOS" {
                        os_from_hints.version = None;
                    }

                    if os_from_hints.name == "Fire OS" {
                        if let Some(os_hint_version) = os_from_hints.version.as_deref() {
                            static FIRE_OS_VERSION: Lazy<HashMap<&str, &str>> =
                                Lazy::new(||
                                          [
                                          ("11" , "8"),
                                          ("10" , "7"),
                                          ("9" , "7"),
                                          ("7" , "6"),
                                          ("5" , "5"),
                                          ("4.4.3" , "4.5.1"),
                                          ("4.4.2" , "4"),
                                          ("4.2.2" , "3"),
                                          ("4.0.3" , "3"),
                                          ("4.0.2" , "3"),
                                          ("4" , "2"),
                                          ("2" , "1"),
                                          ].into_iter().collect::<HashMap<_,_>>()
                                         );

                            let major_version = os_from_hints
                                .version
                                .as_ref()
                                .and_then(|x| x.split('.').next())
                                .unwrap_or("0");

                            if let Some(version) = FIRE_OS_VERSION.get(os_hint_version) {
                                os_from_hints.version = Some((*version).to_owned());
                            } else {
                                os_from_hints.version = FIRE_OS_VERSION.get(major_version).map(|x| (*x).to_owned());
                            }
                        }
                    }
                }
            }

            // Chrome OS is in some cases reported as Linux in client hints, we fix this only if
            // the version matches
            if os_from_hints.name == "GNU/Linux"
                && os_from_ua.name == "Chrome OS"
                && os_from_hints.version == os_from_ua.version
            {
                os_from_hints.name = os_from_ua.name.clone();
            }
        }
    }

    let mut res = os_from_hints.or(os_from_ua);

    if let Some(os) = &mut res {
        if let platform @ Some(_) = parse_platform(ua, client_hints)? {
            os.platform = platform
        }
    }

    if let Some(os) = &mut res {
        if let family @ Some(_) = AVAILABLE_OSSES
            .search_by_name(&os.name)
            .and_then(|x| x.family.clone())
        {
            os.family = family;
        }
    }

    let android_apps = [
        "com.hisense.odinbrowser",
        "com.seraphic.openinet.pre",
        "com.appssppa.idesktoppcbrowser",
        "every.browser.inc",
    ];

    if let Some(os) = &mut res {
        if os.name != "Android" {
            if let Some(client_hints) = &client_hints {
                if let Some(app_hint) = &client_hints.app {
                    if android_apps.iter().any(|app| *app == app_hint) {
                        os.name = "Android".to_owned();
                        os.family = Some("Android".to_owned());
                        os.version = None;
                    }
                }
            }
        }
    }

    Ok(res)
}

fn parse_platform(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<String>> {
    if let Some(client_hints) = client_hints {
        if let Some(architecture) = &client_hints.architecture {
            let arch = architecture.to_lowercase();

            if arch.contains("arm") {
                return Ok(Some("ARM".into()));
            }

            if arch.contains("mips") {
                return Ok(Some("MIPS".into()));
            }

            if arch.contains("sh4") {
                return Ok(Some("SuperH".into()));
            }

            if arch.contains("x64") {
                return Ok(Some("x64".into()));
            }

            if arch.contains("x86") {
                if let Some(bitness) = &client_hints.bitness {
                    if bitness == "64" {
                        return Ok(Some("x64".into()));
                    }
                }
            }

            if arch.contains("x86") {
                return Ok(Some("x86".into()));
            }
        }
    }

    static ARM_REG: Lazy<Regex> =
        static_user_agent_match!("arm|aarch64|Apple ?TV|Watch ?OS|Watch1,[12]");
    static MIPS_REG: Lazy<Regex> = static_user_agent_match!("mips");
    static SH4_REG: Lazy<Regex> = static_user_agent_match!("sh4");
    static X64_REG: Lazy<Regex> =
        static_user_agent_match!("64-?bit|WOW64|(?:Intel)?x64|WINDOWS_64|win64|amd64|x86_?64");
    static X86_REG: Lazy<Regex> = static_user_agent_match!(".+32bit|.+win32|(?:i[0-9]|x)86|i86pc");

    if ARM_REG.is_match(ua)? {
        return Ok(Some("ARM".into()));
    }

    if MIPS_REG.is_match(ua)? {
        return Ok(Some("MIPS".into()));
    }

    if SH4_REG.is_match(ua)? {
        return Ok(Some("SuperH".into()));
    }

    if X64_REG.is_match(ua)? {
        return Ok(Some("x64".into()));
    }

    if X86_REG.is_match(ua)? {
        return Ok(Some("x86".into()));
    }

    Ok(None)
}

struct OSList {
    oss: Vec<OSEntry>,
}

#[derive(Debug)]
struct OSEntry {
    regex: LazyRegex,
    name: String,
    version: Option<String>,
    versions: Vec<OSVersion>,
}

#[derive(Debug)]
struct OSVersion {
    regex: LazyRegex,
    version: String,
}

impl OSEntry {
    fn is_match(&self, ua: &str) -> Result<Option<OS>> {
        if self.regex.is_match(ua)? {
            let mut name = "".to_owned();
            let mut v = "".to_owned();
            if let Some(captures) = self.regex.captures(ua)? {
                captures.expand(&self.name, &mut name);

                if let Some(res) = AVAILABLE_OSSES.search_by_name(&name) {
                    name = res.name.to_owned();
                }

                for version in &self.versions {
                    if let Some(captures) = version.regex.captures(ua)? {
                        captures.expand(&version.version, &mut v);
                        break;
                    }
                }

                if v.is_empty() {
                    if let Some(version) = &self.version {
                        captures.expand(version, &mut v);
                    }
                }
                if v.contains('_') {
                    v = v.replace('_', ".");
                }
                if v.ends_with(['.', ' ']) || v.starts_with(['.', ' ']) {
                    v = v
                        .trim_end_matches(['.', ' '])
                        .trim_start_matches(['.', ' '])
                        .to_owned();
                }
            }

            let v = if !v.is_empty() { Some(v) } else { None };

            let mut os = OS {
                name,
                version: v,
                ..Default::default()
            };

            if let Some(av_os) = AVAILABLE_OSSES.search_by_name(&os.name) {
                os.family = av_os.family.clone();
                os.desktop = av_os.desktop;
            }

            return Ok(Some(os));
        }
        Ok(None)
    }
}

impl OSList {
    fn lookup(&self, ua: &str) -> Result<Option<OS>> {
        for os in self.oss.iter() {
            if let Some(res) = os.is_match(ua)? {
                return Ok(Some(res));
            }
        }

        Ok(None)
    }

    fn from_file(contents: &str) -> Result<OSList> {
        #[derive(Debug, Deserialize)]
        #[serde(try_from = "Value")]
        struct YamlVersion {
            regex: Option<String>,
            version: String,
        }

        impl From<Value> for YamlVersion {
            fn from(value: Value) -> Self {
                match value {
                    Value::String(s) => YamlVersion {
                        regex: None,
                        version: s,
                    },
                    Value::Mapping(m) => {
                        let regex: Option<String> = m
                            .get("regex")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_owned());

                        let version = m
                            .get("version")
                            .and_then(|x| x.as_str())
                            .expect("missing version in os")
                            .to_owned();
                        YamlVersion { regex, version }
                    }
                    _ => panic!("expected string or object with version and regex strings"),
                }
            }
        }

        #[derive(Debug, Deserialize)]
        struct YamlOSEntry {
            name: String,
            regex: String,
            #[serde(default)]
            version: Option<String>,
            #[serde(default)]
            versions: Vec<YamlVersion>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<OSEntry> for YamlOSEntry {
            fn into(self) -> OSEntry {
                let version = self.version;
                let versions = self
                    .versions
                    .into_iter()
                    .map(|x| OSVersion {
                        regex: x
                            .regex
                            // either use the regex for this version, or use the top
                            // level regex if there is none.
                            .map(|x| lazy_user_agent_match(&x))
                            .unwrap_or_else(|| lazy_user_agent_match(&self.regex)),

                        version: x.version,
                    })
                    .collect();

                OSEntry {
                    regex: lazy_user_agent_match(&self.regex),
                    name: self.name,
                    version,
                    versions,
                }
            }
        }

        #[derive(Debug, Deserialize)]
        #[serde(transparent)]
        struct YamlOSList {
            oss: Vec<YamlOSEntry>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<OSList> for YamlOSList {
            fn into(self) -> OSList {
                OSList {
                    oss: self.oss.into_iter().map(|x| x.into()).collect(),
                }
            }
        }

        let res: YamlOSList = serde_yaml::from_str(contents)?;
        Ok(res.into())
    }
}
