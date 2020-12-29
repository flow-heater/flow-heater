use anyhow::Result;
use fh_v8::{process_request, Request, Response};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use warp::{self, http, Filter, Rejection, Reply};

type Responder<T> = oneshot::Sender<T>;
type ReqSender<T> = Arc<Mutex<mpsc::Sender<T>>>;

#[derive(Debug)]
enum ReqCmd {
    Http {
        request: Request,
        cmd_tx: Responder<Result<Response>>,
    },
}

async fn request_processor(rx: &mut mpsc::Receiver<ReqCmd>) {
    while let Some(cmd) = rx.recv().await {
        println!("Got new CMD: {:?}", cmd);

        match cmd {
            ReqCmd::Http {
                request: req,
                cmd_tx: resp_tx,
            } => {
                let res = process_request(&req).await;
                resp_tx.send(Ok(res)).unwrap();
            }
        }
    }
}

pub(crate) fn with_sender(
    tx: ReqSender<ReqCmd>,
) -> impl Filter<Extract = (ReqSender<ReqCmd>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

pub fn extract_request(
) -> impl Filter<Extract = (http::Request<Vec<u8>>,), Error = warp::Rejection> + Copy {
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
                req
            },
        )
}

async fn hello_handler(
    _name: String,
    tx: ReqSender<ReqCmd>,
    request: http::Request<Vec<u8>>,
) -> Result<impl Reply, Rejection> {
    let mut tx2 = tx.lock().unwrap().clone();
    let (resp_tx, resp_rx) = oneshot::channel();

    let req = Request::from(request);
    tx2.send(ReqCmd::Http {
        request: req,
        cmd_tx: resp_tx,
    })
    .await
    .unwrap();
    let res = resp_rx.await;
    println!("GOT response: {:?}", res);
    Ok(warp::reply::reply())
}

async fn web_server(tx: ReqSender<ReqCmd>) {
    // GET /hello/{name} => 200 OK with body "Hello, {name}!"
    let hello = warp::path!("hello" / String)
        .and(with_sender(tx.clone()))
        .and(extract_request())
        .and_then(hello_handler);

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(4096);
    tokio::join!(
        web_server(Arc::new(Mutex::new(tx))),
        request_processor(&mut rx),
    );
}
