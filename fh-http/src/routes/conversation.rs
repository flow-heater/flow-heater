use actix_web::{web, HttpResponse};
use fh_db::ReqCmd;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub async fn get_request_conversation(
    info: web::Path<(Uuid,)>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    let res = db_cmd!(
        tx_db,
        ReqCmd::GetRequestConversation {
            id: info.into_inner().0,
            cmd_tx: cmd_tx
        },
        cmd_rx
    );
    HttpResponse::Ok().json(res)
}

pub async fn get_request_conversation_audit_items(
    info: web::Path<(Uuid,)>,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();
    let res = db_cmd!(
        tx_db,
        ReqCmd::GetRequestConversationAuditItems {
            id: info.into_inner().0,
            cmd_tx: cmd_tx
        },
        cmd_rx
    );
    HttpResponse::Ok().json(res)
}
