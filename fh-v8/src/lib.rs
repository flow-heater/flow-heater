// use anyhow::Error;
use deno_core::error::AnyError;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use warp::http;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    headers: HashMap<String, String>,
    body: String,
    method: String,
    path: String,
    query: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    code: usize,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

fn op_get_request(
    state: &mut OpState,
    _args: Value,
    _bufs: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let r = state.borrow::<Request>();
    Ok(serde_json::json!(r))
}

fn op_dispatch_request(
    state: &mut OpState,
    args: Value,
    _bufs: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let r: Request = serde_json::from_value(args).unwrap();
    *state.borrow_mut::<Request>() = r;
    Ok(serde_json::json!(()))
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

pub async fn process_request(req: Request) -> Response {
    let mut js_runtime = JsRuntime::new(Default::default());

    js_runtime.register_op(
        "dispatch_request",
        deno_core::json_op_sync(op_dispatch_request),
    );

    js_runtime.register_op("get_request", deno_core::json_op_sync(op_get_request));

    js_runtime.register_op(
        "op_resources",
        deno_core::json_op_sync(deno_core::op_resources),
    );

    js_runtime.op_state().borrow_mut().put::<Request>(req);

    js_runtime
        .execute("flow_heater.js", include_str!("flow_heater.js"))
        .unwrap();

    js_runtime.run_event_loop().await.unwrap();

    // extract the request
    let state = js_runtime.op_state();
    let op_state = state.borrow();
    let modified_req = op_state.borrow::<Request>();
    println!("RUST: modified request is: {:?}", modified_req);

    Response {
        code: 200,
        headers: HashMap::new(),
        body: Some(modified_req.body.as_bytes().to_vec()),
    }
}
