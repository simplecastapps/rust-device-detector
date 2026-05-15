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
use engines::detect_engine_version;

use once_cell::sync::Lazy;

// Helper function to extract version from user agent for app-based browsers
fn extract_version_from_ua(ua: &str, app_hint: &str) -> Result<Option<String>> {
    // Escape special regex characters in the app hint
    let escaped_app = app_hint.replace(".", r"\.");
    let pattern = format!(r"{}/(\d+[\.\d]+)", escaped_app);
    let regex = Regex::new(&pattern)?;
    
    if let Some(captures) = regex.captures(ua)? {
        if let Some(version_match) = captures.get(1) {
            return Ok(Some(version_match.as_str().to_owned()));
        }
    }
    
    Ok(None)
}

// Browsers that need special version handling early in the process before other logic runs
// Corresponds to PHP's short codes: A0, HP, MU, VR, JR
const BROWSERS_NEEDING_EARLY_VERSION_HANDLING: &[&str] = &[
    "Atom",           // A0 - Needs UA version instead of client hints version
    "Huawei Browser", // HP - Needs UA version for accurate detection
    "Mi Browser",     // MU - Needs UA version due to client hints inconsistencies
    "Veera",          // VR - UA version is more detailed than hints major version
    "OJR Browser",    // JR - UA version is more detailed than hints major version
];

// Browsers that need user agent version after standard processing (final override)
const BROWSERS_USING_UA_VERSION_FINAL: &[&str] = &[
    "Aloha Browser", "JioSphere", "mCent", "Opera", "Opera Mini", "Opera Mobile"
];

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
        (
            "Mi Browser".to_owned(),
            vec!["Miui Browser".to_owned(), "XiaoMiBrowser".to_owned()],
        ),
        ("Microsoft Edge".to_owned(), vec!["Edge".to_owned()]),
        (
            "Norton Private Browser".to_owned(),
            vec!["Norton Secure Browser".to_owned()],
        ),
        (
            "Opera GX".to_owned(),
            vec!["Opera GX Android".to_owned()],
        ),
        (
            "Opera Mini".to_owned(),
            vec!["Opera Mini Android".to_owned()],
        ),
        ("Vewd Browser".to_owned(), vec!["Vewd Core".to_owned()]),
        (
            "Yandex Browser".to_owned(),
            vec!["YaSearchBrowser".to_owned()],
        ),
    ])
});

static AVAILABLE_BROWSERS: Lazy<AvailableBrowsers> = Lazy::new(AvailableBrowsers::default);

pub fn lookup(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<Client>> {
    let client_from_ua: Option<Client> = CLIENT_LIST.lookup(ua)?;

    let mut client_from_hints = if let Some(client_hints) = client_hints {
        // Deduplicate brands like PHP's array_combine: last occurrence of a brand wins.
        // E.g., "Blazer";v="3", ..., "Blazer";v="140" → use version "140".
        let mut brand_map: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
        for (brand, version) in &client_hints.full_version_list {
            brand_map.insert(brand.clone(), version.as_str());
        }
        let client_hints_iter = convert(client_hints.full_version_list.iter().map(anyhow::Ok));
        let mut possible_results: Vec<_> = client_hints_iter
            .filter_map(|i| {
                // Skip if this brand appeared later with a different version (use last occurrence)
                if let Some(&latest_version) = brand_map.get(&i.0) {
                    if latest_version != i.1.as_str() {
                        return Ok(None);
                    }
                }
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
            
            // Determine engine based on browser
            let mut engine = None;
            let mut engine_version = None;
            
            // Chrome, Chromium, Edge and Chrome-based browsers use Blink engine
            if ["Chrome", "Chromium", "Microsoft Edge", "Edge", "CCleaner", "AVG Secure Browser"].contains(&brand_result.name.as_str()) {
                engine = Some("Blink".to_owned());
                
                // First get engine version from User Agent (like PHP does)
                let ua_engine_version = detect_engine_version(ua, "Blink").unwrap_or(None);
                
                // Get client hints version for comparison
                // PHP uses the browser version from client hints as engine version
                let client_hints_version = version.clone();
                
                // Follow PHP logic: use client hints version only if it's more detailed than UA version
                // and the browser is not "Iridium"
                if brand_result.name != "Iridium" {
                    if let (Some(ua_version), Some(ch_version)) = (&ua_engine_version, &client_hints_version) {
                        // Use client hints version if it's greater than UA version
                        if version_compare::compare(ch_version, ua_version) == Ok(version_compare::Cmp::Gt) {
                            engine_version = client_hints_version;
                        } else {
                            engine_version = ua_engine_version;
                        }
                    } else {
                        // Fallback: use whichever version is available
                        engine_version = client_hints_version.or(ua_engine_version);
                    }
                } else {
                    // For Iridium, always use UA version
                    engine_version = ua_engine_version;
                }
            }

            let res = Client {
                name: brand_result.name.clone(),
                version,
                r#type: ClientType::Browser,
                engine,
                engine_version: engine_version.clone(),
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
                // Engine version must come from the UA (Chrome/x.y.z), not the year-based hints version.
                client_from_hints.engine_version = client_from_ua
                    .as_ref()
                    .and_then(|c| c.engine_version.clone());
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

        // Some browsers need special version handling early in the process
        if BROWSERS_NEEDING_EARLY_VERSION_HANDLING.contains(&client_from_hints.name.as_str()) {
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

        // If client hints report Chromium or Chrome Webview, but user agent detects a Chromium-based
        // browser, we favor the UA detection instead (matching PHP logic)
        if client_from_hints.name == "Chromium" || client_from_hints.name == "Chrome Webview" {
            if let Some(client) = &client_from_ua {
                // PHP excludes: CR (Chromium), CV (Chrome Webview), AN (Android Browser), CM (Chrome Mobile)
                if !["Chromium", "Chrome Webview", "Android Browser", "Chrome Mobile"]
                    .contains(&client.name.as_str())
                {
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
                    // Only override engine version if client hints doesn't have one, or if UA version is more detailed
                    if let (Some(ua_engine_version), Some(ch_engine_version)) = (&client.engine_version, &client_from_hints.engine_version) {
                        // Keep the more detailed version
                        if version_compare::compare(ua_engine_version, ch_engine_version) == Ok(version_compare::Cmp::Gt) {
                            client_from_hints.engine_version = client.engine_version.clone();
                        }
                        // Otherwise keep the client hints version
                    } else if client_from_hints.engine_version.is_none() {
                        // For Chrome-family browsers, the browser version IS the engine version.
                        // Compare the hints browser version against the UA engine version and use
                        // whichever is more detailed. Also consider ua_full_version as a fallback
                        // when the hints browser version was cleared (e.g., DuckDuckGo Privacy Browser).
                        let hints_version = client_from_hints.version.as_deref()
                            .or_else(|| client_hints.and_then(|h| h.ua_full_version.as_deref()));
                        match (hints_version, client.engine_version.as_deref()) {
                            (Some(hv), Some(uv)) => {
                                if version_compare::compare(hv, uv) == Ok(version_compare::Cmp::Gt) {
                                    client_from_hints.engine_version = Some(hv.to_owned());
                                } else {
                                    client_from_hints.engine_version = client.engine_version.clone();
                                }
                            }
                            _ => {
                                client_from_hints.engine_version = client.engine_version.clone();
                            }
                        }
                    }
                }
            }
        }

        if let Some(client) = &client_from_ua {
            if client_from_hints.name == client.name {
                client_from_hints.engine = client.engine.clone();
                // Only override engine version if client hints doesn't have one, or if UA version is more detailed
                if let (Some(ua_engine_version), Some(ch_engine_version)) = (&client.engine_version, &client_from_hints.engine_version) {
                    // Keep the more detailed version
                    if version_compare::compare(ua_engine_version, ch_engine_version) == Ok(version_compare::Cmp::Gt) {
                        client_from_hints.engine_version = client.engine_version.clone();
                    }
                    // Otherwise keep the client hints version
                } else if client_from_hints.engine_version.is_none() {
                    // Compare hints browser version against UA engine version; use the higher one.
                    // Also consider ua_full_version as fallback when hints version was cleared.
                    let hints_version = client_from_hints.version.as_deref()
                        .or_else(|| client_hints.and_then(|h| h.ua_full_version.as_deref()));
                    match (hints_version, client.engine_version.as_deref()) {
                        (Some(hv), Some(uv)) => {
                            if version_compare::compare(hv, uv) == Ok(version_compare::Cmp::Gt) {
                                client_from_hints.engine_version = Some(hv.to_owned());
                            } else {
                                client_from_hints.engine_version = client.engine_version.clone();
                            }
                        }
                        _ => {
                            client_from_hints.engine_version = client.engine_version.clone();
                        }
                    }
                }
            }
        }

        // In case the user agent reports a more detailed version, we try to use this instead
        // This applies regardless of whether browser names match (e.g., "106.0.0.0" vs "106")
        if let Some(client) = &client_from_ua {
            if let Some(client_version) = &client.version {
                if let Some(client_from_hints_version) = &client_from_hints.version {
                    if client_version.starts_with(client_from_hints_version) {
                        // If the user agent version is longer/more detailed than the client hints version,
                        // use the more detailed version from the user agent
                        if client_version.len() > client_from_hints_version.len() {
                            client_from_hints.version = client.version.clone();
                        }
                    }
                }
            }
        }

        // Additional browsers that need user agent version (handled after name resolution)
        if let Some(client) = &client_from_ua {
            if !client.version.as_ref().unwrap_or(&String::new()).is_empty() {
                if BROWSERS_USING_UA_VERSION_FINAL.contains(&client_from_hints.name.as_str()) {
                    client_from_hints.version = client.version.clone();
                }
            }
        }
    };

    let mut res = client_from_hints.or(client_from_ua);

    // Special handling for Opera Mobile with WebView
    // If Chrome WebView is detected but the UA contains OPR/, it's actually Opera Mobile
    if let Some(client) = res.as_mut() {
        if client.name == "Chrome Webview" && ua.contains(" OPR/") {
            // Re-detect as Opera Mobile
            static OPERA_MOBILE_REGEX: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"Mobile.+OPR/(\d+[\.\d]+)").expect("valid opera mobile regex")
            });
            
            if let Some(captures) = OPERA_MOBILE_REGEX.captures(ua)? {
                if let Some(version_match) = captures.get(1) {
                    client.name = "Opera Mobile".to_owned();
                    client.version = Some(version_match.as_str().to_owned());
                    client.engine = Some("Blink".to_owned());
                    
                    // Extract Chrome/Blink engine version
                    static CHROME_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
                        Regex::new(r"Chrome/(\d+[\.\d]+)").expect("valid chrome version regex")
                    });
                    
                    if let Some(chrome_captures) = CHROME_VERSION_REGEX.captures(ua)? {
                        if let Some(chrome_version) = chrome_captures.get(1) {
                            client.engine_version = Some(chrome_version.as_str().to_owned());
                        }
                    }
                    
                    if let Some(browser) = AVAILABLE_BROWSERS.search_by_name("Opera Mobile") {
                        client.browser = Some(browser.to_owned());
                    }
                }
            }
        }
    }

    if let Some(client) = res.as_mut() {
        if let Some(client_hints) = client_hints {
            if let Some(app_hint) = &client_hints.app {
                if let Some(app_name) = super::hints::browsers::get_hint(app_hint)? {
                    if client.name != app_name {
                        client.name = app_name.to_owned();
                        
                        // Try to extract version from user agent for the app-based browser
                        client.version = extract_version_from_ua(ua, app_hint)?;

                        if let Some(browser) = AVAILABLE_BROWSERS.search_by_name(app_name) {
                            static BLINK_REGEX: Lazy<Regex> = Lazy::new(|| {
                                Regex::new(r"Chrome/.+ Safari/537.36").expect("valid blink regex")
                            });

                            // Some app-based browsers are always Blink-based
                            const ALWAYS_BLINK_APPS: &[&str] = &[
                                "TV-Browser Internet",
                                "XnBrowse",
                                "Open Browser Lite",
                            ];

                            if BLINK_REGEX.is_match(ua)? || ALWAYS_BLINK_APPS.contains(&app_name) {
                                client.engine = Some("Blink".to_owned());

                                if let Some(engine) = &client.engine {
                                    client.engine_version =
                                        BrowserClientList::engine_version(ua, engine)?;
                                }

                                // If ua_full_version from client hints is more detailed, use it
                                if let Some(ua_fv) = client_hints.ua_full_version.as_deref() {
                                    if let Some(ev) = &client.engine_version {
                                        if version_compare::compare(ua_fv, ev.as_str()) == Ok(version_compare::Cmp::Gt) {
                                            client.engine_version = Some(ua_fv.to_owned());
                                        }
                                    } else {
                                        client.engine_version = Some(ua_fv.to_owned());
                                    }
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

    // PHP logic: for Blink engines, if ua_full_version from hints is more detailed than current
    // engine version, upgrade it. This handles cases like Chrome Webview being overridden to a
    // specific app-detected browser (e.g., Aloha) where the engine version came from the reduced
    // UA string (Chrome/123.0.0.0) but ua_full_version has the full version (123.0.6312.118).
    if let Some(client) = res.as_mut() {
        if let Some(ch) = client_hints {
            if client.engine.as_deref() == Some("Blink") && client.name != "Iridium" {
                if let Some(ua_fv) = ch.ua_full_version.as_deref() {
                    match &client.engine_version {
                        Some(ev) if version_compare::compare(ua_fv, ev.as_str()) == Ok(Cmp::Gt) => {
                            client.engine_version = Some(ua_fv.to_owned());
                        }
                        None => {
                            client.engine_version = Some(ua_fv.to_owned());
                        }
                        _ => {}
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
            token = "(?:Chr[o0]me|Chromium|Cronet)";
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

        for (engine_version, eng) in &engine_versions {
            match version_compare::compare(version, engine_version) {
                Ok(Cmp::Eq) | Ok(Cmp::Gt) => {
                    engine = Some(eng.to_string());
                }
                _ => {}
            }
        }

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
