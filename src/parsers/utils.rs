use anyhow::Result;

use fancy_regex::{Captures, Regex, CaptureMatches, Error, Expander, Replacer};
use once_cell::sync::Lazy;

use once_cell::sync::OnceCell;

#[derive(Debug)]
pub(crate) struct LazyRegex {
    pattern: String,
    regex: OnceCell<SafeRegex>,
}
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use std::borrow::Cow;

/// This is a regex that won't crash due to run time errors on match.
/// This can still crash if passed an invalid regex in the first place.
#[derive(Debug)]
pub(crate) struct SafeRegex {
    regex: Regex,
}

impl SafeRegex {

    fn squash_runtime_error<T>(err: Result<T, Error>, ret: T) -> Result<T, Error> {
        // this is either a stack overflow or a backtrack limit reached.
        // in either case, we don't want to crash, just deny a match and move on.
        match err {
            Err(Error::RuntimeError(_)) => Ok(ret),
            err => err
        }
    }

    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)?;
        Ok(Self { regex })
    }

    pub fn is_match(&self, text: &str) -> Result<bool> {
        let res = Self::squash_runtime_error(self.regex.is_match(text), false)?;
        Ok(res)
    }

    pub fn captures<'t>(&self, text: &'t str) -> Result<Option<Captures<'t>>> {
        let res = Self::squash_runtime_error(self.regex.captures(text), None)?;
        Ok(res)
    }

    pub fn replace_all<'t, R: Replacer>(&self, text: &'t str, rep: R) -> Cow<'t, str> {
        self.regex.replace_all(text, rep)
    }

    pub fn captures_iter<'r, 'h>(
    &'r self,
    haystack: &'h str
) -> CaptureMatches<'r, 'h> {
        self.regex.captures_iter(haystack)
    }
}

pub(crate) struct LimitedUserMatchRegex {
    limit: usize,
    hm: Arc<RwLock<HashMap<String, Arc<SafeRegex>>>>,
}

impl LazyRegex {
    pub(crate) fn new(pattern: String) -> Self {
        Self {
            pattern,
            regex: OnceCell::new(),
        }
    }

    pub(crate) fn is_match(&self, text: &str) -> Result<bool> {
        let regex = self.regex.get_or_try_init(|| {
            // println!("is_match compilation: {}", &self.pattern);
            SafeRegex::new(&self.pattern)
        })?;
        Ok(regex.is_match(text)?)
    }

    pub(crate) fn captures<'t>(&self, text: &'t str) -> Result<Option<Captures<'t>>> {
        let regex = self.regex.get_or_try_init(|| {
            // println!("captures compilation: {}", &self.pattern);
            SafeRegex::new(&self.pattern)
        })?;
        Ok(regex.captures(text)?)
    }
}

impl LimitedUserMatchRegex {
    /// Creates a new LimitedUserMatchRegex with the given limit. It will go beyond
    /// that limit, but will warn on every new entry. At double this limit, it will
    /// panic because it can't continue that way forever, the memory use would be
    /// unbounded.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            hm: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Either returns a cached regex, or compiles a new one and caches it.
    pub fn regex(&self, key: &str) -> Arc<SafeRegex> {
        let hm = self.hm.clone();
        {
            let hm = hm.read().unwrap();
            hm.get(key).cloned()
        }
        .unwrap_or_else(|| {
            let mut reg = "(?i:".to_owned();
            reg.push_str(key);
            reg.push_str(r#"\s*/?\s*((?=\d+\.\d)\d+[.\d]*|\d{1,7}(?=(?:\D|$)))"#);
            reg.push(')');

            // println!("LimitedUserMatchRegex compilation: {}", reg);
            let mut hm = hm.write().unwrap();

            if hm.len() >= self.limit {
                eprintln!("LimitedUserMatchRegex limit of {} reached by key '{}', which is incredibly bad and should be investigated", key, self.limit);
            }

            if hm.len() >= self.limit * 2 {
                panic!("LimitedUserMatchRegex limit of {} doubled reached by key '{}', and it can't be allowed to continue, exiting", key, self.limit);
            }

            let value = Arc::new(SafeRegex::new(&reg).expect("compilable pattern"));
            hm.insert(key.to_owned(), value.clone());
            value
        })
    }
}

macro_rules! static_user_agent_match {
    ($re:literal $(,)?) => {{
        Lazy::new(|| {
            let reg = const_format::concatcp!(
                r"(?i:^|[^A-Z0-9\-_]|[^A-Z0-9\-]_|sprd-|MZ-)(?i:",
                $re,
                r")"
            );
            // println!("static_user_agent_match compilation: {}", reg);
            Regex::new(reg).expect("static user agent match regex")
        })
    }};
}
pub(crate) use static_user_agent_match;

pub(crate) fn lazy_user_agent_match(pattern: &str) -> LazyRegex {
    let mut reg = r"(?i:^|[^A-Z0-9\-_]|[^A-Z0-9\-]_|sprd-|MZ-)(?i:".to_owned();
    reg.push_str(pattern.replace('/', r"\/").as_str());
    reg.push(')');

    // println!("lazy_user_agent_match compilation: {}", reg);
    LazyRegex::new(reg)
}

pub(crate) fn expand(template: &str, dst: &mut String, captures: &Captures<'_>) {
    // If Expander's internals were public, we could just change allow_undelimited_name to false
    // and we wouldn't need this utility at all.
    // https://docs.rs/fancy-regex/latest/fancy_regex/struct.Expander.html
    //
    // would just use `caps.expand(&template, &mut dst);`
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$([1-9])").unwrap());

    //dbg!(template);
    let template = RE.replace_all(template, |caps: &Captures<'_>| format!("${{{}}}", &caps[1]));

    //dbg!(&dst, &*template, &captures);
    Expander::default().append_expansion(dst, &template, captures);
}
