use anyhow::Result;

use super::HintList;
use once_cell::sync::Lazy;

static HINT_LIST: Lazy<HintList> = Lazy::new(|| {
    let contents = std::include_str!("../../../../regexes/client/hints/browsers.yml");
    HintList::from_file(contents).expect("loading hints/browsers.yml")
});

pub fn get_hint(app: &str) -> Result<Option<&str>> {
    HINT_LIST.get_hint(app)
}
