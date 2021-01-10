pub(crate) mod filters {
    use crate::server::{with_prelude, with_sender, FhHttpError};
    use fh_core::{request::Request, ReqSender};
    use fh_db::ReqCmd;
    use fh_v8::ProcessorCmd;
    use std::convert::TryFrom;
    use uuid::Uuid;
    use warp::{http, Filter, Rejection};

    pub(crate) fn public_filters(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request_old(tx_db.clone(), tx_proc.clone())
            .or(run_request_processor(tx_db.clone(), tx_proc.clone()))
            .or(run_request_processor_with_prelude(tx_db, tx_proc))
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
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run")
            .and(with_sender(tx_db.clone()))
            .and(with_sender(tx_proc.clone()))
            .and(with_prelude(false))
            .and(extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn run_request_processor_with_prelude(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run_with_prelude")
            .and(with_sender(tx_db.clone()))
            .and(with_sender(tx_proc.clone()))
            .and(with_prelude(true))
            .and(extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn process_request_old(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String)
            .and(with_sender(tx_db.clone()))
            .and(with_sender(tx_proc.clone()))
            .and(extract_request())
            .and_then(super::handlers::process_request)
    }
}

pub(crate) mod handlers {
    use crate::server::FhHttpError;
    use fh_core::{request::Request, FhLockingError, ReqSender};
    use fh_db::ReqCmd;
    use fh_v8::ProcessorCmd;
    use tokio::sync::oneshot;
    use uuid::Uuid;
    use warp::{Rejection, Reply};

    pub(crate) async fn run_request_processor(
        id: Uuid,
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
        prelude: bool,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let mut tx2 = tx_proc
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        let (cmd_tx, cmd_rx) = oneshot::channel();
        tx2.send(ProcessorCmd::RunRequestProcessor {
            id,
            request,
            cmd_tx,
            tx_db,
            prelude,
        })
        .await
        .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        let res = cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        Ok(warp::reply::with_header(
            warp::reply::json(&res),
            "FH-Conversation-Id",
            res.headers
                .get("FH-Conversation-Id")
                .ok_or(warp::reject::custom(FhHttpError::new(anyhow::Error::msg(
                    "Missing response header 'FH-Conversation-Id'.",
                ))))?,
        ))
    }

    pub(crate) async fn process_request(
        _name: String,
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let mut tx2 = tx_proc
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ProcessorCmd::Http {
            request,
            cmd_tx: resp_tx,
            tx_db,
        })
        .await
        .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        let res = resp_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        Ok(warp::reply::with_header(
            warp::reply::json(&res),
            "FH-Conversation-Id",
            res.headers
                .get("FH-Conversation-Id")
                .ok_or(warp::reject::custom(FhHttpError::new(anyhow::Error::msg(
                    "Missing response header 'FH-Conversation-Id'.",
                ))))?,
        ))
    }
}
