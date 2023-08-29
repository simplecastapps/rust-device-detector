use anyhow::Result;

use fancy_regex::{Captures, Regex};

use once_cell::sync::OnceCell;

#[derive(Debug)]
pub(crate) struct LazyRegex {
    pattern: String,
    regex: OnceCell<Regex>,
}

impl LazyRegex {
    pub(crate) fn new(pattern: String) -> Self {
        Self {
            pattern,
            regex: OnceCell::new(),
        }
    }

    pub(crate) fn is_match(&self, text: &str) -> Result<bool> {
        let regex = self.regex.get_or_try_init(|| Regex::new(&self.pattern))?;
        Ok(regex.is_match(text)?)
    }

    pub(crate) fn captures<'t>(&self, text: &'t str) -> Result<Option<Captures<'t>>> {
        let regex = self.regex.get_or_try_init(|| Regex::new(&self.pattern))?;
        Ok(regex.captures(text)?)
    }
}

pub(crate) fn user_agent_match(pattern: &str) -> LazyRegex {
    // only match if useragent begins with given regex or there is no letter before it
    let mut reg = r"(?i:^|[^A-Z0-9\-_]|[^A-Z0-9\-]_|sprd-|MZ-)(?i:".to_owned();
    reg.push_str(pattern.replace('/', r"\/").as_str());
    reg.push(')');
    LazyRegex::new(reg)
}

pub(crate) fn expand(template: &str, dst: &mut String, captures: &Captures<'_>) {
    // If Expander's internals were public, we could just change allow_undelimited_name to false
    // and we wouldn't need this utility at all.
    // https://docs.rs/fancy-regex/latest/fancy_regex/struct.Expander.html
    //
    // would just use `caps.expand(&template, &mut dst);`
    use fancy_regex::Expander;
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"\$([1-9])").unwrap();
    }

    //dbg!(template);
    let template = RE.replace_all(template, |caps: &Captures<'_>| format!("${{{}}}", &caps[1]));

    //dbg!(&dst, &*template, &captures);
    Expander::default().append_expansion(dst, &template, captures);
}
