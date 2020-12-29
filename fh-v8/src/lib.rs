// use anyhow::Error;
// use deno_core::error::AnyError;
use deno_core::JsRuntime;
// use deno_core::OpState;
// use deno_core::ZeroCopyBuf;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::collections::HashMap;
use warp::http;

#[derive(Debug)]
pub struct Request {
    headers: HashMap<String, String>,
    body: Vec<u8>,
    method: String,
    path: String,
    query: String,
    host: String,
    port: u16,
    scheme: String,
}

#[derive(Debug)]
pub struct Response {
    code: usize,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl From<http::Request<Vec<u8>>> for Request {
    fn from(req: http::Request<Vec<u8>>) -> Self {
        let mut headers = HashMap::new();
        for h in req.headers() {
            headers.insert(h.0.to_string(), h.1.to_str().unwrap().to_string());
        }

        let (parts, body) = req.into_parts();

        Request {
            body,
            headers,
            method: parts.method.to_string(),
            path: parts.uri.path().to_string(),
            query: parts.uri.query().unwrap_or("").to_string(),
            host: parts.uri.host().unwrap_or("").to_string(),
            port: parts.uri.port_u16().unwrap_or(0),
            scheme: parts.uri.scheme_str().unwrap_or("").to_string(),
        }
    }
}

pub async fn process_request(_req: &Request) -> Response {
    let mut js_runtime = JsRuntime::new(Default::default());
    js_runtime
        .execute("asd", "Deno.core.print(`Hello from deno!\n`);")
        .unwrap();
    js_runtime.run_event_loop().await.unwrap();

    Response {
        code: 200,
        headers: HashMap::new(),
        body: None,
    }
}
