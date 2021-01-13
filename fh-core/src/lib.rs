use sqlx::{pool::PoolConnection, Pool, Sqlite, SqlitePool};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot::error::RecvError;
use tokio::sync::{mpsc, oneshot};
use warp::{
    http::{self, HeaderValue},
    hyper::HeaderMap,
    reject::Reject,
};

pub mod request;
pub mod response;

pub type Responder<T> = oneshot::Sender<T>;
pub type ReqSender<T> = Arc<Mutex<mpsc::Sender<T>>>;
pub type DbPool<T> = Pool<T>;
pub type TypedPool = SqlitePool;
pub type DbType = Sqlite;
pub type DbConnection = PoolConnection<DbType>;

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
