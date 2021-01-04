pub(crate) mod admin;
pub(crate) mod public;

use crate::manager::{ReqCmd, ReqSender, RequestProcessorError};
use crate::server::admin::filters::admin_filters;
use crate::server::public::filters::public_filters;
use serde::Serialize;
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

pub(crate) fn with_sender(
    tx: ReqSender<ReqCmd>,
) -> impl Filter<Extract = (ReqSender<ReqCmd>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Debug)]
pub struct FhLockingError<T> {
    err: T,
}

impl<T> FhLockingError<T> {
    fn new(err: T) -> Self {
        Self { err }
    }
}

impl Reject for FhLockingError<String> {}

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

pub(crate) async fn web_server(tx: ReqSender<ReqCmd>) {
    let routes = public_filters(tx.clone())
        .or(admin_filters(tx.clone()))
        .recover(handle_rejections);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}
