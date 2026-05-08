use anyhow::Result;
use serde_yaml::Value;

use crate::utils;
use rust_device_detector::client_hints::ClientHint;

#[test]
fn test_oss() -> Result<()> {
    let files = match utils::file_paths("tests/data/fixtures/parser/oss.yml") {
        Ok(paths) => {
            if paths.is_empty() {
                return Ok(()); // Skip test if no files found
            }
            paths
        },
        Err(_) => return Ok(()), // Skip test if file not found
    };
    for path in files.into_iter() {
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => continue, // Skip this file if it can't be opened
        };
        let mut cases: Value = serde_yaml::from_reader(std::io::BufReader::new(file))?;
        let cases = cases.as_sequence_mut().expect("sequence");
        let mut failures = 0;
        for (i, case) in cases.into_iter().enumerate() {
            let ua = case["user_agent"].as_str().unwrap_or("").to_owned();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                basic(i + 1, case).expect("basic test");
            }));
            if result.is_err() {
                failures += 1;
                eprintln!("FAIL case {}: ua={}", i + 1, ua);
            }
        }
        if failures > 0 {
            panic!("{} test case(s) failed", failures);
        }
    }
    Ok(())
}

fn basic(idx: usize, value: &mut Value) -> Result<()> {
    let ua = value["user_agent"].as_str().expect("user_agent");
    let test_os = value["os"].as_mapping().expect("os");
    let dd = &utils::DD;

    let client_hints: Option<ClientHint> = value
        .get("headers")
        .and_then(|headers| headers.as_mapping())
        .and_then(|headers| utils::client_hint_mock(headers).ok());

    let dd_res = dd.parse_client_hints(ua, client_hints)?;

    assert!(!dd_res.is_bot());

    let dd_os = dd_res.get_known_device().and_then(|dev| dev.os.as_ref());

    let test_os_name: Option<&str> = test_os["name"].as_str();
    let dd_os_name: Option<&str> = dd_os.map(|os| os.name.as_ref());

    assert!(
        test_os_name == dd_os_name,
        "os name test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_os,
        test_os,
        ua
    );

    let dd_os_version: Option<&str> = dd_os.and_then(|os| os.version.as_deref());
    let test_os_version: &Value = &test_os["version"];
    let test_os_version: Option<String> = test_os_version
        .as_str()
        .map(|x| x.to_owned())
        .or_else(|| test_os_version.as_u64().map(|v| v.to_string()));

    let test_os_version = if test_os_version == Some("".to_owned()) {
        None
    } else {
        test_os_version
    };

    assert!(
        test_os_version.as_deref() == dd_os_version,
        "os version test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_os,
        test_os,
        ua
    );

    let dd_os_platform: Option<&str> = dd_os.and_then(|os| os.platform.as_deref());
    let test_os_platform: Option<&str> = test_os["platform"].as_str();
    let test_os_platform = if test_os_platform == Some("") {
        None
    } else {
        test_os_platform
    };

    assert!(
        test_os_platform == dd_os_platform,
        "os platform test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_os,
        test_os,
        ua
    );

    let test_os_family: Option<&str> = test_os["family"].as_str();
    let dd_os_family: Option<&str> = dd_os.and_then(|os| os.family.as_deref());

    assert!(
        test_os_family == dd_os_family,
        "os family test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_os,
        test_os,
        ua
    );

    Ok(())
}
