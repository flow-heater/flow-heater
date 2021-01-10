use super::{request_processor::get_request_processor, RequestProcessorError};
use anyhow::Result;
use chrono::{DateTime, Utc};
use fh_core::DbConnection;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AuditItem {
    #[serde(rename = "request")]
    Request {
        id: Uuid,
        created_at: DateTime<Utc>,
        inc: i32,
        conversation_id: Uuid,
        payload: fh_core::request::Request,
    },
    #[serde(rename = "response")]
    Response {
        id: Uuid,
        created_at: DateTime<Utc>,
        conversation_id: Uuid,
        request_id: Uuid,
        payload: fh_core::response::Response,
    },
    #[serde(rename = "log")]
    Log {
        id: Uuid,
        created_at: DateTime<Utc>,
        conversation_id: Uuid,
        payload: String,
    },
}

impl AuditItem {
    pub fn new_request(
        conversation_id: Uuid,
        inc: i32,
        payload: fh_core::request::Request,
    ) -> Self {
        Self::Request {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            inc,
            conversation_id,
            payload,
        }
    }

    pub fn new_response(
        conversation_id: Uuid,
        request_id: Uuid,
        payload: fh_core::response::Response,
    ) -> Self {
        Self::Response {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            conversation_id,
            request_id,
            payload,
        }
    }

    pub fn new_log(conversation_id: Uuid, payload: String) -> Self {
        Self::Log {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            conversation_id,
            payload,
        }
    }

    fn from_db_audit_item(item: &DbAuditItem) -> Result<Self, RequestProcessorError> {
        Ok(match item.kind.as_str() {
            "request" => Self::Request {
                id: item.id,
                created_at: item.created_at,
                inc: item.inc.ok_or(RequestProcessorError::EmptyDbField(
                    "Field 'ínc' must not be NULL for kind 'request'".to_string(),
                ))?,
                conversation_id: item.conversation_id,
                payload: serde_json::from_str(&item.payload)?,
            },
            "response" => Self::Response {
                id: item.id,
                created_at: item.created_at,
                conversation_id: item.conversation_id,
                request_id: item.request_id.ok_or(RequestProcessorError::EmptyDbField(
                    "Field 'request' must not be NULL for kind 'response'".to_string(),
                ))?,
                payload: serde_json::from_str(&item.payload)?,
            },
            "log" => Self::Log {
                id: item.id,
                created_at: item.created_at,
                conversation_id: item.conversation_id,
                payload: item.payload.clone(),
            },
            _ => todo!(),
        })
    }

    fn to_audit_db_item(&self) -> Result<DbAuditItem, RequestProcessorError> {
        Ok(match self {
            AuditItem::Request {
                id,
                conversation_id,
                created_at,
                inc,
                payload,
            } => DbAuditItem {
                kind: "request".to_string(),
                id: *id,
                conversation_id: *conversation_id,
                created_at: *created_at,
                inc: Some(*inc),
                payload: serde_json::to_string(payload)?,
                request_id: None,
            },
            AuditItem::Response {
                id,
                created_at,
                conversation_id,
                request_id,
                payload,
            } => DbAuditItem {
                kind: "response".to_string(),
                id: *id,
                conversation_id: *conversation_id,
                created_at: *created_at,
                inc: None,
                payload: serde_json::to_string(payload)?,
                request_id: Some(*request_id),
            },
            AuditItem::Log {
                id,
                created_at,
                conversation_id,
                payload,
            } => DbAuditItem {
                kind: "log".to_string(),
                id: *id,
                conversation_id: *conversation_id,
                created_at: *created_at,
                inc: None,
                payload: payload.clone(),
                request_id: None,
            },
        })
    }
}

#[derive(Debug, FromRow)]
struct DbAuditItem {
    id: Uuid,
    kind: String,
    created_at: DateTime<Utc>,
    inc: Option<i32>,
    conversation_id: Uuid,
    request_id: Option<Uuid>,
    payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConversation {
    pub id: Uuid,
    created_at: chrono::DateTime<Utc>,
    request_processor_id: Uuid,
    audit_items: Vec<AuditItem>,
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
    let now_str = now.to_rfc3339();
    sqlx::query!(
        r#"INSERT INTO request_conversation
                    (id, created_at, request_processor)
                    VALUES (?1, ?2, ?3)"#,
        id_str,
        now_str,
        req_id_str,
    )
    .execute(conn)
    .await?;

    Ok(RequestConversation {
        id: conversation_id,
        created_at: now,
        request_processor_id: *request_processor_id,
        audit_items: Vec::new(),
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
    .fetch_one(&mut *conn)
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
            created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            request_processor_id: Uuid::from_str(&row.request_processor)?,
            audit_items: get_audit_items(conn, id).await?,
        }),
    }
}

pub async fn get_audit_items(
    conn: &mut DbConnection,
    conversation_id: &Uuid,
) -> Result<Vec<AuditItem>, RequestProcessorError> {
    let conv_id_str = conversation_id.to_string();
    let mut items = Vec::new();

    let mut rows =
        sqlx::query("SELECT * FROM conversation_audit_item WHERE request_conversation = ?1")
            .bind(conv_id_str)
            .fetch(conn);

    while let Some(row) = rows.try_next().await? {
        let i = DbAuditItem {
            id: Uuid::from_str(row.try_get("id")?)?,
            kind: row.try_get("kind")?,
            created_at: row.try_get("created_at")?,
            conversation_id: Uuid::from_str(row.try_get("request_conversation")?)?,
            inc: row.try_get("inc")?,
            payload: row.try_get("payload")?,
            // TODO: this won't work yet, because this is an Option<Uuid>
            // and we do not use `Uuid::from_str()`, yet
            request_id: row.try_get("parent")?,
        };

        items.push(AuditItem::from_db_audit_item(&i)?);
    }

    items.sort_by(|a, b| {
        let a_created_at = match a {
            AuditItem::Log { created_at, .. } => created_at,
            AuditItem::Response { created_at, .. } => created_at,
            AuditItem::Request { created_at, .. } => created_at,
        };

        let b_created_at = match b {
            AuditItem::Log { created_at, .. } => created_at,
            AuditItem::Response { created_at, .. } => created_at,
            AuditItem::Request { created_at, .. } => created_at,
        };

        a_created_at.cmp(&b_created_at)
    });

    Ok(items)
}

pub(crate) async fn create_audit_item(
    conn: &mut DbConnection,
    item: AuditItem,
) -> Result<AuditItem, RequestProcessorError> {
    let db_item = item.to_audit_db_item()?;
    let conv = get_request_conversation(conn, &db_item.conversation_id).await?;
    let item_id_str = db_item.id.to_string();
    let conv_id_str = conv.id.to_string();
    let created_at = Utc::now().to_rfc3339();
    let payload = db_item.payload;

    sqlx::query!(
        r#"INSERT INTO conversation_audit_item
                    (id, kind, created_at, inc, request_conversation, parent, payload)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        item_id_str,
        db_item.kind,
        created_at,
        db_item.inc,
        conv_id_str,
        db_item.request_id,
        payload,
    )
    .execute(conn)
    .await?;

    Ok(item)
}
