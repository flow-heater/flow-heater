mod manager;
mod server;

use crate::manager::request_manager;
use crate::server::web_server;
use anyhow::Result;
use dotenv::dotenv;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    // TODO: is 4096 enough? make this configurable
    let (tx, mut rx) = mpsc::channel(4096);
    let (_web_server, req_manager) = tokio::join!(
        web_server(Arc::new(Mutex::new(tx))),
        request_manager(&mut rx),
    );

    req_manager?;

    Ok(())
}
