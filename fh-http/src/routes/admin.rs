use actix_web::{web, HttpResponse};
use fh_db::{request_processor::RequestProcessor, ReqCmd};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub async fn create_processor(
    processor: web::Json<RequestProcessor>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    let res = db_cmd!(
        tx_db,
        ReqCmd::CreateRequestProcessor {
            proc: processor.clone(),
            cmd_tx,
        },
        cmd_rx
    );
    HttpResponse::Ok().json(res)
}

pub async fn get_processor(
    info: web::Path<(Uuid,)>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    let proc = db_cmd!(
        tx_db,
        ReqCmd::GetRequestProcessor {
            id: info.into_inner().0,
            cmd_tx
        },
        cmd_rx
    );
    HttpResponse::Ok().json(proc)
}

pub async fn update_processor(
    info: web::Path<(Uuid,)>,
    processor: web::Json<RequestProcessor>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    let res = db_cmd!(
        tx_db,
        ReqCmd::UpdateRequestProcessor {
            id: info.into_inner().0,
            proc: processor.clone(),
            cmd_tx,
        },
        cmd_rx
    );

    HttpResponse::Ok().json(res)
}

pub async fn delete_processor(
    info: web::Path<(Uuid,)>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    db_cmd!(
        tx_db,
        ReqCmd::DeleteRequestProcessor {
            id: info.into_inner().0,
            cmd_tx
        },
        cmd_rx
    );

    HttpResponse::Ok().finish()
}
