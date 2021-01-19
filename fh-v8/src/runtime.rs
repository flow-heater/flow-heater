use anyhow::Result;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use deno_core::{error::AnyError, BufVec};
use fh_core::{
    request::{Request, RequestResponseList, RequestSpec},
    response::Response,
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
    pub(crate) request_audit_id: Uuid,
    pub(crate) request_list: RequestResponseList,
    pub(crate) conversation_id: Uuid,
    pub(crate) final_response: Option<Response>,
}

impl RuntimeState {
    async fn new(
        request: Request,
        tx_db: ReqSender<ReqCmd>,
        conversation_id: Uuid,
    ) -> anyhow::Result<Self> {
        let (cmd_tx2, cmd_rx2) = oneshot::channel();
        let req_audit_item = execute_command!(
            tx_db,
            ReqCmd::CreateAuditLogEntry {
                item: fh_db::request_conversation::AuditItem::new_request(
                    conversation_id,
                    0,
                    request.clone(),
                ),
                cmd_tx: cmd_tx2,
            },
            cmd_rx2
        );

        Ok(Self {
            counter: RequestCounter(0),
            conversation_id,
            final_response: None,
            request,
            request_list: RequestResponseList::new(),
            tx_db,
            request_audit_id: req_audit_item.get_id(),
        })
    }

    async fn add_request(&mut self, request: Request) -> anyhow::Result<usize> {
        let inc = self.counter.increment();
        self.request_list.add_request(inc, request.clone());

        let (cmd_tx2, cmd_rx2) = oneshot::channel();
        execute_command!(
            self.tx_db,
            ReqCmd::CreateAuditLogEntry {
                item: fh_db::request_conversation::AuditItem::new_request(
                    self.conversation_id,
                    inc as i32,
                    request.clone(),
                ),
                cmd_tx: cmd_tx2,
            },
            cmd_rx2
        );
        Ok(inc)
    }

    async fn add_response(&mut self, idx: usize, response: Response) -> anyhow::Result<()> {
        self.request_list.add_response(idx, response.clone());

        let (cmd_tx2, cmd_rx2) = oneshot::channel();
        execute_command!(
            self.tx_db,
            ReqCmd::CreateAuditLogEntry {
                item: fh_db::request_conversation::AuditItem::new_response(
                    self.conversation_id,
                    self.request_audit_id,
                    response,
                ),
                cmd_tx: cmd_tx2,
            },
            cmd_rx2
        );

        Ok(())
    }

    async fn add_final_response(&mut self, response: Response) -> anyhow::Result<()> {
        self.final_response = Some(response.clone());

        let (cmd_tx2, cmd_rx2) = oneshot::channel();
        execute_command!(
            self.tx_db,
            ReqCmd::CreateAuditLogEntry {
                item: fh_db::request_conversation::AuditItem::new_response(
                    self.conversation_id,
                    self.request_audit_id,
                    response,
                ),
                cmd_tx: cmd_tx2,
            },
            cmd_rx2
        );

        Ok(())
    }

    async fn add_log_entry(&mut self, log: String) -> anyhow::Result<()> {
        let (cmd_tx2, cmd_rx2) = oneshot::channel();
        execute_command!(
            self.tx_db,
            ReqCmd::CreateAuditLogEntry {
                item: fh_db::request_conversation::AuditItem::new_log(self.conversation_id, log,),
                cmd_tx: cmd_tx2,
            },
            cmd_rx2
        );

        Ok(())
    }

    pub fn get_final_response_body(&self) -> anyhow::Result<String> {
        if self.final_response.is_some() {
            return Ok(self
                .final_response
                .clone()
                .unwrap()
                .body
                .unwrap_or("".to_string()));
        }

        if self.request_list.get_last_response_body().is_some() {
            return Ok(self
                .request_list
                .get_last_response_body()
                .unwrap_or("".to_string()));
        }

        // fallback: return the initial requests body
        Ok(self.request.body.clone())
    }
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

async fn op_respond_with(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: BufVec,
) -> Result<Value, AnyError> {
    let response: Response = serde_json::from_value(args)?;
    let mut op_state = state.borrow_mut();
    let rt_state = op_state.borrow_mut::<RuntimeState>();

    rt_state.add_final_response(response).await?;

    Ok(serde_json::json!(()))
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
    let rt_state = op_state.borrow_mut::<RuntimeState>();
    rt_state.add_log_entry(log_entry).await?;

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

    let inc = rt_state.add_request(request_spec.request.clone()).await?;

    let c = reqwest::Client::builder().build()?;
    let response: reqwest::Response = c
        .request(
            Method::from_str(&request_spec.request.method)?,
            Url::parse(&request_spec.url)?,
        )
        .body(request_spec.request.body)
        .header(header::ACCEPT, "application/json")
        .send()
        .await?;

    let r = Response::try_from_response(response).await?;

    rt_state.add_response(inc, r.clone()).await?;

    Ok(serde_json::json!(r))
}

pub(crate) async fn prepare_runtime(
    tx_db: ReqSender<ReqCmd>,
    request: Request,
    conversation_id: Uuid,
) -> anyhow::Result<JsRuntime> {
    let mut js_runtime = JsRuntime::new(Default::default());

    js_runtime.register_op(
        "dispatch_request",
        deno_core::json_op_async(op_dispatch_request),
    );
    js_runtime.register_op("fh_log", deno_core::json_op_async(op_log));
    js_runtime.register_op("respond_with", deno_core::json_op_async(op_respond_with));
    js_runtime.register_op("get_request", deno_core::json_op_sync(op_get_request));
    js_runtime.register_op(
        "op_resources",
        deno_core::json_op_sync(deno_core::op_resources),
    );

    js_runtime
        .op_state()
        .borrow_mut()
        .put::<RuntimeState>(RuntimeState::new(request, tx_db, conversation_id).await?);

    Ok(js_runtime)
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
