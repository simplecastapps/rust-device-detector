use anyhow::Result;
use serde_yaml::Value;

use crate::utils;

#[test]
fn test_type_methods() -> Result<()> {
    let files = utils::files("tests/data/fixtures/parser/type-methods.yml")?;

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
    let test_checks = value["check"].as_sequence().expect("checks");

    let dd = &utils::DD;
    let dd_res = dd.parse(ua, None)?;

    let test_bot = test_checks[0].as_bool().expect("bot");
    let dd_bot = dd_res.is_bot();

    assert!(
        test_bot == dd_bot,
        "bot test case: {}\n code: {:?}\n test: {:?} ua: {}",
        idx,
        dd_bot,
        test_bot,
        ua
    );

    if !dd_res.is_bot() {
        let known_device = dd_res.get_known_device().expect("known_device");

        let test_mobile = test_checks[1].as_bool().expect("mobile");
        let dd_mobile = known_device.is_mobile();

        assert!(
            test_mobile == dd_mobile,
            "mobile test case: {}\n code: {:?}\n test: {:?} ua: {}",
            idx,
            dd_mobile,
            test_mobile,
            ua
        );

        let test_desktop = test_checks[2].as_bool().expect("desktop");
        let dd_desktop = known_device.is_desktop();

        assert!(
            test_desktop == dd_desktop,
            "desktop test case: {}\n code: {:?}\n test: {:?} ua: {}",
            idx,
            dd_desktop,
            test_desktop,
            ua
        );

        let test_tablet = test_checks[3].as_bool().expect("tablet");
        let dd_tablet = known_device.is_tablet();

        assert!(
            test_tablet == dd_tablet,
            "tablet test case: {}\n code: {:?}\n test: {:?} ua: {}",
            idx,
            dd_tablet,
            test_tablet,
            ua
        );

        let test_tv = test_checks[4].as_bool().expect("tv");
        let dd_tv = known_device.is_television();

        assert!(
            test_tv == dd_tv,
            "tv test case: {}\n code: {:?}\n test: {:?} ua: {}",
            idx,
            dd_tv,
            test_tv,
            ua
        );

        let test_wearable = test_checks[5].as_bool().expect("wearable");
        let dd_wearable = known_device.is_wearable();

        assert!(
            test_wearable == dd_wearable,
            "wearable test case: {}\n code: {:?}\n test: {:?} ua: {}",
            idx,
            dd_wearable,
            test_wearable,
            ua
        );
    }

    Ok(())
}
