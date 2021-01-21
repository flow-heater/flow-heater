pub(crate) mod filters {
    use crate::server::{util, with_prelude, with_sender};
    use fh_core::ReqSender;
    use fh_db::ReqCmd;
    use fh_v8::ProcessorCmd;
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn public_filters(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request_old(tx_db.clone(), tx_proc.clone())
            .or(run_request_processor(tx_db.clone(), tx_proc.clone()))
            .or(run_request_processor_with_prelude(tx_db, tx_proc))
    }

    pub(crate) fn run_request_processor(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run")
            .and(with_sender(tx_db.clone()))
            .and(with_sender(tx_proc.clone()))
            .and(with_prelude(false))
            .and(util::extract_request())
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
            .and(util::extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn process_request_old(
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String)
            .and(with_sender(tx_db.clone()))
            .and(with_sender(tx_proc.clone()))
            .and(util::extract_request())
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
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = execute_command!(
            tx_db,
            tx_proc,
            ProcessorCmd::RunRequestProcessor {
                id,
                request,
                cmd_tx,
                tx_db,
                prelude,
            },
            cmd_rx
        );

        Ok(warp::reply::with_header(
            warp::reply::json(&res),
            "FH-Conversation-Id",
            res.headers
                .get("FH-Conversation-Id")
                .ok_or(warp::reject::custom(FhHttpError::new(anyhow::Error::msg(
                    "Missing response header 'FH-Conversation-Id'.",
                ))))?[0]
                .clone(),
        ))
    }

    pub(crate) async fn process_request(
        _name: String,
        tx_db: ReqSender<ReqCmd>,
        tx_proc: ReqSender<ProcessorCmd>,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let res = execute_command!(
            tx_db,
            tx_proc,
            ProcessorCmd::Http {
                request,
                cmd_tx: resp_tx,
                tx_db,
            },
            resp_rx
        );

        Ok(warp::reply::with_header(
            warp::reply::json(&res),
            "FH-Conversation-Id",
            res.headers
                .get("FH-Conversation-Id")
                .ok_or(warp::reject::custom(FhHttpError::new(anyhow::Error::msg(
                    "Missing response header 'FH-Conversation-Id'.",
                ))))?[0]
                .clone(),
        ))
    }
}
