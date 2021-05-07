use actix_web::http::HeaderMap as ActixHeaderMap;
use sqlx::{pool::PoolConnection, postgres::PgPool, Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot::error::RecvError;
use tokio::sync::{mpsc, oneshot};
use warp::{
    http::{self, HeaderValue},
    hyper::HeaderMap,
    reject::Reject,
};

pub mod request;
pub mod response;

/// Generic type alias for oneshot Responders.
pub type Responder<T> = oneshot::Sender<T>;

/// Generic type alias for cross-thread clonable mpsc::Senders.
pub type ReqSender<T> = Arc<mpsc::Sender<T>>;

/// Generic type alias for the SQLX DB Pool.
pub type DbPool<T> = Pool<T>;

/// Alias for a typed database pool. Needed to easily change to a different pool
/// type later on.
pub type TypedPool = PgPool;

/// Alias for a typed database. Needed to easily change to a different database
/// type later on.
pub type DbType = Postgres;

/// Alias for the database connection.
pub type DbConnection = PoolConnection<DbType>;

/// Locking Error, used in the warp rejection handling.
#[derive(Debug)]
pub struct FhLockingError<T> {
    err: T,
}

impl<T> FhLockingError<T> {
    pub fn new(err: T) -> Self {
        Self { err }
    }
}

impl Reject for FhLockingError<String> {}
impl Reject for FhLockingError<anyhow::Error> {}

/// ProcessorError, used in the warp rejection handling.
#[derive(Debug)]
pub struct FhProcessorError<T> {
    err: T,
}

impl<T> FhProcessorError<T> {
    pub fn new(err: T) -> Self {
        Self { err }
    }
}

impl Reject for FhProcessorError<String> {}
impl Reject for FhProcessorError<anyhow::Error> {}
impl Reject for FhProcessorError<RecvError> {}

/// Utility function which tries to convert the given
/// [`http::header::map::HeaderMap`] to a serializable HashMap.
fn try_header_map_to_hashmap(
    hm: HeaderMap<HeaderValue>,
) -> Result<HashMap<String, Vec<String>>, anyhow::Error> {
    let mut res: HashMap<String, Vec<String>> = HashMap::new();
    hm.keys().for_each(|k| {
        hm.get_all(k).into_iter().for_each(|v| {
            res.entry(k.to_string())
                .and_modify(|e| e.push(v.to_str().unwrap_or("").to_string()))
                .or_insert(vec![v.to_str().unwrap_or("").to_string()]);
        })
    });

    Ok(res)
}

fn try_actix_header_map_to_hashmap(
    hm: ActixHeaderMap,
) -> Result<HashMap<String, Vec<String>>, anyhow::Error> {
    let mut res: HashMap<String, Vec<String>> = HashMap::new();
    hm.keys().for_each(|k| {
        hm.get_all(k).into_iter().for_each(|v| {
            res.entry(k.to_string())
                .and_modify(|e| e.push(v.to_str().unwrap_or("").to_string()))
                .or_insert(vec![v.to_str().unwrap_or("").to_string()]);
        })
    });

    Ok(res)
}

/// Utility function to convert [`http::Version`] variants to simple strings.
fn version_to_string(v: http::Version) -> String {
    match v {
        http::Version::HTTP_09 => "HTTP/0.9",
        http::Version::HTTP_10 => "HTTP/1.0",
        http::Version::HTTP_11 => "HTTP/1.1",
        http::Version::HTTP_2 => "HTTP/2",
        http::Version::HTTP_3 => "HTTP/3",
        _ => unreachable!(),
    }
    .to_string()
}
