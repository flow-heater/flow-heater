pub(crate) mod filters {
    use crate::manager::{ReqCmd, ReqSender};
    use crate::server::with_sender;
    use uuid::Uuid;
    use warp::Filter;

    pub(crate) fn admin_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        create_processor(tx.clone())
            .or(get_processor(tx.clone()))
            .or(update_processor(tx.clone()))
            .or(delete_processor(tx))
    }

    pub(crate) fn create_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("admin" / "processors")
            .and(with_sender(tx))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(super::handlers::create_processor)
    }

    pub(crate) fn get_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("admin" / "processors" / Uuid)
            .and(with_sender(tx))
            .and(warp::get())
            .and_then(super::handlers::get_processor)
    }

    pub(crate) fn update_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("admin" / "processors" / Uuid)
            .and(with_sender(tx))
            .and(warp::put())
            .and(warp::body::json())
            .and_then(super::handlers::update_processor)
    }

    pub(crate) fn delete_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("admin" / "processors" / Uuid)
            .and(with_sender(tx))
            .and(warp::delete())
            .and_then(super::handlers::delete_processor)
    }
}

pub(crate) mod handlers {
    use crate::manager::request_processor::RequestProcessor;
    use crate::manager::{ReqCmd, ReqSender};
    use tokio::sync::oneshot;
    use uuid::Uuid;

    pub(crate) async fn create_processor(
        tx: ReqSender<ReqCmd>,
        processor: RequestProcessor,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let mut tx2 = tx.lock().unwrap().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::CreateRequestProcessor {
            proc: processor.clone(),
            cmd_tx: resp_tx,
        })
        .await
        .unwrap();
        let _res = resp_rx.await.unwrap().unwrap();

        Ok(warp::reply::json(&processor))
    }

    pub(crate) async fn get_processor(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let mut tx2 = tx.lock().unwrap().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::GetRequestProcessor {
            id,
            cmd_tx: resp_tx,
        })
        .await
        .unwrap();
        let proc = resp_rx.await.unwrap().unwrap();

        Ok(warp::reply::json(&proc))
    }

    pub(crate) async fn update_processor(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
        processor: RequestProcessor,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let mut tx2 = tx.lock().unwrap().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::UpdateRequestProcessor {
            id,
            proc: processor.clone(),
            cmd_tx: resp_tx,
        })
        .await
        .unwrap();
        let _res = resp_rx.await.unwrap().unwrap();

        Ok(warp::reply::json(&processor))
    }

    pub(crate) async fn delete_processor(
        id: Uuid,
        tx: ReqSender<ReqCmd>,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let mut tx2 = tx.lock().unwrap().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::DeleteRequestProcessor {
            id,
            cmd_tx: resp_tx,
        })
        .await
        .unwrap();
        let _res = resp_rx.await.unwrap().unwrap();

        Ok(warp::reply())
    }
}
