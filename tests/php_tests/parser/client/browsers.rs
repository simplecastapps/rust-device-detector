use anyhow::Result;
use serde_yaml::Value;

use crate::utils;
use std::fs::File;
use std::io::BufReader;

use rust_device_detector::client_hints::ClientHint;

#[tokio::test]
async fn test_parser_browsers() -> Result<()> {
    let files: Vec<BufReader<File>> = vec![BufReader::new(
        File::open("tests/data/fixtures/parser/client/browser.yml").expect("file"),
    )];

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
    let test_client = value["client"].as_mapping().expect("client");
    let dd = &utils::DD;

    let client_hints: Option<ClientHint> = value
        .get("headers")
        .and_then(|headers| headers.as_mapping())
        .and_then(|headers| utils::client_hint_mock(headers).ok());

    let dd_res = dd.parse_client_hints(ua, client_hints).await?;

    assert!(!dd_res.is_bot());

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

    let dd_browser_name: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .map(|client| client.name.as_ref());
    let test_browser_name: Option<&str> = test_client["name"].as_str();

    assert!(
        test_browser_name == dd_browser_name,
        "browser name test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_browser_name,
        test_browser_name,
        ua
    );

    let dd_browser_version: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .and_then(|client| client.version.as_ref())
        .map(|version| version.as_str());
    let test_browser_version: Option<&str> = test_client["version"].as_str();
    let test_browser_version = if test_browser_version == Some("") {
        None
    } else {
        test_browser_version
    };

    assert!(
        test_browser_version == dd_browser_version,
        "browser version test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_browser_version,
        test_browser_version,
        ua
    );

    let dd_browser_engine: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .and_then(|client| client.engine.as_deref());
    let test_browser_engine: Option<&str> =
        test_client.get("engine").and_then(|engine| engine.as_str());

    assert!(
        test_browser_engine == dd_browser_engine
            || test_browser_engine == Some("") && dd_browser_engine.is_none(),
        "browser engine test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_browser_engine,
        test_browser_engine,
        ua
    );

    let dd_browser_engine_version: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.client.as_ref())
        .and_then(|client| client.engine_version.as_deref());

    let test_browser_engine_version: Option<&str> = test_client
        .get("engine_version")
        .and_then(|engine_version| engine_version.as_str());

    assert!(
        test_browser_engine_version == dd_browser_engine_version
            || test_browser_engine_version == Some("") && dd_browser_engine_version.is_none(),
        "browser engine version test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_browser_engine_version,
        test_browser_engine_version,
        ua
    );

    Ok(())
}
