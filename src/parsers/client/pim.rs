use anyhow::Result;

use super::{Client, ClientList};
use once_cell::sync::Lazy;

static CLIENT_LIST: Lazy<ClientList> = Lazy::new(|| {
    let contents = std::include_str!("../../../regexes/client/pim.yml");
    ClientList::from_file(contents).expect("loading pim.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Client>> {
    CLIENT_LIST.lookup(ua, super::ClientType::Pim)
}
