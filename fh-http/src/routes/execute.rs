use std::convert::TryFrom;

use actix_web::{
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
use fh_core::request::Request;
use fh_db::ReqCmd;
use fh_v8::ProcessorCmd;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub async fn run_processor(
    info: web::Path<(Uuid,)>,
    tx_v8: web::Data<mpsc::Sender<ProcessorCmd>>,
    bytes: Bytes,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
    request: HttpRequest,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();

    let mut request = Request::try_from(request).unwrap();
    request.body = String::from_utf8(bytes.to_vec())
        .map_err(|_| HttpResponse::BadRequest().finish())
        .unwrap();

    let res = proc_cmd!(
        tx_v8,
        ProcessorCmd::RunRequestProcessor {
            id: info.into_inner().0,
            request: request,
            cmd_tx,
            tx_db: tx_db.into_inner(),
            prelude: false,
        },
        cmd_rx
    );

    HttpResponse::Ok()
        .append_header((
            "fh-conversation-id",
            res.headers.get("FH-Conversation-Id").unwrap()[0].clone(),
        ))
        .json(res)
}

pub async fn run_processor_with_prelude(
    info: web::Path<(Uuid,)>,
    tx_v8: web::Data<mpsc::Sender<ProcessorCmd>>,
    bytes: Bytes,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
    request: HttpRequest,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();

    let mut request = Request::try_from(request).unwrap();
    request.body = String::from_utf8(bytes.to_vec())
        .map_err(|_| HttpResponse::BadRequest().finish())
        .unwrap();

    let res = proc_cmd!(
        tx_v8,
        ProcessorCmd::RunRequestProcessor {
            id: info.into_inner().0,
            request: request,
            cmd_tx,
            tx_db: tx_db.into_inner(),
            prelude: true,
        },
        cmd_rx
    );

    HttpResponse::Ok()
        .append_header((
            "fh-conversation-id",
            res.headers.get("FH-Conversation-Id").unwrap()[0].clone(),
        ))
        .json(res)
}

pub async fn deprecated_run_processor(
    tx_v8: web::Data<mpsc::Sender<ProcessorCmd>>,
    bytes: Bytes,
    tx_db: web::Data<mpsc::Sender<ReqCmd>>,
    request: HttpRequest,
) -> HttpResponse {
    let (cmd_tx, cmd_rx) = oneshot::channel();

    let mut request = Request::try_from(request).unwrap();
    request.body = String::from_utf8(bytes.to_vec())
        .map_err(|_| HttpResponse::BadRequest().finish())
        .unwrap();

    let res = proc_cmd!(
        tx_v8,
        ProcessorCmd::Http {
            request,
            cmd_tx: cmd_tx,
            tx_db: tx_db.into_inner(),
        },
        cmd_rx
    );

    HttpResponse::Ok()
        .append_header((
            "fh-conversation-id",
            res.headers.get("FH-Conversation-Id").unwrap()[0].clone(),
        ))
        .json(res)
}
