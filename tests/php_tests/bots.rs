use anyhow::Result;
use serde_yaml::Value;

use crate::utils;

pub(crate) fn basic(idx: usize, value: &Value) -> Result<()> {
    let ua = value["user_agent"].as_str().expect("user_agent");
    let test_bot = value["bot"].as_mapping().expect("bot");
    let dd = &utils::DD;
    let dd_res = dd.parse(ua, None)?;

    assert!(
        dd_res.is_bot(),
        "expected bot test case: {}\n ua: {}",
        idx,
        ua
    );

    let dd_bot_name: Option<&str> = dd_res.get_bot().map(|bot| bot.name.as_ref());
    let test_bot_name: Option<&str> = test_bot["name"].as_str();

    assert!(
        (test_bot_name == dd_bot_name) || (test_bot_name == Some("") && dd_bot_name.is_none()),
        "bot name test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_bot_name,
        test_bot_name,
        ua
    );

    let dd_bot_category: Option<&str> = dd_res.get_bot().and_then(|bot| bot.category.as_deref());
    let test_bot_category: Option<&str> = test_bot.get("category").and_then(|cat| cat.as_str());

    assert!(
        (test_bot_category == dd_bot_category)
            || (test_bot_category == Some("") && dd_bot_category.is_none()),
        "bot category test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_bot_category,
        test_bot_category,
        ua
    );

    let dd_bot_url: Option<&str> = dd_res.get_bot().and_then(|bot| bot.url.as_deref());
    let test_bot_url: Option<&str> = test_bot.get("url").and_then(|url| url.as_str());

    assert!(
        (test_bot_url == dd_bot_url) || (test_bot_url == Some("") && dd_bot_url.is_none()),
        "bot url test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_bot_url,
        test_bot_url,
        ua
    );

    let dd_bot_producer: Option<_> = dd_res.get_bot().and_then(|bot| bot.producer.as_ref());

    let dd_bot_producer_name: Option<&str> = dd_bot_producer.and_then(|prod| prod.name.as_deref());

    let test_bot_producer_name: Option<&str> = test_bot
        .get("producer")
        .and_then(|prod| prod.as_mapping())
        .and_then(|prod| prod["name"].as_str());

    let test_bot_producer_name = if test_bot_producer_name == Some("") {
        None
    } else {
        test_bot_producer_name
    };

    assert!(
        test_bot_producer_name == dd_bot_producer_name,
        "bot producer name test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_bot_producer_name,
        test_bot_producer_name,
        ua
    );

    let dd_bot_producer_url: Option<&str> = dd_bot_producer.and_then(|prod| prod.url.as_deref());

    let test_bot_url: Option<&str> = test_bot
        .get("producer")
        .and_then(|prod| prod.as_mapping())
        .and_then(|prod| prod.get("url"))
        .and_then(|url| url.as_str());

    assert!(
        (test_bot_url == dd_bot_producer_url
            || (test_bot_url == Some("") && dd_bot_producer_url.is_none()))
            || (test_bot_url == Some("") && dd_bot_producer_url.is_none()),
        "bot producer url test case: {}\n code: {:?}\n test: {:?}\n ua: {}",
        idx,
        dd_bot_producer_url,
        test_bot_url,
        ua
    );

    Ok(())
}
