use anyhow::Result;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use serde_yaml::Mapping;

use once_cell::sync::Lazy;

use rust_device_detector::client_hints::ClientHint;
use rust_device_detector::device_detector::DeviceDetector;

pub(crate) static DD: Lazy<DeviceDetector> = Lazy::new(|| DeviceDetector::new());

// use stats_alloc::{Stats, INSTRUMENTED_SYSTEM};
// pub fn memory_test(f: &dyn Fn() -> Result<()>) -> Result<Stats> {
//     let reg = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);
//     f()?;
//
//     // difference in memory before and after function runs.
//     let ch = reg.change();
//
//     Ok(ch)
// }

pub fn file_paths(path: &str) -> Result<Vec<PathBuf>> {
    let files = glob::glob(path)
        .expect("text fixtures")
        .map(|x| x.expect("glob"))
        .collect::<Vec<_>>();

    Ok(files)
}

pub fn files(path: &str) -> Result<Vec<BufReader<File>>> {
    let files = file_paths(path)?
        .into_iter()
        .map(|x| BufReader::new(File::open(x).expect("file")))
        .collect::<Vec<_>>();

    Ok(files)
}

// upstream has begun sometimes mocking their client hints rather than using actual headers
// found in the wild, so for those tests to work for us we must allow mocking using their
// variable names.
static MOCK_HEADERS: [&str; 11] = [
    "arch",
    "architecture",
    "bitness",
    "brands",
    "fullVersionList",
    "mobile",
    "model",
    "platform",
    "platformVersion",
    "uaFullVersion",
    "wow64",
];

pub fn client_hint_mock(fields: &Mapping) -> Result<ClientHint> {
    let mut normal_fields = Vec::new();

    for (key, value) in fields {
        let key = key.as_str().expect("header name or mock field name");
        if !MOCK_HEADERS.iter().any(|x| *x == key) {
            normal_fields.push((
                // In php their client hints allows headers to be prefixed with HTTP_ or HTTP-.
                // as that is a base behavior of php header detection. But we don't need to do
                // that outside of tests.
                key.trim_start_matches("http-").to_owned(),
                value.as_str().expect("header value").to_owned(),
            ));
        }
    }

    let mut client_hints = ClientHint::from_headers(normal_fields)?;

    for (key, value) in fields {
        let key = key.as_str().expect("header name or mock field name");
        if MOCK_HEADERS.iter().any(|x| *x == key) {
            match key {
                "arch" | "architecture" => {
                    client_hints.architecture =
                        Some(value.as_str().expect("arch").trim_matches('"').to_owned());
                }
                "bitness" => {
                    client_hints.bitness = Some(
                        value
                            .as_str()
                            .expect("bitness")
                            .trim_matches('"')
                            .to_owned(),
                    );
                }
                "uaFullVersion" => {
                    client_hints.ua_full_version =
                        Some(value.as_str().expect("uaFullVersion").to_owned());
                }
                "platform" => {
                    client_hints.platform = Some(
                        value
                            .as_str()
                            .expect("platform")
                            .trim_matches('"')
                            .to_owned(),
                    );
                }
                "platformVersion" => {
                    client_hints.platform_version = Some(
                        value
                            .as_str()
                            .expect("platformVersion")
                            .trim_matches('"')
                            .to_owned(),
                    );
                }
                "model" => {
                    client_hints.model =
                        Some(value.as_str().expect("model").trim_matches('"').to_owned());
                }
                "mobile" => {
                    if value.is_bool() {
                        client_hints.mobile = value.as_bool().expect("mobile");
                    } else {
                        let res: &str = value.as_str().expect("mobile field as a string");
                        if res == "1" {
                            client_hints.mobile = true;
                        } else {
                            client_hints.mobile = false;
                        }
                    }
                }

                "fullVersionList" | "brands" => {
                    if key == "brands" && !client_hints.full_version_list.is_empty() {
                        continue;
                    }

                    let brands = value.as_sequence().expect("fullVersionList as a sequence");
                    for brand in brands {
                        let brand = brand.as_mapping().expect("brand as a mapping");
                        let mut brand_name = None;
                        let mut brand_version = None;

                        for (k, v) in brand {
                            let k = k.as_str().expect("brand name");
                            let v = v.as_str().expect("brand version");
                            match k {
                                "brand" => {
                                    brand_name = Some(v.to_owned());
                                }
                                "version" => {
                                    brand_version = Some(v.to_owned());
                                }
                                _ => (),
                            }
                        }

                        client_hints.full_version_list.push((
                            brand_name.expect("brand name"),
                            brand_version.expect("brand version"),
                        ));
                    }
                }

                // this isn't even used.
                "wow64" => (),
                _ => (),
            }
        }
    }

    Ok(client_hints)
}
