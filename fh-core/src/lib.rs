use sqlx::{pool::PoolConnection, Pool, Sqlite, SqlitePool};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot::error::RecvError;
use tokio::sync::{mpsc, oneshot};
use warp::reject::Reject;

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
