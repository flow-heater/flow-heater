pub(crate) mod filters {
    use crate::server::with_sender;
    use fh_core::ReqSender;
    use fh_db::ReqCmd;
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn conversation_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        get_audit_items(tx.clone())
    }

    pub(crate) fn get_audit_items(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("conversation" / Uuid / "items")
            .and(with_sender(tx))
            .and(warp::get())
            .and_then(super::handlers::get_audit_items)
    }
}

pub(crate) mod handlers {
    use crate::server::FhHttpError;
    use fh_core::{FhLockingError, ReqSender};
    use fh_db::ReqCmd;
    use tokio::sync::oneshot;
    use uuid::Uuid;

    pub(crate) async fn get_audit_items(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut tx2 = tx
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::GetConversationItems {
            id,
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
