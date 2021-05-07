use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};
use warp::http;

use crate::{
    response::Response, try_actix_header_map_to_hashmap, try_header_map_to_hashmap,
    version_to_string,
};

/// Simple wrapper type which contains the request to be made and a URL, where
/// the request should be sent to.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestSpec {
    pub request: Request,
    pub url: String,
}

/// (De-)Serializable representation of a HTTP Request.
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

impl TryFrom<actix_web::HttpRequest> for Request {
    type Error = anyhow::Error;

    fn try_from(value: actix_web::HttpRequest) -> Result<Self, Self::Error> {
        Ok(Request {
            body: "".into(),
            headers: try_actix_header_map_to_hashmap(value.headers().clone())?,
            method: value.method().to_string(),
            path: value.uri().to_string(),
            query: Some(value.query_string().to_string()),
            version: version_to_string(value.version()),
        })
    }
}

/// Wrapper for an ordered request/response list. This is used to determine the
/// last response body, which is the one which should be finally returned back
/// to the client.
#[derive(Debug)]
pub struct RequestResponseList {
    pub requests: HashMap<usize, Request>,
    pub responses: HashMap<usize, Response>,
}

impl RequestResponseList {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            responses: HashMap::new(),
        }
    }

    pub fn add_request(&mut self, idx: usize, req: Request) {
        self.requests.insert(idx, req);
    }

    pub fn add_response(&mut self, idx: usize, resp: Response) {
        self.responses.insert(idx, resp);
    }

    /// Returns the last response body, if there are any responses stored, yet.
    /// Returns None otherwise.
    pub fn get_last_response_body(&self) -> Option<String> {
        if self.responses.len() > 0 {
            return Some(
                self.responses
                    .iter()
                    .last()
                    .unwrap()
                    .1
                    .clone()
                    .body
                    .unwrap_or("".to_string()),
            );
        }

        None
    }
}
