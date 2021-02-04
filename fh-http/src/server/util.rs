use crate::server::error::FhHttpError;
use fh_core::request::Request;
use std::convert::TryFrom;
use warp::{http, Filter, Rejection};

use super::AppContext;

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
        let mut tx2 = $ctx
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
        let mut tx2 = $ctx
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
