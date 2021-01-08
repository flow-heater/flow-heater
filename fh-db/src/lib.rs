use self::request_processor::RequestProcessor;
use anyhow::{Context, Error, Result};
use fh_core::{DbPool, DbType, Responder, TypedPool};
use request_conversation::{AuditItem, RequestConversation};
use std::env;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

pub mod request_conversation;
pub mod request_processor;

#[derive(Error, Debug)]
pub enum RequestProcessorError {
    #[error("{kind} with id {id} not found")]
    NotFound { kind: String, id: Uuid },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("Unable to parse enum: {0}")]
    Parse(#[from] strum::ParseError),
    #[error(transparent)]
    UuidParse(#[from] uuid::Error),
    #[error(transparent)]
    JsonSerialize(#[from] serde_json::Error),
    #[error(transparent)]
    Processing(#[from] anyhow::Error),
    #[error("{0}")]
    Locking(String),
    #[error(transparent)]
    TimeParse(#[from] chrono::ParseError),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug)]
pub enum ReqCmd {
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
    CreateRequestConversation {
        request_processor_id: Uuid,
        cmd_tx: Responder<Result<RequestConversation, RequestProcessorError>>,
    },
    CreateAuditLogEntry {
        item: AuditItem,
        cmd_tx: Responder<Result<AuditItem, RequestProcessorError>>,
    },
    GetConversationItems {
        id: Uuid,
        cmd_tx: Responder<Result<Vec<AuditItem>, RequestProcessorError>>,
    },
}

pub async fn request_manager(rx: &mut mpsc::Receiver<ReqCmd>) -> anyhow::Result<()> {
    let pool = TypedPool::connect(&env::var("DATABASE_URL")?)
        .await
        .context("Connection to DB failed")?;

    while let Some(cmd) = rx.recv().await {
        process_command(cmd, &pool).await?;
    }

    Ok(())
}

async fn process_command(cmd: ReqCmd, pool: &DbPool<DbType>) -> Result<()> {
    match cmd {
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
        ReqCmd::CreateRequestConversation {
            request_processor_id,
            cmd_tx,
        } => {
            let conv = self::request_conversation::create_request_conversation(
                &mut pool.acquire().await?,
                &request_processor_id,
            )
            .await;

            cmd_tx
                .send(conv)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
        }
        ReqCmd::CreateAuditLogEntry { item, cmd_tx } => {
            let item =
                self::request_conversation::create_audit_item(&mut pool.acquire().await?, item)
                    .await;

            cmd_tx
                .send(item)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
        }
        ReqCmd::GetConversationItems { id, cmd_tx } => {
            let items =
                self::request_conversation::get_audit_items(&mut pool.acquire().await?, &id).await;

            cmd_tx
                .send(items)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
        }
    }

    Ok(())
}
