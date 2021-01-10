pub(crate) mod admin;
pub(crate) mod conversation;
pub(crate) mod public;

use crate::server::admin::filters::admin_filters;
use crate::server::conversation::filters::conversation_filters;
use crate::server::public::filters::public_filters;
use fh_core::ReqSender;
use fh_db::{ReqCmd, RequestProcessorError};
use fh_v8::ProcessorCmd;
use serde::Serialize;
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

pub(crate) fn with_sender<T: Sync + Send>(
    tx: ReqSender<T>,
) -> impl Filter<Extract = (ReqSender<T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

pub(crate) fn with_prelude(
    prelude: bool,
) -> impl Filter<Extract = (bool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || prelude)
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Debug)]
pub struct FhHttpError<T> {
    err: T,
}

impl<T> FhHttpError<T> {
    fn new(err: T) -> Self {
        Self { err }
    }
}

impl Reject for FhHttpError<RecvError> {}
impl Reject for FhHttpError<RequestProcessorError> {}
impl Reject for FhHttpError<SendError<ReqCmd>> {}
impl Reject for FhHttpError<SendError<ProcessorCmd>> {}
impl Reject for FhHttpError<anyhow::Error> {}

async fn handle_rejections(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(custom_error) = err.find::<FhHttpError<RecvError>>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = custom_error.err.to_string();
    } else if let Some(custom_error) = err.find::<FhHttpError<RequestProcessorError>>() {
        match custom_error.err {
            RequestProcessorError::NotFound { .. } => {
                code = StatusCode::NOT_FOUND;
                message = custom_error.err.to_string();
            }
            _ => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = custom_error.err.to_string();
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        // let cause = e.clone();
        message = e.to_string();
        code = StatusCode::BAD_REQUEST;
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_string();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message,
    });

    Ok(warp::reply::with_status(json, code))
}

pub(crate) async fn web_server(tx_db: ReqSender<ReqCmd>, tx_proc: ReqSender<ProcessorCmd>) {
    let routes = public_filters(tx_db.clone(), tx_proc.clone())
        .or(admin_filters(tx_db.clone()))
        .or(conversation_filters(tx_db.clone()))
        .recover(handle_rejections);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}
