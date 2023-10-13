use anyhow::Result;

use super::{Client, ClientList, ClientType};
use once_cell::sync::Lazy;

use crate::client_hints::ClientHint;

static CLIENT_LIST: Lazy<ClientList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/client/mobile_apps.yml"
    ));
    ClientList::from_file(contents).expect("loading mobile_apps.yml")
});

pub fn lookup(ua: &str, client_hints: Option<&ClientHint>) -> Result<Option<Client>> {
    let client = CLIENT_LIST.lookup(ua, super::ClientType::MobileApp)?;

    if let Some(client_hints) = client_hints {
        if let Some(app_hint) = &client_hints.app {
            if let Some(app) = super::hints::apps::get_hint(app_hint)? {
                // println!("app: {:?}", app);
                if client.is_none() || client.as_ref().unwrap().name != app {
                    // println!("client.is_none() || client.as_ref().unwrap().name != app");
                    return Ok(Some(Client {
                        r#type: ClientType::MobileApp,
                        name: app.into(),
                        version: None,
                        browser: None,
                        engine: None,
                        engine_version: None,
                    }));
                }
            }
        }
    }

    Ok(client)
}
