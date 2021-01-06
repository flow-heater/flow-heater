use super::{request_processor::get_request_processor, RequestProcessorError};
use anyhow::Result;
use chrono::{TimeZone, Utc};
use sqlx::{pool::PoolConnection, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) enum AuditEntry {
    Request(fh_core::request::Request),
    Response(fh_core::response::Response),
    Log(String),
}

#[derive(Debug)]
pub struct RequestConversation {
    pub id: Uuid,
    created_at: chrono::DateTime<Utc>,
    request_processor_id: Uuid,
    audit_entries: Vec<AuditEntry>,
}

pub(crate) async fn create_request_conversation(
    conn: &mut PoolConnection<Sqlite>,
    request_processor_id: &Uuid,
) -> Result<RequestConversation, RequestProcessorError> {
    let conversation_id = Uuid::new_v4();

    let _p = get_request_processor(conn, request_processor_id).await?;
    let id_str = conversation_id.to_string();
    let req_id_str = request_processor_id.to_string();
    let now = Utc::now();
    let now_ts = now.timestamp_nanos();
    sqlx::query!(
        r#"INSERT INTO request_conversation
                    (id, created_at, request_processor)
                    VALUES (?1, ?2, ?3)"#,
        id_str,
        now_ts,
        req_id_str,
    )
    .execute(conn)
    .await?;

    Ok(RequestConversation {
        id: conversation_id,
        created_at: now,
        request_processor_id: *request_processor_id,
        audit_entries: Vec::new(),
    })
}

pub(crate) async fn get_request_conversation(
    conn: &mut PoolConnection<Sqlite>,
    id: &Uuid,
) -> Result<RequestConversation, RequestProcessorError> {
    let id_str = id.to_string();
    let row = sqlx::query!(
        r#"SELECT * FROM request_conversation WHERE id = ?1"#,
        id_str
    )
    .fetch_one(conn)
    .await;

    match row {
        Err(x) => match x {
            sqlx::Error::RowNotFound => {
                return Err(RequestProcessorError::NotFound {
                    id: *id,
                    kind: "RequestConversation".to_string(),
                })
            }
            _ => Err(RequestProcessorError::Db(x)),
        },
        Ok(row) => Ok(RequestConversation {
            id: *id,
            created_at: Utc.timestamp(row.created_at, 0),
            request_processor_id: Uuid::from_str(&row.request_processor)?,
            audit_entries: Vec::new(),
        }),
    }
}

pub(crate) async fn get_audit_log_entries(
    conn: &mut PoolConnection<Sqlite>,
    conversation_id: &Uuid,
) -> Result<Vec<AuditEntry>, RequestProcessorError> {
    let id_str = conversation_id.to_string();
    let row = sqlx::query!(
        r#"SELECT * FROM request_conversation WHERE id = ?1"#,
        id_str
    )
    .fetch_one(conn)
    .await;

    todo!()
}

pub(crate) async fn create_audit_log_entry(
    conn: &mut PoolConnection<Sqlite>,
    conversation_id: &Uuid,
    entry: AuditEntry,
) -> Result<Uuid, RequestProcessorError> {
    let conv = get_request_conversation(conn, conversation_id).await?;
    let entry_id = Uuid::new_v4();

    let conv_id_str = conv.id.to_string();
    let entry_id_str = entry_id.to_string();
    let created_at = Utc::now().timestamp_nanos();

    let kind: String;
    let payload: String;

    match entry {
        AuditEntry::Request(r) => {
            kind = "request".to_string();
            payload = serde_json::to_string(&r)?;
        }
        AuditEntry::Response(r) => {
            kind = "response".to_string();
            payload = serde_json::to_string(&r)?;
        }
        AuditEntry::Log(s) => {
            kind = "log".to_string();
            payload = s;
        }
    }

    sqlx::query!(
        r#"INSERT INTO conversation_audit_log
                    (id, created_at, request_conversation, kind, payload)
                    VALUES (?1, ?2, ?3, ?4, ?5)"#,
        entry_id_str,
        created_at,
        conv_id_str,
        kind,
        payload
    )
    .execute(conn)
    .await?;

    Ok(entry_id)
}
