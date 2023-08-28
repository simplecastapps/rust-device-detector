use anyhow::Result;

use lazy_static::lazy_static;

use super::HintList;

lazy_static! {
    static ref HINT_LIST: HintList = {
        let contents = std::include_str!("../../../../regexes/client/hints/browsers.yml");
        HintList::from_file(contents).expect("loading hints/browsers.yml")
    };
}

pub fn get_hint(app: &str) -> Result<Option<&str>> {
    HINT_LIST.get_hint(app)
}
