use crate::server::error::FhHttpError;
use fh_core::request::Request;
use std::convert::TryFrom;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Registry};
use warp::{http, Filter, Rejection};

use super::AppContext;

pub(crate) fn _get_json_subscriber(
    name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let json_formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(json_formatting_layer)
}

pub(crate) fn get_standard_subscriber(
    _name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let standard_formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_span_events(FmtSpan::CLOSE);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(standard_formatting_layer)
}

pub(crate) fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Warp filter which wraps the [`AppContext`].
pub(crate) fn with_ctx(
    ctx: AppContext,
) -> impl Filter<Extract = (AppContext,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}

/// Warp filter which wraps the prelude boolean parameter.
pub(crate) fn with_prelude(
    prelude: bool,
) -> impl Filter<Extract = (bool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || prelude)
}

/// Warp filter which extracts the full http request data.
pub(crate) fn extract_request() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Copy
{
    warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(try_extract_request)
}

/// Helpe function which tries to extract a [`fh_core::request::Request`] from
/// warps request data.
async fn try_extract_request(
    method: http::Method,
    path: warp::path::FullPath,
    headers: http::HeaderMap,
    body: warp::hyper::body::Bytes,
) -> Result<Request, Rejection> {
    let mut req = http::Request::builder()
        .method(method)
        .uri(path.as_str())
        .body(body.iter().cloned().collect::<Vec<u8>>())
        .expect("request builder");
    {
        *req.headers_mut() = headers;
    }

    Request::try_from(req).map_err(|e| warp::reject::custom(FhHttpError::new(e)))
}

#[macro_export]
macro_rules! proc_cmd {
    ($ctx: expr, $cmd: expr, $cmd_rx: ident) => {{
        let tx2 = $ctx
            .tx_proc
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        tx2.send($cmd)
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        $cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
    }};
}

#[macro_export]
macro_rules! db_cmd {
    ($ctx: expr, $cmd: expr, $cmd_rx: ident) => {{
        let tx2 = $ctx
            .tx_db
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        tx2.send($cmd)
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        $cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
    }};
}
