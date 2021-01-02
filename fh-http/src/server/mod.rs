pub(crate) mod admin;
pub(crate) mod request_processor;

use crate::manager::{ReqCmd, ReqSender};
use crate::server::admin::filters::admin_filters;
use crate::server::request_processor::filters::request_filters;
use warp::Filter;

pub(crate) fn with_sender(
    tx: ReqSender<ReqCmd>,
) -> impl Filter<Extract = (ReqSender<ReqCmd>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

pub(crate) async fn web_server(tx: ReqSender<ReqCmd>) {
    warp::serve(request_filters(tx.clone()).or(admin_filters(tx.clone())))
        .run(([127, 0, 0, 1], 3030))
        .await
}
