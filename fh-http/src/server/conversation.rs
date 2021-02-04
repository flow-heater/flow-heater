//! HTTP Endpoints for the `/conversation` path.

/// Wraps all warp Filters for the RequestConversation endpoints.
pub(crate) mod filters {
    use crate::server::{util, AppContext};
    use uuid::Uuid;
    use warp::Filter;

    /// Convenient wrapper function which contains all filters.
    pub(crate) fn conversation_filters(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        get_request_conversation_audit_items(ctx).or(get_request_conversation(ctx))
    }

    /// Fetch a RequestConversation by Uuid.
    ///
    /// - method: GET
    /// - path: /conversation/{conversation_id}
    pub(crate) fn get_request_conversation(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("conversation" / Uuid)
            .and(util::with_ctx(ctx.clone()))
            .and(warp::get())
            .and_then(super::handlers::get_request_conversation)
    }

    /// Fetch a RequestConversations AuditItems by Conversation Uuid.
    ///
    /// - method: GET
    /// - path: /conversation/{conversation_id}/audit_item
    pub(crate) fn get_request_conversation_audit_items(
        ctx: &AppContext,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("conversation" / Uuid / "audit_item")
            .and(util::with_ctx(ctx.clone()))
            .and(warp::get())
            .and_then(super::handlers::get_request_conversation_audit_items)
    }
}

pub(crate) mod handlers {
    use crate::server::{error::FhHttpError, AppContext};
    use fh_core::FhLockingError;
    use fh_db::ReqCmd;
    use tokio::sync::oneshot;
    use uuid::Uuid;

    /// Gets a RequestConversation.
    pub(crate) async fn get_request_conversation(
        id: Uuid,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = db_cmd!(
            ctx,
            ReqCmd::GetRequestConversation { id, cmd_tx: cmd_tx },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }

    /// Gets a RequestConversation's AuditItems.
    pub(crate) async fn get_request_conversation_audit_items(
        id: Uuid,
        ctx: AppContext,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = db_cmd!(
            ctx,
            ReqCmd::GetRequestConversationAuditItems { id, cmd_tx: cmd_tx },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }
}
