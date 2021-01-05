use anyhow::Result;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use deno_core::{error::AnyError, BufVec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use warp::http;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Response {
    pub fn error_msg(reason: &str, r: Self) -> anyhow::Error {
        anyhow::Error::msg(format!("{}: {:?}", reason, r))
    }
}

fn op_get_request(
    state: &mut OpState,
    _args: Value,
    _bufs: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let r = state.borrow::<Request>();
    Ok(serde_json::json!(r))
}

#[derive(Debug)]
struct RequestList {
    inner: Vec<Request>,
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

async fn op_dispatch_request(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: BufVec,
) -> Result<Value, AnyError> {
    let r: Request = serde_json::from_value(args)?;
    let mut op_state = state.borrow_mut();
    op_state.borrow_mut::<RequestList>().push(r);

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

fn prepare_runtime(req: Request) -> JsRuntime {
    let mut js_runtime = JsRuntime::new(Default::default());

    js_runtime.register_op(
        "dispatch_request",
        deno_core::json_op_async(op_dispatch_request),
    );

    js_runtime.register_op("get_request", deno_core::json_op_sync(op_get_request));

    js_runtime.register_op(
        "op_resources",
        deno_core::json_op_sync(deno_core::op_resources),
    );

    js_runtime.op_state().borrow_mut().put::<Request>(req);
    js_runtime
        .op_state()
        .borrow_mut()
        .put::<RequestList>(RequestList { inner: Vec::new() });

    js_runtime
}

pub async fn process_request(req: Request, code: Option<String>) -> Result<Response> {
    let mut js_runtime = prepare_runtime(req.clone());

    if let Some(c) = code {
        js_runtime.execute("custom_code.js", &c)?;
    } else {
        js_runtime.execute("flow_heater.js", include_str!("flow_heater.js"))?;
    }

    js_runtime.run_event_loop().await?;

    // extract the requests
    let state = js_runtime.op_state();
    let op_state = state.borrow();
    let requests = op_state.borrow::<RequestList>();

    // println!("Requests: {:?}", requests);

    Ok(Response {
        code: 200,
        headers: HashMap::new(),
        body: Some(
            requests
                .iter()
                .last()
                .cloned()
                .unwrap_or(req)
                .body
                .as_bytes()
                .to_vec(),
        ),
    })
}
