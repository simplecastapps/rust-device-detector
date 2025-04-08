use anyhow::Result;
use serde_yaml::Value;

use crate::utils;
use rust_device_detector::client_hints::ClientHint;

use rust_device_detector::device_detector::Detection;

use test_each_file::test_each_file;

test_each_file! { in "./tests/data/fixtures/" => base_fixture_tests }

// #[test]
//#[allow(dead_code)]
//fn repeat_memory_tests() -> Result<()> {
//    use stats_alloc::INSTRUMENTED_SYSTEM;
//    let reg = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);
//
//    for r in [1..10, 1..10, 1..12, 1..12, 5..13, 1..13] {
//        println!("range {:?}", r);
//
//        // this will cause allocations (500M or so)
//        let mch = utils::memory_test(&|| {
//            for i in r.clone().into_iter() {
//                test_some_files(i)?;
//            }
//            Ok(())
//        })?;
//
//        let ch = reg.change();
//
//        println!("run: stats {:?}", &mch);
//
//        println!("global: stats {:?}", &ch);
//    }
//
//    Ok(())
//}
//
async fn base_fixture_tests(file_path: &str, contents: &str) {
    let mut cases: Value = serde_yaml::from_str(contents).expect("valid test yaml");

    let cases = cases.as_sequence_mut().expect("sequence");

    for (i, case) in cases.into_iter().enumerate() {
        basic(file_path, i + 1, case).await.expect("basic test");
    }
}

async fn basic(test_file: &str, idx: usize, value: &Value) -> Result<()> {
    if value
        .as_mapping()
        .map(|m| m.contains_key("bot"))
        .unwrap_or(false)
    {
        crate::bots::basic(idx, value).await?;
    } else {
        basic_known(test_file, idx, value).await?;
    }

    Ok(())
}

async fn basic_known(file_path: &str, idx: usize, value: &Value) -> Result<()> {
    let dd = &utils::DD;

    let ua = value["user_agent"]
        .as_str()
        .unwrap_or_else(|| panic!("missing user_agent, file: {}, case: {}", file_path, idx));

    let client_hints: Option<ClientHint> = value
        .get("headers")
        .and_then(|headers| headers.as_mapping())
        .and_then(|headers| utils::client_hint_mock(headers).ok());

    let dd_res = dd.parse_client_hints(ua, client_hints).await?;
    assert!(!dd_res.is_bot(), "should not be a bot");

    basic_os(file_path, idx, ua, value, &dd_res)?;
    basic_client(file_path, idx, ua, value, &dd_res)?;
    basic_device(file_path, idx, ua, value, &dd_res)?;

    Ok(())
}

fn basic_client(
    file_path: &str,
    idx: usize,
    ua: &str,
    value: &Value,
    dd_res: &Detection,
) -> Result<()> {
    let test_client = &value["client"];

    let dd_res = dd_res.get_known_device().expect("known device");

    if !test_client.is_mapping() {
        assert!(
            dd_res.client.is_none(),
            "client non null file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
            file_path,
            idx,
            dd_res.client,
            test_client,
            ua
        );
        return Ok(());
    }

    assert!(!dd_res.is_bot());

    let dd_client_type: Option<&str> = dd_res.client.as_ref().map(|client| client.r#type.as_str());

    let test_client_type: Option<&str> = test_client["type"].as_str();

    assert!(
        test_client_type == dd_client_type,
        "client type test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_client_type,
        test_client_type,
        ua
    );

    let dd_name: Option<&str> = dd_res.client.as_ref().map(|client| client.name.as_ref());
    let test_name: Option<&str> = test_client["name"].as_str();

    assert!(
        test_name == dd_name,
        "client name test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_name,
        test_name,
        ua
    );

    let dd_version: Option<&str> = dd_res
        .client
        .as_ref()
        .and_then(|client| client.version.as_ref())
        .map(|version| version.as_str());
    let test_version: Option<&str> = test_client["version"].as_str();
    let test_version = if test_version == Some("") {
        None
    } else {
        test_version
    };

    assert!(
        test_version == dd_version,
        "client version test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_version,
        test_version,
        ua
    );

    let dd_engine: Option<&str> = dd_res
        .client
        .as_ref()
        .and_then(|client| client.engine.as_deref());

    let test_engine: Option<&str> = test_client.get("engine").and_then(|engine| engine.as_str());
    let test_engine = if test_engine == Some("") {
        None
    } else {
        test_engine
    };

    assert!(
        test_engine == dd_engine,
        "client engine test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_engine,
        test_engine,
        ua
    );

    let test_engine_version: Option<&str> = test_client
        .get("engine_version")
        .and_then(|engine| engine.as_str());
    let test_engine_version = if test_engine_version == Some("") {
        None
    } else {
        test_engine_version
    };

    assert!(
        test_engine_version
            == dd_res
                .client
                .as_ref()
                .and_then(|client| client.engine_version.as_deref()),
        "client engine version test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_res
            .client
            .as_ref()
            .and_then(|client| client.engine_version.as_deref()),
        test_engine_version,
        ua
    );

    Ok(())
}

fn basic_device(
    file_path: &str,
    idx: usize,
    ua: &str,
    value: &Value,
    dd_res: &Detection,
) -> Result<()> {
    let test_device = &value["device"];

    let dd_res = dd_res.get_known_device().expect("known device");

    if !test_device.is_mapping() {
        assert!(
            dd_res.client.is_none(),
            "client non null file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
            file_path,
            idx,
            dd_res.client,
            test_device,
            ua
        );
        return Ok(());
    }

    let dd_type: Option<&str> = dd_res
        .device
        .as_ref()
        .and_then(|device| device.device_type.as_ref())
        .map(|device_type| device_type.as_str());
    let dd_type = if dd_type == Some("") { None } else { dd_type };

    let test_type: Option<&str> = test_device["type"].as_str();
    let test_type = if test_type == Some("") {
        None
    } else {
        test_type
    };

    assert!(
        test_type == dd_type,
        "device type test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_type,
        test_type,
        ua
    );

    let dd_brand: Option<&str> = dd_res
        .device
        .as_ref()
        .and_then(|device| device.brand.as_deref());

    let test_brand: Option<&str> = test_device.get("brand").and_then(|brand| brand.as_str());
    let test_brand = if test_brand == Some("") {
        None
    } else {
        test_brand
    };

    assert!(
        test_brand == dd_brand,
        "device brand test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_brand,
        test_brand,
        ua
    );

    let dd_model: Option<&str> = dd_res
        .device
        .as_ref()
        .and_then(|device| device.model.as_deref());

    let test_model: Option<&str> = test_device.get("model").and_then(|model| model.as_str());
    let test_model = if test_model == Some("") {
        None
    } else {
        test_model
    };

    assert!(
        test_model == dd_model,
        "device model test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_model,
        test_model,
        ua
    );

    Ok(())
}

fn basic_os(
    file_path: &str,
    idx: usize,
    ua: &str,
    value: &Value,
    dd_res: &Detection,
) -> Result<()> {
    let test_os = &value["os"];

    let dd_res = dd_res.get_known_device().expect("known device");

    if !test_os.is_mapping() {
        assert!(
            dd_res.os.is_none(),
            "os non null file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
            file_path,
            idx,
            dd_res.client,
            test_os,
            ua
        );
        return Ok(());
    }

    let dd_name: Option<&str> = dd_res.os.as_ref().map(|os| os.name.as_ref());

    let test_name: Option<&str> = test_os["name"].as_str();

    assert!(
        test_name == dd_name,
        "os name test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_name,
        test_name,
        ua
    );

    let dd_version: Option<&str> = dd_res
        .os
        .as_ref()
        .and_then(|os| os.version.as_ref())
        .map(|version| version.as_str());
    let dd_version = if dd_version == Some("") {
        None
    } else {
        dd_version
    };

    let test_version: Option<&str> = test_os["version"].as_str();
    let test_version = if test_version == Some("") {
        None
    } else {
        test_version
    };

    assert!(
        test_version == dd_version,
        "os version test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_version,
        test_version,
        ua
    );

    let dd_platform: Option<&str> = dd_res
        .os
        .as_ref()
        .and_then(|os| os.platform.as_ref())
        .map(|platform| platform.as_str());

    let test_platform: Option<&str> = test_os["platform"].as_str();
    let test_platform = if test_platform == Some("") {
        None
    } else {
        test_platform
    };

    assert!(
        test_platform == dd_platform,
        "os platform test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_platform,
        test_platform,
        ua
    );

    let dd_family = dd_res
        .os
        .as_ref()
        .and_then(|os| os.family.as_ref())
        .map(|family| family.as_str());

    let test_family: Option<&str> = value["os_family"].as_str();
    let test_family = if test_family == Some("") || test_family == Some("Unknown") {
        None
    } else {
        test_family
    };

    assert!(
        test_family == dd_family,
        "os family test file: {}, case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        file_path,
        idx,
        dd_family,
        test_family,
        ua
    );

    Ok(())
}
