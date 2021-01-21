pub(crate) mod filters {
    use crate::server::{util, AppContext};
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn admin_filters(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        create_processor(ctx)
            .or(get_processor(ctx))
            .or(update_processor(ctx))
            .or(delete_processor(ctx))
    }

    pub(crate) fn create_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor")
            .and(util::with_ctx(ctx.clone()))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(super::handlers::create_processor)
    }

    pub(crate) fn get_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(util::with_ctx(ctx.clone()))
            .and(warp::get())
            .and_then(super::handlers::get_processor)
    }

    pub(crate) fn update_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(util::with_ctx(ctx.clone()))
            .and(warp::put())
            .and(warp::body::json())
            .and_then(super::handlers::update_processor)
    }

    pub(crate) fn delete_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(util::with_ctx(ctx.clone()))
            .and(warp::delete())
            .and_then(super::handlers::delete_processor)
    }
}

pub(crate) mod handlers {
    use crate::server::{error::FhHttpError, AppContext};
    use fh_core::FhLockingError;
    use fh_db::{request_processor::RequestProcessor, ReqCmd};
    use tokio::sync::oneshot;
    use uuid::Uuid;

    pub(crate) async fn create_processor(
        ctx: AppContext,
        processor: RequestProcessor,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = db_cmd!(
            ctx,
            ReqCmd::CreateRequestProcessor {
                proc: processor.clone(),
                cmd_tx,
            },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }

    pub(crate) async fn get_processor(
        id: Uuid,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let proc = db_cmd!(ctx, ReqCmd::GetRequestProcessor { id, cmd_tx }, cmd_rx);

        Ok(warp::reply::json(&proc))
    }

    pub(crate) async fn update_processor(
        id: Uuid,
        ctx: AppContext,
        processor: RequestProcessor,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = db_cmd!(
            ctx,
            ReqCmd::UpdateRequestProcessor {
                id,
                proc: processor.clone(),
                cmd_tx,
            },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }

    pub(crate) async fn delete_processor(
        id: Uuid,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        db_cmd!(ctx, ReqCmd::DeleteRequestProcessor { id, cmd_tx }, cmd_rx);

        Ok(warp::reply())
    }
}
