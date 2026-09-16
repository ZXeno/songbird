use std::time::{Duration, Instant};

use iced::futures::future::BoxFuture;

use crate::core::{Header, HttpMethod, HttpRequest, HttpResponse};

use super::{HttpError, HttpExecutor};

const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

pub struct ReqwestExecutor {
    client: reqwest::Client,
}

impl ReqwestExecutor {
    pub fn new() -> Result<Self, HttpError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self { client })
    }
}

fn to_reqwest_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}

impl HttpExecutor for ReqwestExecutor {
    fn execute(&self, request: HttpRequest) -> BoxFuture<'static, Result<HttpResponse, HttpError>> {
        let client = self.client.clone();
        Box::pin(async move {
            let url = reqwest::Url::parse(&request.url)
                .map_err(|error| HttpError::InvalidUrl(error.to_string()))?;
            let mut builder = client.request(to_reqwest_method(request.method), url);
            for header in &request.headers {
                builder = builder.header(header.name.as_str(), header.value.as_str());
            }
            if let Some(body) = request.body {
                builder = builder.body(body);
            }

            let started = Instant::now();
            let response = builder.send().await?;
            let status = response.status();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    Header::new(name.as_str(), value.to_str().unwrap_or("<binary>"))
                })
                .collect();
            let raw = response.bytes().await?;
            let truncated = raw.len() > MAX_BODY_BYTES;
            let window = raw.len().min(MAX_BODY_BYTES);
            let body = String::from_utf8_lossy(&raw[..window]).into_owned();
            let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

            Ok(HttpResponse {
                status: status.as_u16(),
                reason: status.canonical_reason().map(str::to_owned),
                headers,
                body,duration_ms,
                truncated,
            })
        })
    }
}
