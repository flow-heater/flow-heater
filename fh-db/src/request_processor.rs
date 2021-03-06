//! Database structs and functions for the RequestProcessor entity.
use super::RequestProcessorError;
use anyhow::Result;
use fh_core::DbConnection;
use futures::TryStreamExt;
use serde::{self, Deserialize, Serialize};
use sqlx::Row;
use std::{convert::AsRef, str::FromStr};
use strum_macros::{self, AsRefStr, EnumString};
use uuid::Uuid;

/// Central representation of a user-defined code snippet, which is used to
/// process incoming requests. The RequestProcessor's Uuid is used as the
/// identifier in the URL (`/processor/{processor_id}/run`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestProcessor {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub name: String,
    pub language: RequestProcessorLanguage,
    pub runtime: RequestProcessorRuntime,
    pub code: String,
    #[serde(default = "anonymous")]
    pub user_id: String,
}

fn anonymous() -> String {
    "anonymous".to_string()
}

/// Variantes of supported language snippets.
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum RequestProcessorLanguage {
    Javascript,
    //Typescript,
}

/// Variantes of supported runtimes.
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum RequestProcessorRuntime {
    V8,
    //WASM,
}

/// Stores a new RequestProcessor to the underlying database.
pub(crate) async fn create_request_processor(
    conn: &mut DbConnection,
    data: &RequestProcessor,
) -> Result<(), RequestProcessorError> {
    let id_str = data.id.to_string();
    let user_id_str = data.user_id.to_string();
    let language = data.language.as_ref();
    let runtime = data.runtime.as_ref();
    sqlx::query!(
        r#"INSERT INTO request_processor
                    (id, name, language, runtime, code, user_id)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        id_str,
        data.name,
        language,
        runtime,
        data.code,
        user_id_str,
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Fetches a RequestProcessor for the given Uuid.
pub(crate) async fn get_request_processor(
    conn: &mut DbConnection,
    id: &Uuid,
) -> Result<RequestProcessor, RequestProcessorError> {
    let id_str = id.to_string();
    let row = sqlx::query!(r#"SELECT * FROM request_processor WHERE id = ?1"#, id_str)
        .fetch_one(conn)
        .await;

    match row {
        Err(x) => match x {
            sqlx::Error::RowNotFound => {
                return Err(RequestProcessorError::NotFound {
                    id: *id,
                    kind: "RequestProcessor".to_string(),
                })
            }
            _ => Err(RequestProcessorError::Db(x)),
        },
        Ok(row) => Ok(RequestProcessor {
            id: *id,
            name: row.name,
            language: RequestProcessorLanguage::from_str(&row.language)?,
            runtime: RequestProcessorRuntime::from_str(&row.runtime)?,
            code: row.code,
            user_id: row.user_id,
        }),
    }
}

/// Fetches all RequestProcessor for the given User UUID.
pub(crate) async fn get_request_processors(
    conn: &mut DbConnection,
    user_id: &String,
) -> Result<Vec<RequestProcessor>, RequestProcessorError> {
    let id_str = user_id.to_string();
    let mut items = Vec::new();

    let mut rows = sqlx::query("SELECT * FROM request_processor WHERE user_id = ?1")
        .bind(id_str)
        .fetch(conn);

    while let Some(row) = rows.try_next().await? {
        items.push(RequestProcessor {
            id: Uuid::from_str(row.try_get("id")?)?,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            language: RequestProcessorLanguage::from_str(row.try_get("language")?)?,
            runtime: RequestProcessorRuntime::from_str(row.try_get("runtime")?)?,
            user_id: row.try_get("user_id")?,
        });
    }

    Ok(items)
}

/// Updates a RequestProcessor with the given struct.
pub(crate) async fn update_request_processor(
    conn: &mut DbConnection,
    id: &Uuid,
    data: &mut RequestProcessor,
) -> Result<(), RequestProcessorError> {
    let _ = get_request_processor(conn, id).await?;
    let id_str = id.to_string();
    let language = data.language.as_ref();
    let runtime = data.runtime.as_ref();
    sqlx::query!(
        r#"UPDATE request_processor
           SET name=?1, language=?2, runtime=?3, code=?4
           WHERE id=?5"#,
        data.name,
        language,
        runtime,
        data.code,
        id_str,
    )
    .execute(conn)
    .await?;

    data.id = *id;

    Ok(())
}

/// Deletes a RequestProcessor with the given Uuid.
pub(crate) async fn delete_request_processor(
    conn: &mut DbConnection,
    id: &Uuid,
) -> Result<(), RequestProcessorError> {
    let _ = get_request_processor(conn, id).await?;
    let id_str = id.to_string();
    sqlx::query!(
        r#"DELETE FROM request_processor
           WHERE id=?1"#,
        id_str,
    )
    .execute(conn)
    .await?;

    Ok(())
}
