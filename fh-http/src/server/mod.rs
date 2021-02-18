// import this first, to have the macros available
#[macro_use]
mod util;

pub(crate) mod admin;
pub(crate) mod conversation;
pub(crate) mod error;
pub(crate) mod public;

use crate::server::admin::filters::admin_filters;
use crate::server::conversation::filters::conversation_filters;
use crate::server::public::filters::public_filters;
use fh_core::ReqSender;
use fh_db::ReqCmd;
use fh_v8::ProcessorCmd;
use warp::Filter;

/// Contain application specific configuration variables. This will include
/// further config options later.
#[derive(Debug)]
pub(crate) struct Config {
    /// Local port, the HTTP server will bind to.
    pub(crate) port: u16,
}

/// Async function to be run by an executor like tokio. Loads all endpoint
/// configurations and runs the server.
pub(crate) async fn web_server(ctx: AppContext, cfg: &Config) {
    let routes = public_filters(&ctx)
        .or(admin_filters(&ctx))
        .or(conversation_filters(&ctx))
        .recover(error::handle_rejections);

    warp::serve(routes).run(([127, 0, 0, 1], cfg.port)).await
}

/// Cheap clonable wrapper struct which contains cheap clonable references so
/// that it can be used to pass it around in warp filters.
#[derive(Debug, Clone)]
pub struct AppContext {
    tx_db: ReqSender<ReqCmd>,
    tx_proc: ReqSender<ProcessorCmd>,
}

impl AppContext {
    pub fn new(tx_db: ReqSender<ReqCmd>, tx_proc: ReqSender<ProcessorCmd>) -> Self {
        Self { tx_db, tx_proc }
    }
}
