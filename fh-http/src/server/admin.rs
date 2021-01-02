use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProcessorReq {
    pub(crate) name: String,
    pub(crate) language: String, //TODO: replace with Enum
    pub(crate) runtime: String,  //TODO: replace with Enum
    pub(crate) code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RequestProcessorResp {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) language: String, //TODO: replace with Enum
    pub(crate) runtime: String,  //TODO: replace with Enum
    pub(crate) code: String,
}

pub(crate) mod filters {
    use crate::manager::{ReqCmd, ReqSender};
    use crate::server::with_sender;
    use warp::Filter;

    pub(crate) fn admin_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        create_processor(tx.clone())
    }

    pub(crate) fn create_processor(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "processors")
            .and(with_sender(tx))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(super::handlers::create_processor)
    }
}

pub(crate) mod handlers {
    use super::{RequestProcessorReq, RequestProcessorResp};
    use crate::manager::{ReqCmd, ReqSender};
    use tokio::sync::oneshot;
    use uuid::Uuid;

    pub(crate) async fn create_processor(
        tx: ReqSender<ReqCmd>,
        processor: RequestProcessorReq,
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

        Ok(warp::reply::json(&RequestProcessorResp {
            id: Uuid::new_v4(),
            name: processor.name,
            language: processor.language,
            runtime: processor.runtime,
            code: processor.code,
        }))
    }
}
