mod manager;
mod server;

use crate::manager::request_manager;
use crate::server::web_server;
use dotenv::dotenv;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (tx, mut rx) = mpsc::channel(4096);
    tokio::join!(
        web_server(Arc::new(Mutex::new(tx))),
        request_manager(&mut rx),
    );
}
