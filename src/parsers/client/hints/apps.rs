use anyhow::Result;

use super::HintList;
use once_cell::sync::Lazy;

static HINT_LIST: Lazy<HintList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/client/hints/apps.yml"
    ));
    HintList::from_file(contents).expect("loading hints/apps.yml")
});

pub fn get_hint(app: &str) -> Result<Option<&str>> {
    HINT_LIST.get_hint(app)
}
