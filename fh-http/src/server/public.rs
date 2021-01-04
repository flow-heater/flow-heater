pub(crate) mod filters {
    use crate::manager::{ReqCmd, ReqSender};
    use crate::server::with_sender;
    use fh_v8::Request;
    use warp::{http, Filter};

    pub(crate) fn public_filters(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        process_request(tx.clone())
    }

    fn extract_request() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Copy {
        warp::method()
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::body::bytes())
            .map(
                |method: http::Method,
                 path: warp::path::FullPath,
                 headers: http::HeaderMap,
                 body: warp::hyper::body::Bytes| {
                    let mut req = http::Request::builder()
                        .method(method)
                        .uri(path.as_str())
                        .body(body.iter().cloned().collect::<Vec<u8>>())
                        .expect("request builder");
                    {
                        *req.headers_mut() = headers;
                    }

                    Request::from(req)
                },
            )
    }

    pub(crate) fn process_request(
        tx: ReqSender<ReqCmd>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("hello" / String)
            .and(with_sender(tx.clone()))
            .and(extract_request())
            .and_then(super::handlers::process_request)
    }
}

pub(crate) mod handlers {
    use crate::manager::{ReqCmd, ReqSender};
    use fh_v8::Request;
    use tokio::sync::oneshot;
    use warp::{Rejection, Reply};

    pub(crate) async fn process_request(
        _name: String,
        tx: ReqSender<ReqCmd>,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let mut tx2 = tx.lock().unwrap().clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(ReqCmd::Http {
            request,
            cmd_tx: resp_tx,
        })
        .await
        .unwrap();
        let res = resp_rx.await.unwrap().unwrap();
        // println!("GOT response: {:?}", res);
        Ok(warp::reply::json(&res))
    }
}
