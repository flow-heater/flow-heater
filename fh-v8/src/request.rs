use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use warp::http;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestSpec {
    pub(crate) request: Request,
    pub(crate) url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) query: String,
}

impl From<http::Request<Vec<u8>>> for Request {
    fn from(req: http::Request<Vec<u8>>) -> Self {
        let mut headers = HashMap::new();
        for h in req.headers() {
            headers.insert(h.0.to_string(), h.1.to_str().unwrap().to_string());
        }

        let (parts, body) = req.into_parts();

        Request {
            body: String::from_utf8(body).unwrap(),
            headers,
            method: parts.method.to_string(),
            path: parts.uri.path().to_string(),
            query: parts.uri.query().unwrap_or("").to_string(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RequestList {
    pub(crate) inner: Vec<Request>,
}

impl Deref for RequestList {
    type Target = Vec<Request>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RequestList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
