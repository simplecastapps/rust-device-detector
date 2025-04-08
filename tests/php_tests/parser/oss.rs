use anyhow::Result;
use serde_yaml::Value;

use crate::utils;
use rust_device_detector::client_hints::ClientHint;

#[tokio::test]
async fn test_oss() -> Result<()> {
    let files = utils::files("tests/data/fixtures/parser/oss.yml")?;

    assert!(!files.is_empty(), "expected at least one file");

    for file in files.into_iter() {
        let mut cases: Value = serde_yaml::from_reader(file)?;
        let cases = cases.as_sequence_mut().expect("sequence");

        for (i, case) in cases.into_iter().enumerate() {
            basic(i + 1, case).await.expect("basic test");
        }
    }

    Ok(())
}

async fn basic(idx: usize, value: &mut Value) -> Result<()> {
    let ua = value["user_agent"].as_str().expect("user_agent");
    let test_os = value["os"].as_mapping().expect("os");
    let dd = &utils::DD;

    let client_hints: Option<ClientHint> = value
        .get("headers")
        .and_then(|headers| headers.as_mapping())
        .and_then(|headers| utils::client_hint_mock(headers).ok());

    let dd_res = dd.parse_client_hints(ua, client_hints).await?;

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
