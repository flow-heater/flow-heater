use self::request_processor::RequestProcessor;
use anyhow::{Context, Result};
use fh_v8::{process_request, Request, Response};
use sqlx::{Pool, SqlitePool};
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

type Responder<T> = oneshot::Sender<T>;
pub(crate) type ReqSender<T> = Arc<Mutex<mpsc::Sender<T>>>;

pub(crate) mod request_processor;

#[derive(Debug)]
pub(crate) enum ReqCmd {
    Http {
        request: Request,
        cmd_tx: Responder<Result<Response>>,
    },
    CreateRequestProcessor {
        proc: RequestProcessor,
        cmd_tx: Responder<Result<RequestProcessor>>,
    },
    GetRequestProcessor {
        id: Uuid,
        cmd_tx: Responder<Result<RequestProcessor>>,
    },
    UpdateRequestProcessor {
        id: Uuid,
        proc: RequestProcessor,
        cmd_tx: Responder<Result<RequestProcessor>>,
    },
    DeleteRequestProcessor {
        id: Uuid,
        cmd_tx: Responder<Result<()>>,
    },
    RunRequestProcessor {
        id: Uuid,
        request: Request,
        cmd_tx: Responder<Result<Response>>,
    },
}

pub(crate) async fn request_manager(rx: &mut mpsc::Receiver<ReqCmd>) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?)
        .await
        .context("Connection to DB failed")?;

    while let Some(cmd) = rx.recv().await {
        process_command(cmd, &pool).await?;
    }

    Ok(())
}

async fn process_command(cmd: ReqCmd, pool: &Pool<sqlx::Sqlite>) -> Result<()> {
    match cmd {
        ReqCmd::Http {
            request: req,
            cmd_tx,
        } => {
            let res = process_request(req, None).await;
            cmd_tx.send(res).unwrap();
        }
        ReqCmd::CreateRequestProcessor {
            proc: processor,
            cmd_tx,
        } => {
            self::request_processor::create_request_processor(
                &mut pool.acquire().await?,
                &processor,
            )
            .await?;
            cmd_tx.send(Ok(processor)).unwrap();
        }
        ReqCmd::GetRequestProcessor { id, cmd_tx } => {
            let p = self::request_processor::get_request_processor(&mut pool.acquire().await?, &id)
                .await?;
            cmd_tx.send(Ok(p)).unwrap();
        }
        ReqCmd::UpdateRequestProcessor {
            id,
            proc: mut processor,
            cmd_tx,
        } => {
            self::request_processor::update_request_processor(
                &mut pool.acquire().await?,
                &id,
                &mut processor,
            )
            .await?;
            cmd_tx.send(Ok(processor)).unwrap();
        }
        ReqCmd::DeleteRequestProcessor { id, cmd_tx } => {
            let p =
                self::request_processor::delete_request_processor(&mut pool.acquire().await?, &id)
                    .await?;
            cmd_tx.send(Ok(p)).unwrap();
        }
        ReqCmd::RunRequestProcessor {
            id,
            request,
            cmd_tx,
        } => {
            let r = self::request_processor::run_request_processor(
                &mut pool.acquire().await?,
                &id,
                request,
            )
            .await?;
            cmd_tx.send(Ok(r)).unwrap();
        }
    }

    Ok(())
}
