use super::builder::{HttpMethod, RequestBuilder};

#[derive(Clone, Debug)]
pub struct HttpClient {
    base_url: String,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            base_url: String::new(),
        }
    }
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn url(&self, path: &str) -> String {
        let clean_path = path.trim_start_matches('/');
        if self.base_url.is_empty() {
            format!("/{clean_path}")
        } else {
            format!("{}/{clean_path}", self.base_url)
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Get, self.url(path))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Post, self.url(path))
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Put, self.url(path))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Delete, self.url(path))
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(HttpMethod::Patch, self.url(path))
    }
}
