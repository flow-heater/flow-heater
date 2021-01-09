use super::{request_processor::get_request_processor, RequestProcessorError};
use anyhow::Result;
use chrono::{DateTime, Utc};
use fh_core::DbConnection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

pub trait AuditItem {}

#[derive(Debug, Serialize, Deserialize)]
struct AuditItemRequest {
    id: Uuid,
    created_at: DateTime<Utc>,
    inc: usize,
    conversation_id: Uuid,
    payload: fh_core::request::Request,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditItemResponse {
    id: Uuid,
    created_at: DateTime<Utc>,
    conversation_id: Uuid,
    request_id: Uuid,
    payload: fh_core::response::Response,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditItemLog {
    id: Uuid,
    created_at: DateTime<Utc>,
    conversation_id: Uuid,
    payload: String,
    items: AuditItems,
}

impl AuditItem for AuditItemLog {}
impl AuditItem for AuditItemResponse {}
impl AuditItem for AuditItemRequest {}

#[derive(Debug, Serialize, Deserialize)]
struct AuditItems {
    requests: Vec<AuditItemRequest>,
    responses: Vec<AuditItemResponse>,
    logs: Vec<AuditItemLog>,
}

impl AuditItems {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            responses: Vec::new(),
            logs: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConversation {
    pub id: Uuid,
    created_at: chrono::DateTime<Utc>,
    request_processor_id: Uuid,
    items: AuditItems,
}

impl RequestConversation {
    fn get_items(&self) -> Result<AuditItems, RequestProcessorError> {
        todo!()
    }
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
        items: AuditItems::new(),
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
            created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            request_processor_id: Uuid::from_str(&row.request_processor)?,
            items: AuditItems::new(),
        }),
    }
}

pub(crate) async fn get_audit_items(
    _conn: &mut DbConnection,
    conversation_id: &Uuid,
) -> Result<AuditItems, RequestProcessorError> {
    let _id_str = conversation_id.to_string();
    // let mut rows = sqlx::query("SELECT * FROM conversation_audit_log WHERE id = ?")
    //     .bind(id_str)
    //     .fetch(conn);

    let items = AuditItems::new();

    // while let Some(row) = rows.try_next().await? {
    //     items.push(match row.try_get("kind")? {
    //        "request" => Ok(AuditItem::Request {
    //            id: Uuid::from_str(&row.id)?,
    //            inc: 0,
    //            conversation_id: conversation_id.clone(),
    //            payload: serde_json::from_str(&row.payload),
    //        }),
    //        "response" => Ok(AuditItem::Response {
    //            id: (), id: (), conversation_id: (), payload: ()
    //            conversation_id: (),
    //            request_id: (),
    //            payload: (),
    //        }),
    //        "log" => Ok(AuditItem::Log { id: (), conversation_id: (), payload: ()}),
    //    })
    // }

    Ok(items)
}

pub(crate) async fn create_audit_item<T: AuditItem>(
    conn: &mut DbConnection,
    item: T,
) -> Result<T, RequestProcessorError> {
    let conv = get_request_conversation(conn, &item.get_conversation_id()).await?;

    let item_id = Uuid::new_v4();
    let item_id_str = item_id.to_string();
    let conv_id_str = conv.id.to_string();
    let created_at = Utc::now().to_rfc3339();
    let kind = item.to_string();
    let payload = item.get_payload()?;
    let _parent = item
        .get_parent()
        .map_or("".to_string(), |id| id.to_string());

    // TODO: either parent is NULL or filled with a string
    sqlx::query!(
        r#"INSERT INTO conversation_audit_log
                    (id, created_at, request_conversation, kind, payload)
                    VALUES (?1, ?2, ?3, ?4, ?5)"#,
        item_id_str,
        created_at,
        conv_id_str,
        kind,
        payload,
    )
    .execute(conn)
    .await?;

    Ok(item)
}
