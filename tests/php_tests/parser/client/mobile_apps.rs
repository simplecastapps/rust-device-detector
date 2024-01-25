use anyhow::Result;
use serde_yaml::Value;

use crate::utils;

#[test]
fn test_parser_mobile_apps() -> Result<()> {
    let files = utils::files("tests/data/fixtures/parser/client/mobile_app.yml")?;

    assert!(!files.is_empty(), "expected at least one file");

    for file in files.into_iter() {
        let mut cases: Value = serde_yaml::from_reader(file)?;
        let cases = cases.as_sequence_mut().expect("sequence");

        for (i, case) in cases.into_iter().enumerate() {
            basic(i + 1, case).expect("basic test");
        }
    }

    Ok(())
}

fn basic(idx: usize, value: &mut Value) -> Result<()> {
    let ua = value["user_agent"].as_str().expect("user_agent");
    let test_client = value["client"].as_mapping().expect("client");
    let dd = &utils::DD;
    let dd_res = dd.parse(ua, None)?;

    assert!(!dd_res.is_bot(), "not a bot test case: {}\n ua: {}", idx, ua);

    let dd_client_type: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .map(|client| client.r#type.as_str());

    let test_client_type: Option<&str> = test_client["type"].as_str();

    assert!(
        test_client_type == dd_client_type,
        "client type test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_client_type,
        test_client_type,
        ua
    );

    let dd_name: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .map(|client| client.name.as_ref());
    let test_name: Option<&str> = test_client["name"].as_str();

    assert!(
        test_name == dd_name,
        "client name test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_name,
        test_name,
        ua
    );

    let dd_version: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .and_then(|client| client.version.as_deref());

    let test_version: Option<&str> = test_client["version"].as_str();
    let test_version = if test_version == Some("") {
        None
    } else {
        test_version
    };

    assert!(
        test_version == dd_version,
        "client version test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_version,
        test_version,
        ua
    );

    Ok(())
}
