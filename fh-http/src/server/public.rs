pub(crate) mod filters {
    use crate::server::{util, AppContext};
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn public_filters(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request_old(ctx)
            .or(run_request_processor(ctx))
            .or(run_request_processor_with_prelude(ctx))
    }

    pub(crate) fn run_request_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run")
            .and(util::with_ctx(ctx.clone()))
            .and(util::with_prelude(false))
            .and(util::extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn run_request_processor_with_prelude(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run_with_prelude")
            .and(util::with_ctx(ctx.clone()))
            .and(util::with_prelude(true))
            .and(util::extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    pub(crate) fn process_request_old(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String)
            .and(util::with_ctx(ctx.clone()))
            .and(util::extract_request())
            .and_then(super::handlers::process_request)
    }
}

pub(crate) mod handlers {
    use crate::server::{error::FhHttpError, AppContext};
    use fh_core::{request::Request, FhLockingError};
    use fh_v8::ProcessorCmd;
    use tokio::sync::oneshot;
    use uuid::Uuid;
    use warp::{Rejection, Reply};

    pub(crate) async fn run_request_processor(
        id: Uuid,
        ctx: AppContext,
        prelude: bool,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = proc_cmd!(
            ctx,
            ProcessorCmd::RunRequestProcessor {
                id,
                request,
                cmd_tx,
                tx_db: ctx.tx_db,
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
        ctx: AppContext,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let res = proc_cmd!(
            ctx,
            ProcessorCmd::Http {
                request,
                cmd_tx: resp_tx,
                tx_db: ctx.tx_db,
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
