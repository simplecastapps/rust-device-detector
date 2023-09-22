use anyhow::Result;
use serde_yaml::Value;

use crate::utils;

#[test]
fn test_parser_consoles() -> Result<()> {
    let files = utils::files("tests/data/fixtures/parser/device/console.yml")?;

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
    let test_device = value["device"].as_mapping().expect("device");
    let dd = &utils::DD;
    let dd_res = dd.parse(ua, None)?;

    assert!(!dd_res.is_bot(), "expected not bot");

    let dd_type: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.device.as_ref())
        .and_then(|dev| dev.device_type.as_ref())
        .map(|t| t.as_str());
    let test_type: Option<&str> = test_device["type"].as_str();

    assert!(
        (test_type == dd_type) || (test_type == Some("") && dd_type.is_none()),
        "device type test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_type,
        test_type,
        ua
    );

    let dd_brand: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.device.as_ref())
        .and_then(|device| device.brand.as_ref())
        .map(|b| b.as_str());
    let test_brand: Option<&str> = test_device["brand"].as_str();

    assert!(
        (test_brand == dd_brand) || (test_brand == Some("") && dd_brand.is_none()),
        "device brand test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_brand,
        test_brand,
        ua
    );

    let dd_model: Option<&str> = dd_res
        .get_known_device()
        .and_then(|dev| dev.device.as_ref())
        .and_then(|device| device.model.as_ref())
        .map(|m| m.as_str());

    let test_model: Option<&str> = test_device["model"].as_str();

    assert!(
        (test_model == dd_model) || (test_model == Some("") && dd_model.is_none()),
        "device model test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_model,
        test_model,
        ua
    );

    Ok(())
}
