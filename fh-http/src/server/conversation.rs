pub(crate) mod filters {
    use crate::server::with_sender;
    use fh_core::ReqSender;
    use fh_db::ReqCmd;
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn conversation_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        get_request_conversation_audit_items(tx.clone()).or(get_request_conversation(tx))
    }

    pub(crate) fn get_request_conversation(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("conversation" / Uuid)
            .and(with_sender(tx))
            .and(warp::get())
            .and_then(super::handlers::get_request_conversation)
    }

    pub(crate) fn get_request_conversation_audit_items(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("conversation" / Uuid / "audit_item")
            .and(with_sender(tx))
            .and(warp::get())
            .and_then(super::handlers::get_request_conversation_audit_items)
    }
}

pub(crate) mod handlers {
    use crate::server::FhHttpError;
    use fh_core::{FhLockingError, ReqSender};
    use fh_db::ReqCmd;
    use tokio::sync::oneshot;
    use uuid::Uuid;

    pub(crate) async fn get_request_conversation(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = execute_command!(
            tx,
            ReqCmd::GetRequestConversation { id, cmd_tx: cmd_tx },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }

    pub(crate) async fn get_request_conversation_audit_items(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (cmd_tx, cmd_rx) = oneshot::channel();
        let res = execute_command!(
            tx,
            ReqCmd::GetRequestConversationAuditItems { id, cmd_tx: cmd_tx },
            cmd_rx
        );

        Ok(warp::reply::json(&res))
    }
}
