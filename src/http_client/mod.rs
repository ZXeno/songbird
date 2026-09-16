mod reqwest_executor;

use iced::futures::future::BoxFuture;
use thiserror::Error;

use crate::core::{HttpRequest, HttpResponse};

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("request failed: {0}")]
    Network(#[from] reqwest::Error),
}

pub trait HttpExecutor: Send + Sync {
    fn execute(&self, request: HttpRequest) -> BoxFuture<'static, Result<HttpResponse, HttpError>>;
}

pub use reqwest_executor::ReqwestExecutor;
