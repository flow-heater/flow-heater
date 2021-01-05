use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};
use warp::http;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub(crate) code: usize,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<Vec<u8>>,
}

impl Response {
    pub fn error_msg(reason: &str, r: Self) -> anyhow::Error {
        anyhow::Error::msg(format!("{}: {:?}", reason, r))
    }
}

impl TryFrom<http::Response<Vec<u8>>> for Response {
    type Error = anyhow::Error;

    fn try_from(_value: http::Response<Vec<u8>>) -> Result<Self, Self::Error> {
        todo!()
    }
}
