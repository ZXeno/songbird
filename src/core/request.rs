use serde::{Deserialize, Serialize};

use super::{Header, HttpMethod};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<Header>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push(Header::new(name, value));
        self
    }

    pub fn with_headers(mut self, headers: Vec<Header>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub reason: Option<String>,
    pub headers: Vec<Header>,
    pub body: String,
    pub duration_ms: u64,
    pub truncated: bool,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}
