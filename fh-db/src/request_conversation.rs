//! Database structs and functions for the [`RequestConversation`] and
//! subsequent [`AuditItem`] entities.
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

/// Represents all possible Item variants, which can be handled/stored for a
/// RequestConversation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AuditItem {
    /// Incoming HTTP request (increment 0) or a subsequently issued HTTP
    /// request from the JavaScript runtime. The `inc` field is just a counter
    /// to indicate, in which order the requests were issued in case the
    /// `created_at` is ambigous.
    ///
    /// Uses [`fh_core::request::Request`] to serialize the full request data as
    /// JSON.
    #[serde(rename = "request")]
    Request {
        id: Uuid,
        created_at: DateTime<Utc>,
        inc: i32,
        conversation_id: Uuid,
        payload: fh_core::request::Request,
    },
    /// Represents a received HTTP response for a given HTTP request. Contains a
    /// `request_id` (Uuid) field, which references an [`AuditItem`] of type
    /// [`AuditItem::Request`].
    ///
    /// Uses [`fh_core::response::Response`] to serialize the full response data
    /// as JSON.
    #[serde(rename = "response")]
    Response {
        id: Uuid,
        created_at: DateTime<Utc>,
        conversation_id: Uuid,
        request_id: Uuid,
        payload: fh_core::response::Response,
    },
    /// Represents a simple log entry (string) to be stored in an [`AuditItem`].
    #[serde(rename = "log")]
    Log {
        id: Uuid,
        created_at: DateTime<Utc>,
        conversation_id: Uuid,
        payload: String,
    },
}

impl AuditItem {
    /// Gets the Uuid of the underlying variant.
    pub fn get_id(&self) -> Uuid {
        match self {
            Self::Response { id, .. } => *id,
            Self::Request { id, .. } => *id,
            Self::Log { id, .. } => *id,
        }
    }

    /// Helper method to create a new [`AuditItem::Request`] variant.
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

    /// Helper method to create a new [`AuditItem::Response`] variant.
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

    /// Helper method to create a new [`AuditItem::Log`] variant.
    pub fn new_log(conversation_id: Uuid, payload: String) -> Self {
        Self::Log {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            conversation_id,
            payload,
        }
    }

    /// Convert a [`DbAuditItem`] back to a [`AuditItem`].
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
            s => {
                return Err(RequestProcessorError::Custom(format!(
                    "Unsupported AuditItem type: '{}'",
                    s
                )))
            }
        })
    }

    /// Convert a [`AuditItem`] to a [`DbAuditItem`].
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

/// Represents an AuditItem which can be stored to the database. This struct is
/// needed because the Payload varies for Request/Response/Log and possible
/// further ones. Conversion from/to [`AuditItem`] happens via
/// [`AuditItem::to_audit_db_item`] and [`AuditItem::from_db_audit_item`].
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

/// A RequestConversation is created on each request to a
/// [`crate::request_processor::RequestProcessor`] endpoint. It is linked to the
/// respective RequestProcessor by it's Uuid and contains a list of
/// [`AuditItem`] entities.
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConversation {
    pub id: Uuid,
    created_at: chrono::DateTime<Utc>,
    request_processor_id: Uuid,
    audit_items: Vec<AuditItem>,
}

/// Physically writes a [`RequestConversation`] struct to the underlying
/// database.
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

/// Fetches a [`RequestConversation`] struct from the underlying database, using
/// a RequestConversation Uuid.
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

/// Fetches all AuditItem instances for a single Conversation Uuid.
/// The output is chronologically sorted.
pub(crate) async fn get_audit_items(
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
        let req_id: Option<String> = row.try_get("parent")?;

        let i = DbAuditItem {
            id: Uuid::from_str(row.try_get("id")?)?,
            kind: row.try_get("kind")?,
            created_at: row.try_get("created_at")?,
            conversation_id: Uuid::from_str(row.try_get("request_conversation")?)?,
            inc: row.try_get("inc")?,
            payload: row.try_get("payload")?,
            request_id: req_id.and_then(|x| Uuid::from_str(&x).ok()),
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

/// Stores a single AuditItem to the underlying database.
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
    let request_id_str = db_item.request_id.and_then(|x| Some(x.to_string()));

    sqlx::query!(
        r#"INSERT INTO conversation_audit_item
                    (id, kind, created_at, inc, request_conversation, parent, payload)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        item_id_str,
        db_item.kind,
        created_at,
        db_item.inc,
        conv_id_str,
        request_id_str,
        payload,
    )
    .execute(conn)
    .await?;

    Ok(item)
}
