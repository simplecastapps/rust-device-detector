use anyhow::Result;

use once_cell::sync::Lazy;

use crate::parsers::utils::SafeRegex as Regex;

pub struct ClientHintMapping {
    mappings: Vec<(String, Vec<String>)>,
}

impl ClientHintMapping {
    pub fn new(mappings: Vec<(String, Vec<String>)>) -> Self {
        let mappings = mappings
            .into_iter()
            .map(|(k, v)| {
                let oss = v.into_iter().collect();
                (k, oss)
            })
            .collect::<Vec<(String, Vec<String>)>>();

        ClientHintMapping { mappings }
    }

    pub fn apply(&self, name: &str) -> Result<String> {
        let res = self
            .mappings
            .iter()
            .find(|(_k, vs)| {
                // println!("k: {:?}, v: {:?}", k, v);
                vs.iter().any(|x| x.to_lowercase() == name.to_lowercase())
            })
            .map(|x| x.0.clone());

        Ok(res.unwrap_or(name.to_owned()))
    }
}

// TODO options?
#[derive(Debug)]
pub struct ClientHint {
    pub architecture: Option<String>,
    pub bitness: Option<String>,
    pub mobile: bool,
    pub model: Option<String>,
    pub ua_full_version: Option<String>,
    pub platform: Option<String>,
    pub platform_version: Option<String>,
    pub full_version_list: Vec<(String, String)>,
    pub app: Option<String>,
    pub form_factors: Vec<String>,
}

impl ClientHint {
    pub fn from_headers(headers: Vec<(String, String)>) -> Result<ClientHint> {
        let mut architecture = None;
        let mut bitness = None;
        let mut mobile = false;
        let mut model = None;
        let mut ua_full_version = None;
        let mut platform = None;
        let mut platform_version = None;
        let mut app = None;
        let mut form_factors: Vec<String> = Vec::new();
        let mut full_version_list: Vec<(String, String)> = Vec::new();

        static BRAND_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#""([^"]+)"; ?v="([^"]+)"?"#).unwrap());

        static FORM_FACTOR_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"~"([a-z]+)"~i"#).unwrap());

        // println!("headers: {:?}", headers);
        for (header, value) in headers {
            let header = header.replace('_', "-").to_lowercase();
            match header.trim() {
                "sec-ch-ua-arch" => {
                    architecture = Some(value.trim_matches('"').to_owned());
                }

                "sec-ch-ua-bitness" => {
                    bitness = Some(value.trim_matches('"').to_owned());
                }

                "sec-ch-ua-mobile" => {
                    // the php version interspersed actual headers and mock variable
                    // names in its code, so I don't know which of these values actually
                    // could come up in real user agents.
                    if value == "1" || value == "true" || value == "yes" || value == "?1" {
                        mobile = true;
                    }
                }

                "sec-ch-ua-model" => {
                    let value = value.trim_matches('"');
                    if !value.is_empty() {
                        model = Some(value.to_owned());
                    }
                }

                "sec-ch-ua-platform" => {
                    platform = Some(value.trim_matches('"').to_owned());
                }

                "sec-ch-ua-platform-version" => {
                    platform_version = Some(value.trim_matches('"').to_owned());
                    // TODO remove blanks from other values and see if tests pass.
                    if platform_version.as_deref() == Some("") {
                        platform_version = None;
                    }
                }

                "x-requested-with" => {
                    if value != "xmlhttprequest" {
                        app = Some(value.to_owned());
                    }
                }

                "sec-ch-ua-full-version" => {
                    ua_full_version = Some(value.trim_matches('"').to_owned());
                }

                "sec-ch-ua" => {
                    if full_version_list.is_empty() {
                        for x in BRAND_REGEX.captures_iter(&value) {
                            let res = x?;
                            let brand = res.get(1).map(|x| x.as_str()).unwrap_or_else(|| "");
                            let brand_version =
                                res.get(2).map(|x| x.as_str()).unwrap_or_else(|| "");
                            full_version_list.push((brand.to_owned(), brand_version.to_owned()));
                        }
                    }
                }

                "sec-ch-ua-full-version-list" => {
                    full_version_list.clear();

                    for x in BRAND_REGEX.captures_iter(&value) {
                        let res = x?;
                        let brand = res.get(1).map(|x| x.as_str()).unwrap_or_else(|| "");
                        let brand_version = res.get(2).map(|x| x.as_str()).unwrap_or_else(|| "");
                        full_version_list.push((brand.to_owned(), brand_version.to_owned()));
                    }
                }

                "sec-ch-ua-form-factors" => {
                    form_factors = FORM_FACTOR_REGEX
                        .captures_iter(&value)
                        .filter_map(|x| x.ok()?.get(1).map(|x| x.as_str().to_lowercase()))
                        .collect();
                }

                _ => {}
            }
        }

        let res = ClientHint {
            architecture,
            bitness,
            mobile,
            model,
            ua_full_version,
            platform,
            platform_version,
            full_version_list,
            app,
            form_factors,
        };

        // println!("client hints: {:?}", res);

        Ok(res)
    }
}
