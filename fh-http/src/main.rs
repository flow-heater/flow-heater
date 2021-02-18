mod server;

use crate::server::web_server;
use anyhow::Result;
use dotenv::dotenv;
use fh_db::request_manager;
use fh_v8::request_processing_manager;
use server::{AppContext, Config};
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let config = Config {
        port: env::var("CORE_PORT")
            .unwrap_or("3030".into())
            .parse::<u16>()?,
    };

    // TODO is 4096 enough? make this configurable!
    let (tx_db, mut rx_db) = mpsc::channel(4096);
    let (tx_v8, mut rx_v8) = mpsc::channel(4096);
    let ctx = AppContext::new(Arc::new(Mutex::new(tx_db)), Arc::new(Mutex::new(tx_v8)));

    let (_web_server, req_manager, req_proc_manager) = tokio::join!(
        web_server(ctx, &config),
        request_manager(&mut rx_db),
        request_processing_manager(&mut rx_v8)
    );

    req_manager?;
    req_proc_manager?;

    Ok(())
}
