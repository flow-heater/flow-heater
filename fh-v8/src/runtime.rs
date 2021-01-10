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

struct RequestCounter(usize);

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
    let r = state.borrow::<Request>();
    Ok(serde_json::json!(r))
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

    let mut op_state = state.borrow_mut();
    let conversation_id = op_state.borrow::<Uuid>().clone();

    let x = op_state.borrow_mut::<ReqSender<ReqCmd>>();
    let mut tx_db2 = x
        .lock()
        .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
        .clone();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();
    tx_db2
        .send(ReqCmd::CreateAuditLogEntry {
            item: fh_db::request_conversation::AuditItem::new_log(conversation_id, log_entry),
            cmd_tx: cmd_tx2,
        })
        .await
        .map_err(anyhow::Error::new)?;

    // HINT: never omit awaiting here... this leads to runtime hangs!
    cmd_rx2.await??;

    Ok(serde_json::json!(()))
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

    let old_inc = op_state.borrow_mut::<RequestCounter>().increment();

    // TODO: check if we could have multiple Uuid's in the Gotham Store via Type Aliases
    // ```rust
    // type ConversationId = Uuid;
    // type RequestProcessorId = Uuid;
    // let conversation_id = op_state.borrow::<ConversationId>().clone();
    // let request_proc_id = op_state.borrow::<RequestProcessorId>().clone();
    // ```
    // A better alternative could actually be to offer one structure of our own which we
    // put into the GothamStore which we use for reading+writing.
    let conversation_id = op_state.borrow::<Uuid>().clone();

    let x = op_state.borrow_mut::<ReqSender<ReqCmd>>();
    let mut tx_db2 = x
        .lock()
        .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
        .clone();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();
    tx_db2
        .send(ReqCmd::CreateAuditLogEntry {
            item: fh_db::request_conversation::AuditItem::new_request(
                conversation_id,
                old_inc as i32,
                request_spec.request.clone(),
            ),
            cmd_tx: cmd_tx2,
        })
        .await
        .map_err(anyhow::Error::new)?;

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

    // HINT: never omit awaiting here... this leads to runtime hangs!
    cmd_rx2.await??;

    Ok(serde_json::json!(()))
}

pub(crate) fn prepare_runtime(
    tx_db: ReqSender<ReqCmd>,
    req: Request,
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
        .put::<Uuid>(conversation_id);
    js_runtime
        .op_state()
        .borrow_mut()
        .put::<ReqSender<ReqCmd>>(tx_db);
    js_runtime.op_state().borrow_mut().put::<Request>(req);
    js_runtime
        .op_state()
        .borrow_mut()
        .put::<RequestCounter>(RequestCounter(0));
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
