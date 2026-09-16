use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub const ALL: [HttpMethod; 7] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Patch,
        HttpMethod::Delete,
        HttpMethod::Head,
        HttpMethod::Options,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }

    pub const fn allows_body(self) -> bool {
        matches!(
            self,
            HttpMethod::Post
            | HttpMethod::Put
            | HttpMethod::Patch
            | HttpMethod::Delete
            | HttpMethod::Options
        )
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for HttpMethod {
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
        where T: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'dl> Deserialize<'dl> for HttpMethod {
    fn deserialize<T>(deserializer: T) -> Result<Self, T::Error>
        where T: Deserializer<'dl>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

impl FromStr for HttpMethod {
    type Err = UnknownMethod;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HttpMethod::ALL
            .into_iter()
            .find(|method| method.as_str().eq_ignore_ascii_case(s))
            .ok_or_else(|| UnknownMethod(s.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown HTTP method '{0}'")]
pub struct UnknownMethod(String);
