use crate::server::admin::{RequestProcessorReq, RequestProcessorResp};
use anyhow::Result;
use fh_v8::{process_request, Request, Response};
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

type Responder<T> = oneshot::Sender<T>;
pub(crate) type ReqSender<T> = Arc<Mutex<mpsc::Sender<T>>>;

#[derive(Debug)]
pub(crate) enum ReqCmd {
    Http {
        request: Request,
        cmd_tx: Responder<Result<Response>>,
    },
    CreateRequestProcessor {
        proc: RequestProcessorReq,
        cmd_tx: Responder<Result<RequestProcessorResp>>,
    },
}

use sqlx::SqlitePool;

pub(crate) async fn request_manager(rx: &mut mpsc::Receiver<ReqCmd>) {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    while let Some(cmd) = rx.recv().await {
        // println!("Got new CMD: {:?}", cmd);
        match cmd {
            ReqCmd::Http {
                request: req,
                cmd_tx,
            } => {
                let res = process_request(req).await;
                cmd_tx.send(Ok(res)).unwrap();
            }
            ReqCmd::CreateRequestProcessor {
                proc: processor,
                cmd_tx,
            } => {
                //TODO: refactor :)
                let mut conn = pool.acquire().await.unwrap();
                let id = Uuid::new_v4();
                let id_str = id.to_string();
                sqlx::query!(
                    r#"INSERT INTO request_processor (id, name, language, runtime, code)
                VALUES (?1, ?2, ?3, ?4, ?5)"#,
                    id_str,
                    processor.name,
                    processor.language,
                    processor.runtime,
                    processor.code
                )
                .execute(&mut conn)
                .await
                .unwrap();

                cmd_tx
                    .send(Ok(RequestProcessorResp {
                        id,
                        name: processor.name,
                        language: processor.language,
                        runtime: processor.runtime,
                        code: processor.code,
                    }))
                    .unwrap();
            }
        }
    }
}
