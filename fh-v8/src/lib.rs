// use anyhow::Error;
// use deno_core::error::AnyError;
use deno_core::JsRuntime;
// use deno_core::OpState;
// use deno_core::ZeroCopyBuf;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Response {
    code: usize,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(headers: HashMap<String, String>, body: Option<Vec<u8>>) -> Self {
        Request { headers, body }
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
