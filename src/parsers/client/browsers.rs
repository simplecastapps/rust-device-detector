use anyhow::Result;

use fancy_regex::Regex;

use serde::Deserialize;

use version_compare::Cmp;

use std::cmp::Ordering;

use std::collections::HashMap;

use fallible_iterator::{convert, FallibleIterator};

use super::{Client, ClientType};
use crate::client_hints::{ClientHint, ClientHintMapping};
use crate::known_browsers::AvailableBrowsers;

use crate::parsers::utils::LazyRegex;

pub mod engines;

use once_cell::sync::Lazy;

static CLIENT_LIST: Lazy<BrowserClientList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/client/browsers.yml"
    ));
    BrowserClientList::from_file(contents).expect("loading browsers.yml")
});

static CLIENT_HINT_MAPPING: Lazy<ClientHintMapping> = Lazy::new(|| {
    ClientHintMapping::new(vec![
        ("Chrome".to_owned(), vec!["Google Chrome".to_owned()]),
        (
            "Chrome Webview".to_owned(),
            vec!["Android WebView".to_owned()],
        ),
        (
            "DuckDuckGo Privacy Browser".to_owned(),
            vec!["DuckDuckGo".to_owned()],
        ),
        (
            "Edge WebView".to_owned(),
            vec!["Microsoft Edge WebView2".to_owned()],
        ),
        ("Microsoft Edge".to_owned(), vec!["Edge".to_owned()]),
        (
            "Norton Private Browser".to_owned(),
            vec!["Norton Secure Browser".to_owned()],
        ),
        ("Vewd Browser".to_owned(), vec!["Vewd Core".to_owned()]),
    ])
});

static AVAILABLE_BROWSERS: Lazy<AvailableBrowsers> = Lazy::new(AvailableBrowsers::default);

pub fn lookup(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<Client>> {
    let client_from_ua: Option<Client> = CLIENT_LIST.lookup(ua)?;

    let mut client_from_hints = if let Some(client_hints) = client_hints {
        let client_hints_iter = convert(client_hints.full_version_list.iter().map(anyhow::Ok));
        let mut possible_results: Vec<_> = client_hints_iter
            .filter_map(|i| {
                let brand = CLIENT_HINT_MAPPING.apply(&i.0)?;
                let res = AVAILABLE_BROWSERS
                    .search_by_name(brand.trim())
                    .map(|x| (&i.0, &i.1, x));

                Ok(res)
            })
            .collect()?;

        // ensure chromium is the last result
        possible_results.sort_by_key(|x| x.0 == "Chromium" || x.0 == "Microsoft Edge");

        if let Some((brand_version, brand_result)) = possible_results.first().map(|x| (x.1, x.2)) {
            let version = if let Some(ua_full_version) = &client_hints.ua_full_version {
                Some(ua_full_version.to_owned())
            } else {
                Some(brand_version.to_owned())
            };

            let res = Client {
                name: brand_result.name.clone(),
                version,
                r#type: ClientType::Browser,
                engine: None,
                engine_version: None,
                browser: Some(brand_result.to_owned()),
            };
            Some(res)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(client_from_hints) = client_from_hints.as_mut() {
        if let Some(client_hints_version) = &client_from_hints.version {
            // If the version reported from the client hints is YYYY or YYYY.MM (e.g., 2022 or 2022.04), then it's Iridium.
            // https://iridiumbrowser.de/news/
            let iridium = ["2020", "2021", "2022", "2023", "2024"]
                .iter()
                .any(|year| client_hints_version.starts_with(year));
            if iridium {
                client_from_hints.name = "Iridium".to_owned();
            }

            // https://bbs.360.cn/thread-16096544-1-1.html
            if let Some(ua_client) = &client_from_ua {
                if let Some(ua_client_version) = &ua_client.version {
                    if client_hints_version.starts_with("15")
                        && ua_client_version.starts_with("114")
                    {
                        client_from_hints.name = "360 Secure Browser".to_owned();
                        client_from_hints.engine = ua_client.engine.clone();
                        client_from_hints.engine_version = ua_client.engine_version.clone();
                    }
                }
            }
        }

        if client_from_hints.name == "Atom" || client_from_hints.name == "Huawei Browser" {
            client_from_hints.version = client_from_ua
                .as_ref()
                .map(|x| x.version.clone())
                .unwrap_or_default();
        }

        if client_from_hints.name == "DuckDuckGo Privacy Browser" {
            client_from_hints.version = None;
        }

        if client_from_hints.name == "Vewd Browser" {
            client_from_hints.engine = client_from_ua
                .as_ref()
                .map(|x| x.engine.clone())
                .unwrap_or_default();
            client_from_hints.engine_version = client_from_ua
                .as_ref()
                .map(|x| x.engine_version.clone())
                .unwrap_or_default();
        }

        if client_from_hints.name == "Chromium" {
            if let Some(client) = &client_from_ua {
                if client.name != "Chromium" {
                    client_from_hints.name = client.name.clone();
                    client_from_hints.version = client.version.clone();
                }
            }
        }

        if let Some(client) = &client_from_ua {
            if client.name == format!("{} Mobile", client_from_hints.name) {
                client_from_hints.name = client_from_ua
                    .as_ref()
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
            }
        }

        if let Some(client) = &client_from_ua {
            #[allow(clippy::collapsible_if)]
            if client_from_hints.name != client.name {
                if client_from_hints
                    .browser
                    .as_ref()
                    .and_then(|browser| browser.family.as_ref())
                    .is_some()
                    && client_from_hints.browser.as_ref().map(|x| &x.family)
                        == client.browser.as_ref().map(|x| &x.family)
                {
                    client_from_hints.engine = client.engine.clone();
                    client_from_hints.engine_version = client.engine_version.clone();
                }
            }
        }

        if let Some(client) = &client_from_ua {
            if client_from_hints.name == client.name {
                client_from_hints.engine = client.engine.clone();
                client_from_hints.engine_version = client.engine_version.clone();

                #[allow(clippy::collapsible_if)]
                if let Some(client_version) = &client.version {
                    if let Some(client_from_hints_version) = &client_from_hints.version {
                        if client_version.starts_with(client_from_hints_version) {
                            if version_compare::compare(client_from_hints_version, client_version)
                                .unwrap_or(Cmp::Eq)
                                == Cmp::Lt
                            {
                                client_from_hints.version = client.version.clone();
                            }
                        }
                    }
                }
            }
        }
    };

    let mut res = client_from_hints.or(client_from_ua);

    if let Some(client) = res.as_mut() {
        if let Some(client_hints) = client_hints {
            if let Some(app_hint) = &client_hints.app {
                if let Some(app_name) = super::hints::browsers::get_hint(app_hint)? {
                    if client.name != app_name {
                        client.name = app_name.to_owned();
                        client.version = None;

                        if let Some(browser) = AVAILABLE_BROWSERS.search_by_name(app_name) {
                            static BLINK_REGEX: Lazy<Regex> = Lazy::new(|| {
                                Regex::new(r"Chrome/.+ Safari/537.36").expect("valid blink regex")
                            });

                            if BLINK_REGEX.is_match(ua)? {
                                client.engine = Some("Blink".to_owned());

                                if let Some(engine) = &client.engine {
                                    client.engine_version =
                                        BrowserClientList::engine_version(ua, engine)?;
                                }

                                let mut client_browser = browser.clone();
                                if client_browser.family.is_none() {
                                    client_browser.family = Some("Chrome".to_owned());
                                }

                                client.browser = Some(client_browser);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(client) = &mut res {
        if let Some(engine) = &client.engine {
            if engine == "Blink" && client.name == "Flow Browser" {
                client.engine_version = None;
            }

            if client.name == "Every Browser" {
                client.engine = Some("Blink".to_owned());
                client.engine_version = None;
            }
        }
    }

    Ok(res)
}

#[derive(Debug, Deserialize)]
struct BrowserClientEntry {
    name: String,
    #[serde(deserialize_with = "super::de_regex")]
    regex: LazyRegex,
    version: String,
    engine: Option<BrowserEngine>,
}

#[derive(Debug, Deserialize)]
struct BrowserEngine {
    default: Option<String>,
    #[serde(default)]
    versions: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
struct BrowserClientList {
    clients: Vec<BrowserClientEntry>,
}

impl BrowserClientList {
    pub fn lookup(&self, ua: &str) -> Result<Option<Client>> {
        for entry in self.clients.iter() {
            if entry.regex.is_match(ua)? {
                let mut name = "".to_owned();
                let mut version = "".to_owned();

                let caps = entry.regex.captures(ua)?.expect("valid_regex");

                caps.expand(&entry.version, &mut version);
                let version = if version.ends_with(&['.', ' ']) {
                    version.trim_end_matches(&['.', ' ']).to_owned()
                } else {
                    version
                };

                caps.expand(&entry.name, &mut name);

                // browsers are always have engine versions even if they are empty strings
                let mut engine = None;
                let mut engine_version = None;

                if let Some(entry_engine) = &entry.engine {
                    if let Some(e) = Self::engine(ua, entry_engine, &version)? {
                        engine = Some(e);
                    }
                }

                if engine.is_none() {
                    engine = self::engines::lookup(ua)?;
                }

                if let Some(e) = &engine {
                    if let Some(entry_version) = Self::engine_version(ua, e)? {
                        engine_version = Some(entry_version);
                    }
                }

                let browser = AVAILABLE_BROWSERS
                    .search_by_name(&name)
                    .map(|browser| browser.to_owned());

                let version = if version.is_empty() {
                    None
                } else {
                    Some(version)
                };

                return Ok(Some(Client {
                    name,
                    version,
                    r#type: ClientType::Browser,
                    engine,
                    engine_version,
                    browser,
                }));
            }
        }

        Ok(None)
    }

    fn engine_version(ua: &str, engine: &str) -> Result<Option<String>> {
        if engine.is_empty() {
            return Ok(None);
        }

        if engine == "Gecko" || engine == "Clecko" {
            static GECKO_VERSION: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r#"(?i:[ ](?:rv[: ]([0-9\.]+)).*(?:g|cl)ecko/[0-9]{8,10})"#)
                    .expect("valid browser regex")
            });

            for m in GECKO_VERSION.captures_iter(ua) {
                if let Some(r#match) = m?.get(1) {
                    return Ok(Some(r#match.as_str().to_owned()));
                }
            }
        }

        let mut token = engine;
        if engine == "Blink" {
            token = "(?:Chrome|Cronet)";
        } else if engine == "Arachne" {
            token = "(?:Arachne\\/5\\.)";
        } else if engine == "LibWeb" {
            token = "(?:LibWeb\\+LibJs)";
        }

        use crate::parsers::utils::LimitedUserMatchRegex;

        // There are very few browser engines, like 20 or less, so this
        // will never get very big, but recompiling the regex every time the
        // same engines come along is wasteful, so we're just going to share
        // the compiled regexes amongst threads to ease memory fragmentation.
        static ENGINE_VERSION_REGEXEN: Lazy<LimitedUserMatchRegex> =
            Lazy::new(|| LimitedUserMatchRegex::new(40));

        let reg = ENGINE_VERSION_REGEXEN.regex(token);

        if let Some(r#match) = reg.captures(ua)? {
            return Ok(Some(
                r#match.get(1).expect("browser version").as_str().to_owned(),
            ));
        }

        Ok(None)
    }

    fn engine(ua: &str, entry_engine: &BrowserEngine, version: &str) -> Result<Option<String>> {
        let mut engine = None;
        let mut engine_versions = entry_engine.versions.iter().collect::<Vec<_>>();

        // php code assumes the versions will be sorted as they are in
        // the file, but yaml doesn't guarantee that, so we sort them
        // lexographically, which is all we can do.

        engine_versions.sort_by(|&(a, _), &(b, _)| {
            version_compare::compare(a, b)
                .unwrap_or(Cmp::Eq)
                .ord()
                .unwrap_or(Ordering::Equal)
        });

        engine_versions
            .into_iter()
            .for_each(|(engine_version, eng)| {
                if let Cmp::Eq | Cmp::Gt =
                    version_compare::compare(version, engine_version).expect("valid version")
                {
                    engine = Some(eng.clone())
                }
            });

        engine = engine.or_else(|| entry_engine.default.clone());

        if engine.is_none() || engine.as_ref().unwrap() == "" {
            engine = self::engines::lookup(ua)?;
        }

        Ok(engine)
    }

    pub fn from_file(contents: &str) -> Result<Self> {
        let res = serde_yaml::from_str(contents)?;
        Ok(res)
    }
}
