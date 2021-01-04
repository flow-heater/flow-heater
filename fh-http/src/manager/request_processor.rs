use anyhow::Result;
use fh_v8::{process_request, Request, Response};
use serde::{self, Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Sqlite};
use std::convert::AsRef;
use std::str::FromStr;
use strum_macros::{self, AsRefStr, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RequestProcessor {
    #[serde(skip_deserializing)]
    #[serde(default = "Uuid::new_v4")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) language: RequestProcessorLanguage,
    pub(crate) runtime: RequestProcessorRuntime,
    pub(crate) code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RequestProcessorLanguage {
    Javascript,
    Typescript,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RequestProcessorRuntime {
    V8,
    WASM,
}

pub(crate) async fn create_request_processor(
    conn: &mut PoolConnection<Sqlite>,
    data: &RequestProcessor,
) -> Result<()> {
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
    conn: &mut PoolConnection<Sqlite>,
    id: &Uuid,
) -> Result<RequestProcessor> {
    let id_str = id.to_string();
    let row = sqlx::query!(r#"SELECT * FROM request_processor WHERE id = ?1"#, id_str)
        .fetch_one(conn)
        .await?;

    Ok(RequestProcessor {
        id: Uuid::from_str(&row.id)?,
        name: row.name,
        language: RequestProcessorLanguage::from_str(&row.language)?,
        runtime: RequestProcessorRuntime::from_str(&row.runtime)?,
        code: row.code,
    })
}

pub(crate) async fn update_request_processor(
    conn: &mut PoolConnection<Sqlite>,
    id: &Uuid,
    data: &mut RequestProcessor,
) -> Result<()> {
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
    conn: &mut PoolConnection<Sqlite>,
    id: &Uuid,
) -> Result<()> {
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

pub(crate) async fn run_request_processor(
    conn: &mut PoolConnection<Sqlite>,
    id: &Uuid,
    request: Request,
) -> Result<Response> {
    let p = get_request_processor(conn, id).await?;
    let res = process_request(request, Some(p.code)).await;

    Ok(res)
}
