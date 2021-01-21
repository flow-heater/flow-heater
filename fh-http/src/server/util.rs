use crate::server::FhHttpError;
use fh_core::request::Request;
use std::convert::TryFrom;
use warp::{http, Filter, Rejection};

pub(crate) fn extract_request() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Copy
{
    warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(try_extract_request)
}

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
macro_rules! execute_command {
    ($tx_db: ident, $tx_proc: ident, $cmd: expr, $cmd_rx: ident) => {{
        let mut tx2 = $tx_proc
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
    ($tx_db: ident, $cmd: expr, $cmd_rx: ident) => {{
        let mut tx2 = $tx_db
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
