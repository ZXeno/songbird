use iced::widget::text_editor;

use crate::core::{Header, HttpMethod, HttpRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonBodyStatus {
    Empty,
    Valid,
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct HeaderRow{
    pub enabled: bool,
    pub header: Header,
}

impl HeaderRow {
    fn from_header(header: Header) -> Self {
        Self {
            enabled: true,
            header,
        }
    }

    fn empty() -> Self {
        Self {
            enabled: true,
            header: Header::new(String::new(), String::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestEditor {
    pub method: HttpMethod,
    pub url: String,
    pub body: text_editor::Content,
    pub headers: Vec<HeaderRow>,
}

impl RequestEditor {
    pub fn empty() -> Self {
        Self {
            method: HttpMethod::Get,
            url: String::new(),
            body: text_editor::Content::new(),
            headers: Vec::new()
        }
    }

    pub fn load(request: &HttpRequest) -> Self {
        Self {
            method: request.method,
            url: request.url.clone(),
            body: text_editor::Content::with_text(request.body.as_deref().unwrap_or_default()),
            headers: request
                .headers
                .iter()
                .cloned()
                .map(HeaderRow::from_header)
                .collect(),
        }
    }

    pub fn to_request(&self) -> HttpRequest {
        let headers = self.headers
            .iter()
            .filter(|row| row.enabled && !row.header.name.trim().is_empty())
            .map(|row| Header::new(row.header.name.trim(), row.header.value.clone()))
            .collect();
        let body = self.method.allows_body()
            .then(|| self.body.text())
            .filter(|text| !text.trim().is_empty());

        HttpRequest {
            method: self.method,
            url: self.url.trim().to_owned(),
            headers,
            body,
        }
    }

    pub fn json_status(&self) -> JsonBodyStatus {
        let text = self.body.text();
        if text.trim().is_empty() {
            return JsonBodyStatus::Empty;
        }
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(_) => JsonBodyStatus::Valid,
            Err(error) => JsonBodyStatus::Invalid(error.to_string()),
        }
    }

    pub fn add_header(&mut self) {
        self.headers.push(HeaderRow::empty());
    }

    pub fn remove_header(&mut self, index: usize) {
        if index < self.headers.len() {
            self.headers.remove(index);
        }
    }
}

impl Default for RequestEditor {
    fn default() -> Self {
        Self::empty()
    }
}
