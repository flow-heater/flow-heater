use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
    ops::{Deref, DerefMut},
};
use warp::http;

use crate::{try_header_map_to_hashmap, version_to_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestSpec {
    pub request: Request,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub headers: HashMap<String, Vec<String>>,
    pub body: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub version: String,
}

impl TryFrom<http::Request<Vec<u8>>> for Request {
    type Error = anyhow::Error;

    fn try_from(req: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
        let (parts, body) = req.into_parts();

        Ok(Request {
            body: String::from_utf8(body)?,
            headers: try_header_map_to_hashmap(parts.headers)?,
            method: parts.method.to_string(),
            path: parts.uri.path().to_string(),
            query: parts.uri.query().and_then(|x| Some(x.to_string())),
            version: version_to_string(parts.version),
        })
    }
}

#[derive(Debug)]
pub struct RequestList {
    pub inner: Vec<Request>,
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
