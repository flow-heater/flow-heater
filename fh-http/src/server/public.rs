pub(crate) mod filters {
    use crate::manager::{ReqCmd, ReqSender};
    use crate::server::{with_sender, FhHttpError};
    use fh_v8::request::Request;
    use std::convert::TryFrom;
    use uuid::Uuid;
    use warp::{http, Filter, Rejection};

    pub(crate) fn public_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request_old(tx.clone()).or(run_request_processor(tx))
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

    fn extract_request() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Copy {
        warp::method()
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::body::bytes())
            .and_then(try_extract_request)
    }

    pub(crate) fn run_request_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run")
            .and(with_sender(tx.clone()))
            .and(extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn process_request_old(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String)
            .and(with_sender(tx.clone()))
            .and(extract_request())
            .and_then(super::handlers::process_request)
    }
}

pub(crate) mod handlers {
    use crate::{
        manager::{ReqCmd, ReqSender},
        server::{FhHttpError, FhLockingError},
    };
    use fh_v8::request::Request;
    use tokio::sync::oneshot;
    use uuid::Uuid;
    use warp::{Rejection, Reply};

    pub(crate) async fn run_request_processor(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let mut tx2 = tx
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        let (cmd_tx, cmd_rx) = oneshot::channel();
        tx2.send(ReqCmd::RunRequestProcessor {
            id,
            request,
            cmd_tx,
        })
        .await
        .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        let res = cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        Ok(warp::reply::json(&res))
    }

    pub(crate) async fn process_request(
        _name: String,
        tx: ReqSender<ReqCmd>,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let mut tx2 = tx
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::Http {
            request,
            cmd_tx: resp_tx,
        })
        .await
        .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        let res = resp_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        Ok(warp::reply::json(&res))
    }
}
