use anyhow::Result;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use deno_core::{error::AnyError, BufVec};
use fh_core::{
    request::{Request, RequestList, RequestSpec},
    ReqSender,
};
use fh_db::{ReqCmd, RequestProcessorError};
use reqwest::{header, Method, Url};
use serde_json::Value;
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};
use tokio::sync::oneshot;
use uuid::Uuid;

pub(crate) struct RuntimeState {
    pub(crate) counter: RequestCounter,
    pub(crate) tx_db: ReqSender<ReqCmd>,
    pub(crate) request: Request,
    pub(crate) request_list: RequestList,
    pub(crate) conversation_id: Uuid,
}
pub(crate) struct RequestCounter(usize);

impl RequestCounter {
    fn increment(&mut self) -> usize {
        let old = self.0;
        self.0 += 1;
        old
    }
}

fn op_get_request(
    state: &mut OpState,
    _args: Value,
    _bufs: &mut [ZeroCopyBuf],
) -> Result<Value, AnyError> {
    let r = state.borrow::<RuntimeState>();
    Ok(serde_json::json!(r.request))
}

async fn op_log(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: BufVec,
) -> Result<Value, AnyError> {
    let log_entry = args
        .get("data")
        .ok_or(RequestProcessorError::Custom(
            "No 'data' attribute available for AuditEntry::Log".to_string(),
        ))?
        .to_string();

    let op_state = state.borrow();
    let rt_state = op_state.borrow::<RuntimeState>();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();
    execute_command!(
        rt_state.tx_db,
        ReqCmd::CreateAuditLogEntry {
            item: fh_db::request_conversation::AuditItem::new_log(
                rt_state.conversation_id,
                log_entry,
            ),
            cmd_tx: cmd_tx2,
        },
        cmd_rx2
    );

    Ok(serde_json::json!(()))
}
async fn op_dispatch_request(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: BufVec,
) -> Result<Value, AnyError> {
    let request_spec: RequestSpec = serde_json::from_value(args)?;

    let mut op_state = state.borrow_mut();

    let rt_state = op_state.borrow_mut::<RuntimeState>();
    rt_state.request_list.push(request_spec.request.clone());

    let (cmd_tx2, cmd_rx2) = oneshot::channel();
    execute_command!(
        rt_state.tx_db,
        ReqCmd::CreateAuditLogEntry {
            item: fh_db::request_conversation::AuditItem::new_request(
                rt_state.conversation_id,
                rt_state.counter.increment() as i32,
                request_spec.request.clone(),
            ),
            cmd_tx: cmd_tx2,
        },
        cmd_rx2
    );

    // TODO: this is the part, where we need to:
    // - convert the fh_v8::Request to a http::Request
    // - convert the http::Response to a fh_v8::Response
    // This is basically described in: https://github.com/flow-heater/fh-core/issues/25
    let c = reqwest::Client::builder().build()?;
    let _response = c
        .request(
            Method::from_str(&request_spec.request.method)?,
            Url::parse(&request_spec.url)?,
        )
        .body(request_spec.request.body)
        .header(header::ACCEPT, "application/json")
        .send()
        .await?;

    // TODO: this is the point, where also the AuditEntry::Response needs to be
    // created in the DB

    Ok(serde_json::json!(()))
}

pub(crate) fn prepare_runtime(
    tx_db: ReqSender<ReqCmd>,
    request: Request,
    conversation_id: Uuid,
) -> JsRuntime {
    let mut js_runtime = JsRuntime::new(Default::default());

    js_runtime.register_op(
        "dispatch_request",
        deno_core::json_op_async(op_dispatch_request),
    );
    js_runtime.register_op("fh_log", deno_core::json_op_async(op_log));
    js_runtime.register_op("get_request", deno_core::json_op_sync(op_get_request));
    js_runtime.register_op(
        "op_resources",
        deno_core::json_op_sync(deno_core::op_resources),
    );

    js_runtime
        .op_state()
        .borrow_mut()
        .put::<RuntimeState>(RuntimeState {
            counter: RequestCounter(0),
            tx_db,
            request,
            request_list: RequestList { inner: Vec::new() },
            conversation_id,
        });
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
