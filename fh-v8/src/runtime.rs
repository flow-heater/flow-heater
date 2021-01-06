use anyhow::Result;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use deno_core::{error::AnyError, BufVec};
use fh_core::request::{Request, RequestList, RequestSpec};
use reqwest::{header, Method, Url};
use serde_json::Value;
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};

fn op_get_request(
    state: &mut OpState,
    _args: Value,
    _bufs: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let r = state.borrow::<Request>();
    Ok(serde_json::json!(r))
}

async fn op_dispatch_request(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: BufVec,
) -> Result<Value, AnyError> {
    let request_spec: RequestSpec = serde_json::from_value(args)?;
    let mut op_state = state.borrow_mut();
    op_state
        .borrow_mut::<RequestList>()
        .push(request_spec.request.clone());

    // TODO: this is the part, where we need to:
    // - convert the fh_v8::Request to a http::Request
    // - convert the http::Response to a fh_v8::Response
    // This is basically described in: https://github.com/flow-heater/fh-core/issues/25
    let c = reqwest::Client::builder().build()?;
    let response = c
        .request(
            Method::from_str(&request_spec.request.method)?,
            Url::parse(&request_spec.url)?,
        )
        .body(request_spec.request.body)
        .header(header::ACCEPT, "application/json")
        .send()
        .await?;

    println!("{:?}", response);

    Ok(serde_json::json!(()))
}

pub(crate) fn prepare_runtime(req: Request) -> JsRuntime {
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

pub(crate) fn prepare_user_code(code: &str, wrap_prelude: bool) -> String {
    let mut final_code: String = "".to_string();

    if wrap_prelude {
        final_code.push_str(include_str!("fh_prelude.js"));
        final_code.push_str(&code);
        final_code.push_str(include_str!("fh_sequel.js"));
    } else {
        final_code.push_str(&code);
    }

    final_code
}
