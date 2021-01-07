mod runtime;

use crate::runtime::{prepare_runtime, prepare_user_code};
use anyhow::{Error, Result};
use fh_core::{
    request::{Request, RequestList},
    response::Response,
    ReqSender, Responder,
};
use fh_db::{
    request_conversation::RequestConversation, request_processor::RequestProcessor, ReqCmd,
    RequestProcessorError,
};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub async fn request_processing_manager(
    rx: &mut mpsc::Receiver<ProcessorCmd>,
) -> anyhow::Result<()> {
    while let Some(cmd) = rx.recv().await {
        process_command(cmd).await?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum ProcessorCmd {
    Http {
        request: Request,
        cmd_tx: Responder<Result<Response, anyhow::Error>>,
    },
    RunRequestProcessor {
        id: Uuid,
        request: Request,
        cmd_tx: Responder<Result<Response, RequestProcessorError>>,
        tx_db: ReqSender<ReqCmd>,
    },
}

async fn process_command(cmd: ProcessorCmd) -> Result<()> {
    match cmd {
        ProcessorCmd::Http {
            request: req,
            cmd_tx,
        } => {
            let res = process_request(req, None).await;
            cmd_tx.send(res).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
        ProcessorCmd::RunRequestProcessor {
            id,
            request,
            cmd_tx,
            tx_db,
        } => {
            let conversation_res = create_request_conversation(tx_db.clone(), id).await;
            let _conversation_id = match conversation_res {
                Err(err) => {
                    cmd_tx.send(Err(err)).map_err(|e| {
                        Error::msg(format!(
                            "Unable to send Response to server handler: {:?}",
                            e
                        ))
                    })?;

                    return Ok(());
                }
                Ok(conv) => conv.id,
            };

            let req_proc_res = get_request_processor(tx_db, id).await;

            let request_processor = match req_proc_res {
                Err(err) => {
                    cmd_tx.send(Err(err)).map_err(|e| {
                        Error::msg(format!(
                            "Unable to send Response to server handler: {:?}",
                            e
                        ))
                    })?;

                    return Ok(());
                }
                Ok(req_proc) => req_proc,
            };

            let r = process_request(request, Some(request_processor.code))
                .await
                .map_err(RequestProcessorError::Processing);

            cmd_tx.send(r).map_err(|e| {
                Error::msg(format!(
                    "Unable to send Response to server handler: {:?}",
                    e
                ))
            })?;
        }
    }

    Ok(())
}

async fn create_request_conversation(
    tx_db: ReqSender<ReqCmd>,
    request_processor_id: Uuid,
) -> Result<RequestConversation, RequestProcessorError> {
    let mut tx_db2 = tx_db
        .lock()
        .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
        .clone();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();

    tx_db2
        .send(ReqCmd::CreateRequestConversation {
            request_processor_id: request_processor_id,
            cmd_tx: cmd_tx2,
        })
        .await
        .map_err(anyhow::Error::new)?;

    cmd_rx2
        .await
        .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?
}

async fn get_request_processor(
    tx_db: ReqSender<ReqCmd>,
    id: Uuid,
) -> Result<RequestProcessor, RequestProcessorError> {
    let mut tx_db2 = tx_db
        .lock()
        .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
        .clone();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();

    tx_db2
        .send(ReqCmd::GetRequestProcessor {
            id,
            cmd_tx: cmd_tx2,
        })
        .await
        .map_err(anyhow::Error::new)?;

    cmd_rx2
        .await
        .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?
}

pub async fn process_request(req: Request, code: Option<String>) -> Result<Response> {
    let mut js_runtime = prepare_runtime(req.clone());

    if let Some(c) = code {
        js_runtime.execute("custom_code.js", &prepare_user_code(&c, false))?;
    } else {
        js_runtime.execute(
            "flow_heater.js",
            &prepare_user_code(include_str!("flow_heater.js"), true),
        )?;
    }

    js_runtime.run_event_loop().await?;

    // extract the requests
    let state = js_runtime.op_state();
    let op_state = state.borrow();
    let requests = op_state.borrow::<RequestList>();

    // println!("Requests: {:?}", requests);

    Ok(Response {
        code: 200,
        headers: HashMap::new(),
        body: Some(
            requests
                .iter()
                .last()
                .cloned()
                .unwrap_or(req)
                .body
                .as_bytes()
                .to_vec(),
        ),
    })
}
