use anyhow::Result;
use deno_core::error::AnyError;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
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

/// Wrapper type for all data, which is stored in the JsRuntime.
pub(crate) struct RuntimeState {
    /// Simple counter to count all incoming + issued requests.
    pub(crate) counter: RequestCounter,

    /// Transmitter to transmit `ReqCmd` commands to the `fh_db` crate.
    pub(crate) tx_db: ReqSender<ReqCmd>,

    /// Incoming request.
    pub(crate) request: Request,

    /// Uuid of the incoming requests [`fh_db::request_conversation::AuditItem`].
    pub(crate) request_audit_id: Uuid,

    /// Contains all requests + responses.
    pub(crate) request_list: RequestResponseList,

    /// Id of the current conversation.
    pub(crate) conversation_id: Uuid,

    /// Optional final response.
    pub(crate) final_response: Option<Response>,
}

impl RuntimeState {
    /// Creates a new RuntimeState.
    ///
    /// Implicitly creates an AuditItem for the incoming request.
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

    /// Adds an issued Request.
    ///
    /// Implicitly creates an AuditItem for the issued request.
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

    /// Adds a received Response.
    ///
    /// Implicitly creates an AuditItem for the issued response.
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

    /// Adds the final response.
    async fn add_final_response(&mut self, response: Response) -> anyhow::Result<()> {
        self.final_response = Some(response.clone());
        Ok(())
    }

    /// Adds a log entry by creating a AuditItem::Log.
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

    /// Computes the final response body.
    ///
    /// The computation goes like this:
    /// 1. If a `final_response` was explicitly set with `respond_with`, take this responses body.
    /// 2. If not, take the last response body from the internal `RequestResponseList`.
    /// 3. If there was no response, yet then echo the incoming requests body.
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

/// Simple wrapper type for a Counter
pub(crate) struct RequestCounter(usize);

impl RequestCounter {
    /// Increments the counters value by one and returns the old value.
    fn increment(&mut self) -> usize {
        let old = self.0;
        self.0 += 1;
        old
    }
}

/// Represents the `get_request` function, which can be called from the JsRuntime
/// using `Deno.core.opSync("get_request")`.
/// Returns a JSON representation of the incoming HTTP Request of the the [`fh_core::request::Request`].
fn op_get_request(
    state: &mut OpState,
    _args: Value,
    _bufs: Option<ZeroCopyBuf>,
) -> Result<Value, AnyError> {
    let r = state.borrow::<RuntimeState>();
    Ok(serde_json::json!(r.request))
}

/// Adds a final response, which shall be returned to the client. Represents the `respond_with` function, which can be called from the JsRuntime
/// using `Deno.core.opAsync("respond_with", response)`.
/// The response object is a JSON representation of [`fh_core::response::Response`].
async fn op_respond_with(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: Option<ZeroCopyBuf>,
) -> Result<Value, AnyError> {
    let response: Response = serde_json::from_value(args)?;
    let mut op_state = state.borrow_mut();
    let rt_state = op_state.borrow_mut::<RuntimeState>();

    rt_state.add_final_response(response).await?;

    Ok(serde_json::json!(()))
}

/// Represents the `fh_log` function, which can be called from the JsRuntime
/// using `Deno.core.opAsync("fh_log", spec)`.
/// The `spec` object has one key:
/// - data: String to be logged.
///
/// The data is actually printed to stdout by the JsRuntime and additionally stored to the db
/// as a [`fh_db::request_conversation::AuditItem::Log`].
async fn op_log(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: Option<ZeroCopyBuf>,
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

/// Represents the `dispatch_request` function, which can be called from the
/// JsRuntime using `Deno.core.opAsync("dispatch_request", spec)`. The
/// `spec` object has two keys:
/// - request: regular request object, based on [`fh_core::request::Request`]
/// - url: fully qualified URL, where the request should be sent to.
///
/// The request is stored as a
/// [`fh_db::request_conversation::AuditItem::Request`] to the database. Then
/// the request is converted to a `reqwest::Request` and executed. The returned
/// `reqwest::Response` is converted to a [`fh_core::response::Response`] and
/// then stored in the database as
/// [`fh_db::request_conversation::AuditItem::Response`] with the requests Uuid
/// to match be able to match a Request / Response pair later on.
async fn op_dispatch_request(
    state: Rc<RefCell<OpState>>,
    args: Value,
    _bufs: Option<ZeroCopyBuf>,
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

/// Registers all custom operations and the [`RuntimeState`] and returns the final prepared [`JsRuntime`].
pub(crate) async fn prepare_runtime(
    tx_db: ReqSender<ReqCmd>,
    request: Request,
    conversation_id: Uuid,
) -> anyhow::Result<JsRuntime> {
    let mut js_runtime = JsRuntime::new(Default::default());

    js_runtime.register_op("dispatch_request", deno_core::op_async(op_dispatch_request));
    js_runtime.register_op("fh_log", deno_core::op_async(op_log));
    js_runtime.register_op("respond_with", deno_core::op_async(op_respond_with));
    js_runtime.register_op("get_request", deno_core::op_sync(op_get_request));

    js_runtime
        .op_state()
        .borrow_mut()
        .put::<RuntimeState>(RuntimeState::new(request, tx_db, conversation_id).await?);

    Ok(js_runtime)
}

/// Prepares the final code to be executed. If `wrap_prelude` is true, the given
/// code string is wrapped with prelude and sequel, from `fh_prelude.js` and
/// `fh_sequel.js`.
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
