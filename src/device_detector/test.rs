use anyhow::Result;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;

use serde::de::{value::MapAccessDeserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum Detection {
    KnownDevice(KnownDevice),
    Bot(BotEntry),
}

impl Detection {
    fn get_bot(&self) -> &Bot {
        match self {
            Detection::KnownDevice(_) => panic!("not a bot"),
            Detection::Bot(bot) => &bot.bot,
        }
    }

    fn get_device(&self) -> &KnownDevice {
        match self {
            Detection::KnownDevice(device) => device,
            Detection::Bot(_) => panic!("not a device"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct KnownDevice {
    pub client: Option<Client>,
    #[serde(deserialize_with = "decode_array_or_map")]
    pub os: OS,
    pub device: Device,
    pub os_family: String,
    pub browser_family: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Client {
    pub r#type: String,
    pub name: String,
    #[serde(deserialize_with = "decode_string_or_null_as_string")]
    pub version: String,
    pub engine: Option<String>,
    pub engine_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Device {
    pub r#type: String,
    pub brand: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct OS {
    pub name: Option<String>,
    pub version: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BotEntry {
    bot: Bot,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Bot {
    pub name: String,
    pub category: Option<String>,
    pub url: Option<String>,
    pub producer: Option<BotProducer>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BotProducer {
    #[serde(deserialize_with = "decode_string_or_null_as_string")]
    pub name: String,
    pub url: String,
}

fn decode_string_or_null_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNull {}

    impl<'de> Visitor<'de> for StringOrNull {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a bot producer name, when null")
        }

        fn visit_str<E>(self, val: &str) -> Result<Self::Value, E> {
            Ok(val.to_owned())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok("".to_owned())
        }
    }

    deserializer.deserialize_any(StringOrNull {})
}

fn decode_array_or_map<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    struct ArrayOrMap<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for ArrayOrMap<T>
    where
        T: Deserialize<'de> + Default,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "an array or a map representing an OS")
        }

        fn visit_seq<A>(self, mut _seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // in test cases an empty array is used to represent an empty struct
            Ok(Default::default())
        }
        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(ArrayOrMap(PhantomData))
}

#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub user_agent: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub expected: Detection,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct TestCases {
    cases: Vec<TestCase>,
}

use std::path::PathBuf;

async fn test_fixture(detector: super::DeviceDetector, file_index: usize, path: PathBuf) {
    // println!("fixture path: {:?}", path);
    let file = File::open(path.clone()).expect("valid file");
    let reader = BufReader::new(file);
    let mut bots = 0;
    let mut known_devices = 0;

    let cases: TestCases = serde_yaml::from_reader(reader).expect("test cases");
    for (entry, case) in cases.cases.into_iter().enumerate().skip(0) {
        let entry = entry + 1;

        let headers = case
            .headers
            // .take() vs .clone for faster tests
            .clone()
            .map(|x| x.into_iter().collect::<Vec<(String, String)>>());

        let res = detector.parse(case.user_agent.as_str(), headers);
        // println!("res {:?}", &res);
        match &res.await.unwrap() {
            super::Detection::Bot(bot) => {
                bots += 1;
                assert_eq!(bot.name, case.expected.get_bot().name);
                assert_eq!(bot.category, case.expected.get_bot().category);
                assert_eq!(bot.url, case.expected.get_bot().url);
                assert_eq!(
                    bot.producer.as_ref().map(|x| &x.url),
                    case.expected.get_bot().producer.as_ref().map(|x| &x.url)
                );

                assert_eq!(
                    bot.producer.as_ref().map(|x| &x.name),
                    case.expected.get_bot().producer.as_ref().map(|x| &x.name)
                );
            }
            _wtf @ super::Detection::Known(super::KnownDevice { client, device, os }) => {
                known_devices += 1;

                let us: Option<&str> = os.as_ref().and_then(|x| x.family.as_deref());
                let them: &str = &case.expected.get_device().os_family;

                let family_equality = us == Some(them) || us.is_none() && them == "Unknown";

                assert!(
                    family_equality,
                    "os family filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.display(),
                    file_index,
                    entry,
                    &us,
                    &them,
                );

                let us: Option<&str> = os.as_ref().and_then(|x| x.platform.as_deref());
                let them: Option<&str> = case.expected.get_device().os.platform.as_deref();

                let platform_equality = us == them || us.is_none() && them == Some("");
                assert!(
                    platform_equality,
                    "os platform filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.display(),
                    file_index,
                    entry,
                    &us,
                    &them,
                );

                let us: Option<&str> = client
                    .as_ref()
                    .and_then(|x| x.browser.as_ref())
                    .and_then(|x| x.family.as_deref());

                let them = &case.expected.get_device().browser_family;

                let browser_family_equality = us == Some(them) || us.is_none() && them == "Unknown";

                assert!(
                    browser_family_equality,
                    "browser family filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &us,
                    &them,
                );

                let us: Option<&str> = os.as_ref().and_then(|x| x.version.as_deref());
                let them: Option<&str> = case.expected.get_device().os.version.as_deref();

                assert!(
                    us == them || us.is_none() && them.unwrap() == "",
                    "os version filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &os,
                    &case.expected.get_device().os,
                );

                let us: Option<&str> = os.as_ref().map(|x| x.name.as_str());
                let them: Option<&str> = case.expected.get_device().os.name.as_deref();

                assert!(
                    us == them,
                    "os name filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &os,
                    &case.expected.get_device().os,
                );

                let us: Option<&str> = device
                    .as_ref()
                    .and_then(|device| device.device_type.as_ref())
                    .map(|device_type| device_type.as_str());
                let them: Option<&str> = Some(&case.expected.get_device().device.r#type);

                assert!(
                    us == them || (us.is_none() && them.is_some() && them.unwrap() == ""),
                    "device_type filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &device,
                    &case.expected.get_device().device,
                );

                let us = device.as_ref().and_then(|device| device.brand.as_ref());
                let them = case.expected.get_device().device.brand.as_ref();

                let brand_equality =
                    us == them || (us.is_none() && them.is_some() && them.unwrap() == "");

                assert!(
                    brand_equality,
                    "device_brand filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &device,
                    &case.expected.get_device().device
                );

                let us: Option<&str> = device.as_ref().and_then(|device| device.model.as_deref());
                let them: Option<&str> = case.expected.get_device().device.model.as_deref();

                let model_equality =
                    us == them || us.is_none() && them.is_some() && them.as_ref().unwrap() == &"";
                assert!(
                    model_equality,
                    "device_model filename: {} file: {} entry: {}\n us: {:?}\n them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    &device,
                    &case.expected.get_device().device
                );

                assert_eq!(
                    client.as_ref().map(|x| &x.name),
                    case.expected.get_device().client.as_ref().map(|x| &x.name),
                    "client_name filename: {} file: {} entry: {} ua: {}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    case.user_agent
                );

                let us: Option<&str> = client.as_ref().map(|x| x.r#type.as_str());
                let them: Option<&str> = case
                    .expected
                    .get_device()
                    .client
                    .as_ref()
                    .map(|x| x.r#type.as_ref());

                assert_eq!(
                    us,
                    them,
                    "client_type filename: {} file: {} entry: {} ua: {} us: {:?} them: {:?}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    case.user_agent,
                    us,
                    them
                );

                assert_eq!(
                    client.as_ref().map(|x| &x.version),
                    case.expected
                        .get_device()
                        .client
                        .as_ref()
                        .map(|x| &x.version),
                    "client_version: filename: {} file: {} entry: {} ua: {}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    case.user_agent
                );

                let client_engine: Option<Option<String>> =
                    client.as_ref().map(|x| x.engine.clone());
                let testcase_engine: Option<Option<String>> = case
                    .expected
                    .get_device()
                    .client
                    .as_ref()
                    .map(|x| x.engine.clone())
                    // A client with an empty engine is represented as None in rust
                    // but can be represented as an empty string in the test cases, but they
                    // are equivalent
                    .map(|x| if x == Some("".to_owned()) { None } else { x });

                assert_eq!(
                    client_engine,
                    testcase_engine,
                    "client_engine: filename: {} file: {} entry: {} ua: {}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    case.user_agent
                );

                let client_engine_version: Option<Option<String>> =
                    client.as_ref().map(|x| x.engine_version.clone());
                let testcase_engine_version: Option<Option<String>> = case
                    .expected
                    .get_device()
                    .client
                    .as_ref()
                    .map(|x| x.engine_version.clone())
                    // A client with an empty engine version is represented as None in rust
                    // but can be represented as an empty string in the test cases, but they
                    // are equivalent
                    .map(|x| if x == Some("".to_owned()) { None } else { x });

                assert_eq!(
                    client_engine_version,
                    testcase_engine_version,
                    "client_engine_version: filename: {} file: {} entry: {} ua: {}",
                    path.to_str().unwrap(),
                    file_index,
                    entry,
                    case.user_agent
                );
            }
        }
    }
    println!(
        "fixture {}, bots {} known_devices {}",
        path.display(),
        bots,
        known_devices
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn test_fixtures() {
    let detector = super::DeviceDetector::new();

    let res: Vec<_> = glob::glob("tests/fixtures/*.yml")
        .expect("text fixtures")
        .map(|x| x.expect("glob"))
        .enumerate()
        .skip(0)
        .map(|(i, path)| {
            let detector = detector.clone();
            tokio::spawn(test_fixture(detector, i, path))
        })
        .collect::<Vec<_>>();

    for i in res {
        i.await.unwrap()
    }
}
