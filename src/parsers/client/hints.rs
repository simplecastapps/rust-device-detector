use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

pub mod apps;
pub mod browsers;

#[derive(Debug, Deserialize)]
pub struct HintList {
    #[serde(flatten)]
    pub hints: HashMap<String, String>,
}

impl HintList {
    pub fn from_file(contents: &str) -> Result<HintList> {
        let hints: HintList = serde_yaml::from_str(contents)?;
        Ok(hints)
    }

    pub fn get_hint(&self, app: &str) -> Result<Option<&str>> {
        let res = self.hints.get(app);
        Ok(res.map(|s| s.as_ref()))
    }
}
