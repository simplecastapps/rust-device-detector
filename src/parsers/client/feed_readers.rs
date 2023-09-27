use anyhow::Result;

use super::{Client, ClientList};
use once_cell::sync::Lazy;

static CLIENT_LIST: Lazy<ClientList> = Lazy::new(|| {
    let contents = std::include_str!("../../../regexes/client/feed_readers.yml");
    ClientList::from_file(contents).expect("loading feed_readers.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Client>> {
    CLIENT_LIST.lookup(ua, super::ClientType::FeedReader)
}
