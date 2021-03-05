//! HTTP Endpoints for the `/admin` path.

/// Wraps all warp Filters for the admin endpoints.
pub(crate) mod filters {
    use crate::server::{util, AppContext};
    use uuid::Uuid;
    use warp::Filter;

    /// Convenient wrapper function which contains all filters.
    pub(crate) fn admin_filters(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        create_processor(ctx)
            .or(get_processors(ctx))
            .or(get_processor(ctx))
            .or(update_processor(ctx))
            .or(delete_processor(ctx))
    }

    /// Create a RequestProcessor.
    ///
    /// - method: POST
    /// - path: /admin/processor
    pub fn create_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor")
            .and(warp::post())
            .and(warp::header("fh-user-id"))
            .and(warp::body::json())
            .and(util::with_ctx(ctx.clone()))
            .and_then(super::handlers::create_processor)
    }

    /// Fetch all RequestProcessors by of a User.
    ///
    /// - method: GET
    /// - path: /admin/processors
    ///
    /// TODO: If path "/admin/processor" is used, somehow it overlays the
    /// existing path for POST /admin/processor and makes it not function
    /// anymore as before and I do not know why. I even found examples from
    /// warp, using this "feature" e.g.
    /// https://github.com/seanmonstar/warp/blob/master/examples/todos.rs#L52 ->
    /// GET
    /// https://github.com/seanmonstar/warp/blob/master/examples/todos.rs#L63 ->
    /// POST
    /// but I don't want to debug that right now, hence the path difference.
    pub fn get_processors(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processors")
            .and(warp::get())
            .and(warp::header("fh-user-id"))
            .and(util::with_ctx(ctx.clone()))
            .and_then(super::handlers::get_processors)
    }

    /// Fetch a RequestProcessor by Uuid.
    ///
    /// - method: GET
    /// - path: /admin/processor/{processor_id}
    pub fn get_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(warp::get())
            .and(warp::header("fh-user-id"))
            .and(util::with_ctx(ctx.clone()))
            .and_then(super::handlers::get_processor)
    }

    /// Update a RequestProcessor by Uuid and the given data.
    ///
    /// - method: PUT
    /// - path: /admin/processor/{processor_id}
    pub fn update_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(warp::put())
            .and(warp::header("fh-user-id"))
            .and(warp::body::json())
            .and(util::with_ctx(ctx.clone()))
            .and_then(super::handlers::update_processor)
    }

    /// Delete a RequestProcessor by Uuid.
    ///
    /// - method: DELETE
    /// - path: /admin/processor/{processor_id}
    pub fn delete_processor(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processor" / Uuid)
            .and(warp::delete())
            .and(warp::header("fh-user-id"))
            .and(util::with_ctx(ctx.clone()))
            .and_then(super::handlers::delete_processor)
    }
}

pub(crate) mod handlers {
    use crate::server::{error::FhHttpError, AppContext};
    use fh_core::FhLockingError;
    use fh_db::{request_processor::RequestProcessor, ReqCmd};
    use tokio::sync::oneshot;
    use uuid::Uuid;

    /// Creates a RequestProcessor
    pub(crate) async fn create_processor(
        user_id: String,
        mut processor: RequestProcessor,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        processor.user_id = user_id;
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

    /// Fetches a RequestProcessor.
    pub(crate) async fn get_processor(
        id: Uuid,
        _user_id: String,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let proc = db_cmd!(ctx, ReqCmd::GetRequestProcessor { id, cmd_tx }, cmd_rx);

        Ok(warp::reply::json(&proc))
    }

    /// Fetches all RequestProcessors for a user.
    pub(crate) async fn get_processors(
        user_id: String,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let proc = db_cmd!(
            ctx,
            ReqCmd::GetRequestProcessors { user_id, cmd_tx },
            cmd_rx
        );

        Ok(warp::reply::json(&proc))
    }

    /// Updates a RequestProcessor.
    pub(crate) async fn update_processor(
        id: Uuid,
        _user_id: String,
        processor: RequestProcessor,
        ctx: AppContext,
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

    /// Deletes a RequestProcessor.
    pub(crate) async fn delete_processor(
        id: Uuid,
        _user_id: String,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        db_cmd!(ctx, ReqCmd::DeleteRequestProcessor { id, cmd_tx }, cmd_rx);

        Ok(warp::reply())
    }
}
