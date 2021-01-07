use super::{request_processor::get_request_processor, RequestProcessorError};
use anyhow::Result;
use chrono::{TimeZone, Utc};
use fh_core::DbConnection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AuditEntry {
    Request {
        id: Uuid,
        inc: usize,
        conversation_id: Uuid,
        payload: fh_core::request::Request,
    },
    Response {
        id: Uuid,
        conversation_id: Uuid,
        request_id: Uuid,
        payload: fh_core::response::Response,
    },
    Log {
        id: Uuid,
        conversation_id: Uuid,
        payload: String,
    },
}

impl std::string::ToString for AuditEntry {
    fn to_string(&self) -> String {
        match self {
            AuditEntry::Request { .. } => "request".to_string(),
            AuditEntry::Response { .. } => "response".to_string(),
            AuditEntry::Log { .. } => "log".to_string(),
        }
    }
}

impl AuditEntry {
    fn get_parent(&self) -> Option<Uuid> {
        match self {
            AuditEntry::Response { request_id, .. } => Some(*request_id),
            _ => None,
        }
    }

    fn get_payload(&self) -> Result<String, RequestProcessorError> {
        match self {
            AuditEntry::Request { payload, .. } => {
                serde_json::to_string(&payload).map_err(RequestProcessorError::JsonSerialize)
            }
            AuditEntry::Response { payload, .. } => {
                serde_json::to_string(&payload).map_err(RequestProcessorError::JsonSerialize)
            }
            AuditEntry::Log { payload, .. } => Ok(payload.to_owned()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConversation {
    pub id: Uuid,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    created_at: chrono::DateTime<Utc>,
    request_processor_id: Uuid,
    items: Vec<AuditEntry>,
}

pub(crate) async fn create_request_conversation(
    conn: &mut DbConnection,
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
        items: Vec::new(),
    })
}

pub(crate) async fn get_request_conversation(
    conn: &mut DbConnection,
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
            items: Vec::new(),
        }),
    }
}

pub(crate) async fn get_audit_log_entries(
    conn: &mut DbConnection,
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
    conn: &mut DbConnection,
    conversation_id: &Uuid,
    item: AuditEntry,
) -> Result<Uuid, RequestProcessorError> {
    let conv = get_request_conversation(conn, conversation_id).await?;

    let item_id = Uuid::new_v4();
    let item_id_str = item_id.to_string();
    let conv_id_str = conv.id.to_string();
    let created_at = Utc::now().timestamp_nanos();
    let kind = item.to_string();
    let payload = item.get_payload()?;
    let parent = item
        .get_parent()
        .map_or("".to_string(), |id| id.to_string());

    sqlx::query!(
        r#"INSERT INTO conversation_audit_log
                    (id, created_at, request_conversation, parent, kind, payload)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        item_id_str,
        created_at,
        conv_id_str,
        parent,
        kind,
        payload,
    )
    .execute(conn)
    .await?;

    Ok(item_id)
}
