use super::RequestProcessorError;
use anyhow::Result;
use fh_core::DbConnection;
use serde::{self, Deserialize, Serialize};
use std::{convert::AsRef, str::FromStr};
use strum_macros::{self, AsRefStr, EnumString};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum RequestProcessorLanguage {
    Javascript,
    Typescript,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum RequestProcessorRuntime {
    V8,
    WASM,
}

pub(crate) async fn create_request_processor(
    conn: &mut DbConnection,
    data: &RequestProcessor,
) -> Result<(), RequestProcessorError> {
    let id_str = data.id.to_string();
    let language = data.language.as_ref();
    let runtime = data.runtime.as_ref();
    sqlx::query!(
        r#"INSERT INTO request_processor
                    (id, name, language, runtime, code)
                    VALUES (?1, ?2, ?3, ?4, ?5)"#,
        id_str,
        data.name,
        language,
        runtime,
        data.code
    )
    .execute(conn)
    .await?;

    Ok(())
}

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
        }),
    }
}

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
