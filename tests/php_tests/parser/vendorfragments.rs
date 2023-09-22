use anyhow::Result;
use serde_yaml::Value;

use crate::utils;

#[test]
fn test_vendorfragments() -> Result<()> {
    let files = utils::files("tests/data/fixtures/parser/vendorfragments.yml")?;

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
    let ua = value["useragent"].as_str().expect("user_agent");
    let test_vendor = value["vendor"].as_str().expect("vendor");
    let dd = &utils::DD;
    let dd_res = dd.parse(ua, None)?;

    let dd_brand: &str = dd_res
        .get_known_device()
        .and_then(|dev| dev.device.as_ref())
        .and_then(|dev| dev.brand.as_ref())
        .map(|x| x.as_str())
        .expect("device brand");

    assert!(
        test_vendor == dd_brand,
        "vendor test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_brand,
        test_vendor,
        ua
    );

    Ok(())
}
