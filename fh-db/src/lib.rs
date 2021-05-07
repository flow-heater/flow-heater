use self::request_processor::RequestProcessor;
use anyhow::{Error, Result};
use configuration::DatabaseSettings;
use fh_core::{DbPool, DbType, Responder};
use request_conversation::{AuditItem, RequestConversation};
use sqlx::postgres::PgPoolOptions;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

pub mod configuration;
pub mod request_conversation;
pub mod request_processor;

/// Central Error type for all kinds of internal errors.
///
/// TODO: move to fh-core
#[derive(Error, Debug)]
pub enum RequestProcessorError {
    /// Database Error: in case an entity is not found for a given Uuid.
    #[error("{kind} with id {id} not found")]
    NotFound { kind: String, id: Uuid },

    /// Database Error: All unspecific sqlx Errors are wrapped here.
    #[error(transparent)]
    Db(#[from] sqlx::Error),

    /// Happens when parsing a string to an enum fails.
    #[error("Unable to parse to enum: {0}")]
    Parse(#[from] strum::ParseError),

    /// Happens when parsing a string to a Uuid fails.
    #[error(transparent)]
    UuidParse(#[from] uuid::Error),

    /// Happens when Json (de-)serialization fails.
    #[error(transparent)]
    JsonSerialize(#[from] serde_json::Error),

    /// Generic error variant, wrapping [`anyhow::Error`]. This is needed
    /// because e.g. `deno_core` uses this type a lot.
    #[error(transparent)]
    Processing(#[from] anyhow::Error),

    /// Happens when locking Mutexes fails.
    #[error("{0}")]
    Locking(String),

    /// Happens when time parsing errors occur.
    #[error(transparent)]
    TimeParse(#[from] chrono::ParseError),

    /// Completely custom error type wrapper.
    #[error("{0}")]
    Custom(String),

    /// Error, when a nullable DB field is NULL but should not be for specific
    /// cases.
    #[error("{0}")]
    EmptyDbField(String),
}

/// Central Command Enum, which contains all Commands to be sent to the `fh_db`
/// crate. A ReqCmd is received over a [`tokio::sync::mpsc`] channel and handled
/// in the [`crate::request_manager`] function.
///
/// Each variant of the ReqCmd responds data back using a [`Responder`] type
/// which, by convention is given by the variant field `cmd_tx`. The Responder
/// is the transmitter of a [`tokio::sync::oneshot`] channel.
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
    GetRequestConversation {
        id: Uuid,
        cmd_tx: Responder<Result<RequestConversation, RequestProcessorError>>,
    },
    GetRequestConversationAuditItems {
        id: Uuid,
        cmd_tx: Responder<Result<Vec<AuditItem>, RequestProcessorError>>,
    },
}

/// Async function which can be run e.g. by tokio which loops forever and
/// receives [`ReqCmd`] commands via the given Receiver.
pub async fn request_manager(
    rx: &mut mpsc::Receiver<ReqCmd>,
    settings: &DatabaseSettings,
) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(settings.with_db())
        .await?;

    while let Some(cmd) = rx.recv().await {
        process_command(cmd, &pool).await?;
    }

    Ok(())
}

/// Actual `ReqCmd` command processor which matches the given variant and calls
/// the underlying functions in the submodules (e.g.
/// [`crate::request_processor'] or [`crate::request_conversation`]).
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
        ReqCmd::GetRequestConversationAuditItems { id, cmd_tx } => {
            let items =
                self::request_conversation::get_audit_items(&mut pool.acquire().await?, &id).await;

            cmd_tx
                .send(items)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
        }
        ReqCmd::GetRequestConversation { id, cmd_tx } => {
            let items = self::request_conversation::get_request_conversation(
                &mut pool.acquire().await?,
                &id,
            )
            .await;

            cmd_tx
                .send(items)
                .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?;
        }
    }

    Ok(())
}
