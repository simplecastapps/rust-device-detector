use anyhow::Result;

use super::{Client, ClientList};
use once_cell::sync::Lazy;

static CLIENT_LIST: Lazy<ClientList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/client/libraries.yml"
    ));
    ClientList::from_file(contents).expect("loading libraries.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Client>> {
    CLIENT_LIST.lookup(ua, super::ClientType::Library)
}
