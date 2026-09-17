use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::HttpRequest;

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: String,
    pub name: String,
    pub request: HttpRequest,
}

impl SavedRequest {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            id: new_id(),
            name: derive_name(&request),
            request,
        }
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}

pub fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let sequence = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos:x}-{sequence:x}")
}

fn derive_name(request: &HttpRequest) -> String {
    format!("{} {}", request.method, request.url)
}
