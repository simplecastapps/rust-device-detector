use anyhow::Result;
use serde::Deserialize;

use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::utils::lazy_user_agent_match;
use crate::parsers::utils::LazyRegex;

#[derive(Debug)]
struct VendorFragments {
    vendor: String,
    fragments: Vec<LazyRegex>,
}

static FRAGMENT_LIST: Lazy<VendorFragmentList> = Lazy::new(|| {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regexes/vendorfragments.yml"
    ));
    VendorFragmentList::from_file(contents).expect("loading vendorfragments.yml")
});

pub fn lookup(ua: &str) -> Result<Option<&str>> {
    FRAGMENT_LIST.lookup(ua)
}

#[derive(Debug)]
struct VendorFragmentList {
    list: Vec<VendorFragments>,
}

impl VendorFragments {
    fn is_match(&self, ua: &str) -> Result<bool> {
        for x in self.fragments.iter() {
            if x.is_match(ua)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl VendorFragmentList {
    fn lookup(&self, ua: &str) -> Result<Option<&str>> {
        for x in self.list.iter() {
            if x.is_match(ua)? {
                return Ok(Some(&x.vendor));
            }
        }
        Ok(None)
    }

    fn from_file(contents: &str) -> Result<VendorFragmentList> {
        #[derive(Debug, Deserialize)]
        #[serde(transparent)]
        struct YamlVendorFragmentList {
            list: HashMap<String, Vec<String>>,
        }

        #[allow(clippy::from_over_into)]
        impl Into<VendorFragmentList> for YamlVendorFragmentList {
            fn into(self) -> VendorFragmentList {
                let frags = self
                    .list
                    .into_iter()
                    .map(|(vendor, frags)| VendorFragments {
                        vendor,
                        fragments: frags
                            .iter()
                            .map(|x| {
                                let x = x.to_owned() + "[^a-z0-9]+";
                                lazy_user_agent_match(&x)
                            })
                            .collect(),
                    })
                    .collect();

                VendorFragmentList { list: frags }
            }
        }

        let res: YamlVendorFragmentList = serde_yaml::from_str(contents)?;
        Ok(res.into())
    }
}
