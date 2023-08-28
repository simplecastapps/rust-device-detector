use anyhow::Result;

use lazy_static::lazy_static;

use super::{Client, ClientList};

lazy_static! {
    static ref CLIENT_LIST: ClientList = {
        let contents = std::include_str!("../../../regexes/client/libraries.yml");
        ClientList::from_file(contents).expect("loading libraries.yml")
    };
}

pub fn lookup(ua: &str) -> Result<Option<Client>> {
    CLIENT_LIST.lookup(ua, super::ClientType::Library)
}
