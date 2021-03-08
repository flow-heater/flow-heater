//! HTTP Endpoints for the `/processor` path.

/// Wraps all warp Filters for the RequestProcessor endpoints.
pub(crate) mod filters {
    use crate::server::{util, AppContext};
    use uuid::Uuid;
    use warp::Filter;

    /// Convenient wrapper function which contains all filters.
    pub(crate) fn public_filters(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request_old(ctx)
            .or(run_request_processor(ctx))
            .or(run_request_processor_with_prelude(ctx))
    }

    /// Run a RequestProcessor by Id *without* prelude and sequel.
    ///
    /// - method: any
    /// - path: /processor/{processor_id}/run
    pub(crate) fn run_request_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run")
            .and(util::with_ctx(ctx.clone()))
            .and(util::with_prelude(false))
            .and(util::extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    /// Run a RequestProcessor by Id *with* prelude and sequel.
    ///
    /// - method: any
    /// - path: /processor/{processor_id}/run_with_prelude
    pub(crate) fn run_request_processor_with_prelude(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("processor" / Uuid / "run_with_prelude")
            .and(util::with_ctx(ctx.clone()))
            .and(util::with_prelude(true))
            .and(util::extract_request())
            .and_then(super::handlers::run_request_processor)
    }

    /// Run the code in the static file `fh_v8/src/flow_heater.js` with prelude
    /// and sequel.
    ///
    /// - method: any
    /// - path: /hello/{any_string}
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

    /// Run a RequestProcessor.
    #[tracing::instrument(
        name = "Executing a request processor.",
        skip(ctx),
        fields(
            request_id = %Uuid::new_v4(),
            processor_id = %id,
            %prelude,
        )
    )]
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
        tracing::info!("Request processor executed successfully.");

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

    /// Run the static RequestProcessor from `fh_v8/src/flow_heater.js`.
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
