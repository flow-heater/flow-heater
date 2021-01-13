use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str;

use crate::{try_header_map_to_hashmap, version_to_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub code: u16,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<String>,
    pub version: String,
}

impl Response {
    pub fn error_msg(reason: &str, r: Self) -> anyhow::Error {
        anyhow::Error::msg(format!("{}: {:?}", reason, r))
    }
}

impl Response {
    pub async fn try_from_response(resp: reqwest::Response) -> Result<Self, anyhow::Error> {
        let mut r = Response {
            code: resp.status().as_u16(),
            body: None,
            headers: try_header_map_to_hashmap(resp.headers().clone())?,
            version: version_to_string(resp.version()),
        };

        r.body = Some(String::from_utf8(resp.bytes().await?.to_vec())?);

        Ok(r)
    }
}
