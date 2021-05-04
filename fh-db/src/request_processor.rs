//! Database structs and functions for the RequestProcessor entity.
use super::RequestProcessorError;
use anyhow::Result;
use fh_core::DbConnection;
use serde::{self, Deserialize, Serialize};
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
    let language = data.language.as_ref();
    let runtime = data.runtime.as_ref();
    sqlx::query!(
        r#"INSERT INTO request_processor
                    (id, name, language, runtime, code)
                    VALUES ($1, $2, $3, $4, $5)"#,
        data.id,
        data.name,
        language,
        runtime,
        data.code
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
    let row = sqlx::query!(r#"SELECT * FROM request_processor WHERE id = $1"#, id)
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
        }),
    }
}

/// Updates a RequestProcessor with the given struct.
pub(crate) async fn update_request_processor(
    conn: &mut DbConnection,
    id: &Uuid,
    data: &mut RequestProcessor,
) -> Result<(), RequestProcessorError> {
    let _ = get_request_processor(conn, id).await?;
    let language = data.language.as_ref();
    let runtime = data.runtime.as_ref();
    sqlx::query!(
        r#"UPDATE request_processor
           SET name=$1, language=$2, runtime=$3, code=$4
           WHERE id=$5"#,
        data.name,
        language,
        runtime,
        data.code,
        id,
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
    sqlx::query!(
        r#"DELETE FROM request_processor
           WHERE id=$1"#,
        id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
