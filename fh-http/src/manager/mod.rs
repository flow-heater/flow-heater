use self::request_processor::RequestProcessor;
use anyhow::{Context, Error, Result};
use fh_v8::{process_request, request::Request, response::Response};
use sqlx::{Pool, SqlitePool};
use std::{
    env,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

type Responder<T> = oneshot::Sender<T>;
pub(crate) type ReqSender<T> = Arc<Mutex<mpsc::Sender<T>>>;

pub(crate) mod request_processor;

#[derive(Error, Debug)]
pub enum RequestProcessorError {
    #[error("{kind} with id {id} not found")]
    NotFound { kind: String, id: Uuid },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("Unable to parse enum: {0}")]
    Parse(#[from] strum::ParseError),
    #[error(transparent)]
    Processing(#[from] anyhow::Error),
}

#[derive(Debug)]
pub(crate) enum ReqCmd {
    Http {
        request: Request,
        cmd_tx: Responder<Result<Response, RequestProcessorError>>,
    },
    CreateRequestProcessor {
        proc: RequestProcessor,
        cmd_tx: Responder<Result<RequestProcessor, RequestProcessorError>>,
    },
    GetRequestProcessor {
        id: Uuid,
        cmd_tx: Responder<Result<RequestProcessor, RequestProcessorError>>,
    },
    UpdateRequestProcessor {
        id: Uuid,
        proc: RequestProcessor,
        cmd_tx: Responder<Result<RequestProcessor, RequestProcessorError>>,
    },
    DeleteRequestProcessor {
        id: Uuid,
        cmd_tx: Responder<Result<(), RequestProcessorError>>,
    },
    RunRequestProcessor {
        id: Uuid,
        request: Request,
        cmd_tx: Responder<Result<Response, RequestProcessorError>>,
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
            let res = process_request(req, None)
                .await
                .map_err(|e| RequestProcessorError::from(e));
            cmd_tx.send(res).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
        ReqCmd::CreateRequestProcessor {
            proc: processor,
            cmd_tx,
        } => {
            let res = self::request_processor::create_request_processor(
                &mut pool.acquire().await?,
                &processor,
            )
            .await;

            cmd_tx.send(res.and(Ok(processor))).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
        ReqCmd::GetRequestProcessor { id, cmd_tx } => {
            let p = self::request_processor::get_request_processor(&mut pool.acquire().await?, &id)
                .await;
            cmd_tx.send(p).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
        ReqCmd::UpdateRequestProcessor {
            id,
            proc: mut processor,
            cmd_tx,
        } => {
            let res = self::request_processor::update_request_processor(
                &mut pool.acquire().await?,
                &id,
                &mut processor,
            )
            .await;
            cmd_tx.send(res.and(Ok(processor))).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
        ReqCmd::DeleteRequestProcessor { id, cmd_tx } => {
            let p =
                self::request_processor::delete_request_processor(&mut pool.acquire().await?, &id)
                    .await;
            cmd_tx
                .send(p)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
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
            .await;
            cmd_tx.send(r).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
    }

    Ok(())
}
