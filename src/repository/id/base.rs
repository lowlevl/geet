use std::borrow::Cow;

use parse_display::Display;
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::Error;

/// A valid base for either a namespace or a repository name.
#[derive(Debug, Display, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct Base(pub(crate) Cow<'static, str>);

impl Base {
    fn is_authorized(c: char) -> bool {
        matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '-' | '?' | '.')
    }
}

impl std::str::FromStr for Base {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.len() > 255 {
            Err(Error::IllegalSize)?
        }

        if s.starts_with('.') || s.ends_with('.') {
            Err(Error::IllegalDot)?
        }

        if !s.chars().all(Self::is_authorized) {
            Err(Error::IllegalFormat)?
        }

        Ok(Self(s.to_string().into()))
    }
}

impl std::ops::Deref for Base {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
