use anyhow::Result;
use fancy_regex::Regex;

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
}

impl ClientHint {
    pub(crate) fn from_headers(headers: Vec<(String, String)>) -> Result<ClientHint> {
        let mut architecture = None;
        let mut bitness = None;
        let mut mobile = false;
        let mut model = None;
        let mut ua_full_version = None;
        let mut platform = None;
        let mut platform_version = None;
        let mut app = None;

        let mut full_version_list: Vec<(String, String)> = Vec::new();

        lazy_static::lazy_static! {
            static ref BRAND_REGEX: Regex = Regex::new(r#""([^"]+)"; ?v="([^"]+)"?"#).unwrap();
        }

        // println!("headers: {:?}", headers);
        for (header, value) in headers {
            let header = header.replace('_', "-").to_lowercase();
            match header.trim() {
                "http-sec-ch-ua-arch" | "sec-ch-ua-arch" | "arch" | "architecture" => {
                    architecture = Some(value.trim_matches('"').to_owned());
                }

                "http-sec-ch-ua-bitness" | "sec-ch-ua-bitness" | "bitness" => {
                    bitness = Some(value.trim_matches('"').to_owned());
                }

                "http-sec-ch-ua-mobile" | "sec-ch-ua-mobile" | "mobile" => {
                    if value == "1" || value == "true" || value == "yes" || value == "?1" {
                        mobile = true;
                    }
                }

                "http-sec-ch-ua-model" | "sec-ch-ua-model" | "model" => {
                    let value = value.trim_matches('"');
                    if value != "" {
                        model = Some(value.to_owned());
                    }
                }

                "http-sec-ch-ua-platform" | "sec-ch-ua-platform" | "platform" => {
                    platform = Some(value.trim_matches('"').to_owned());
                }

                "http-sec-ch-ua-platform-version"
                | "sec-ch-ua-platform-version"
                | "platformversion" => {
                    platform_version = Some(value.trim_matches('"').to_owned());
                }

                "http-x-requested-with" | "x-requested-with" => {
                    if value != "xmlhttprequest" {
                        app = Some(value.to_owned());
                    }
                }

                "http-sec-ch-ua-full-version" | "sec-ch-ua-full-version" => {
                    ua_full_version = Some(value.trim_matches('"').to_owned());
                }

                // skipping "brands" / "fullVersionList" as I cannot find any examples of this
                // header or its format
                "http-sec-ch-ua" | "sec-ch-ua" => {
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

                // use this only if no other header already set the list
                "http-sec-ch-ua-full-version-list" | "sec-ch-ua-full-version-list" => {
                    full_version_list.clear();

                    for x in BRAND_REGEX.captures_iter(&value) {
                        let res = x?;
                        let brand = res.get(1).map(|x| x.as_str()).unwrap_or_else(|| "");
                        let brand_version = res.get(2).map(|x| x.as_str()).unwrap_or_else(|| "");
                        full_version_list.push((brand.to_owned(), brand_version.to_owned()));
                    }
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
        };

        // println!("client hints: {:?}", res);

        Ok(res)
    }
}
