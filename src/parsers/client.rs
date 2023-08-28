// mobile_apps regex name version
// libraries regex name version
// mediaplayers regex name version
// mobile_apps regex name version
// pim regex name version
//
// browser_engine regex name
//
// feed_readers regex name version url type
//
// browsers regex name version (optional engine (default))
//
//
// hints/apps key value
// hints/browsers key value
//
//
//

use anyhow::Result;

use serde::{Deserialize, Serialize};

use serde::de::Deserializer;

use crate::known_browsers::AvailableBrowser;
use crate::parsers::utils::user_agent_match;
use crate::parsers::utils::LazyRegex;

pub mod browsers;
pub mod feed_readers;
pub mod hints;
pub mod libraries;
pub mod media_players;
pub mod mobile_apps;
pub mod pim;

use crate::client_hints::ClientHint;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum ClientType {
    #[serde(rename = "browser")]
    Browser,
    #[serde(rename = "feed reader")]
    FeedReader,
    #[serde(rename = "mobile app")]
    MobileApp,
    #[serde(rename = "pim")]
    Pim,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "mediaplayer")]
    MediaPlayer,
}

impl ClientType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientType::Browser => "browser",
            ClientType::FeedReader => "feed reader",
            ClientType::MobileApp => "mobile app",
            ClientType::Pim => "pim",
            ClientType::Library => "library",
            ClientType::MediaPlayer => "mediaplayer",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Client {
    pub name: String,
    pub version: String,
    pub r#type: ClientType,
    pub engine: Option<String>,
    pub engine_version: Option<String>,

    #[serde(skip)]
    pub(crate) browser: Option<AvailableBrowser>,
}

pub fn lookup(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<Client>> {
    if let Some(res) = feed_readers::lookup(ua)? {
        return Ok(Some(res));
    }
    if let Some(res) = mobile_apps::lookup(ua, client_hints)? {
        return Ok(Some(res));
    }
    if let Some(res) = media_players::lookup(ua)? {
        return Ok(Some(res));
    }

    if let Some(res) = pim::lookup(ua)? {
        return Ok(Some(res));
    }

    if let Some(res) = browsers::lookup(ua, client_hints)? {
        return Ok(Some(res));
    }

    if let Some(res) = libraries::lookup(ua)? {
        return Ok(Some(res));
    }

    Ok(None)
}

#[derive(Debug, Deserialize)]
pub struct ClientEntry {
    name: String,
    #[serde(deserialize_with = "de_regex")]
    regex: LazyRegex,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ClientList {
    clients: Vec<ClientEntry>,
}

impl ClientList {
    pub fn lookup(&self, ua: &str, r#type: ClientType) -> Result<Option<Client>> {
        for client in self.clients.iter() {
            if client.regex.is_match(ua)? {
                let mut version = "".to_owned();
                let mut name = "".to_owned();
                let caps = client.regex.captures(ua)?.expect("valid_regex");

                // expands $1, $2 etc in names / versions to captures from regex
                caps.expand(&client.version, &mut version);
                // TODO I don't know if this is needed, but here it is.
                let version = if version.ends_with(&['.', ' ']) {
                    version.trim_end_matches(&['.', ' ']).to_owned()
                } else {
                    version
                };

                caps.expand(&client.name, &mut name);

                return Ok(Some(Client {
                    name,
                    version,
                    r#type,
                    browser: None,
                    engine: None,
                    engine_version: None,
                }));
            }
        }

        Ok(None)
    }
    pub fn from_file(contents: &str) -> Result<ClientList> {
        let res = serde_yaml::from_str(contents)?;
        Ok(res)
    }
}

fn de_regex<'de, D>(deserializer: D) -> Result<LazyRegex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(user_agent_match(&s))
}
