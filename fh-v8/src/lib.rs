mod runtime;

use crate::runtime::{prepare_runtime, prepare_user_code};
use anyhow::{Error, Result};
use fh_core::{request::Request, response::Response, ReqSender, Responder};
use fh_db::{
    request_conversation::RequestConversation,
    request_processor::{RequestProcessor, RequestProcessorLanguage, RequestProcessorRuntime},
    ReqCmd, RequestProcessorError,
};
use runtime::RuntimeState;
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
        cmd_tx: Responder<Result<Response, RequestProcessorError>>,
        tx_db: ReqSender<ReqCmd>,
    },
    RunRequestProcessor {
        id: Uuid,
        request: Request,
        cmd_tx: Responder<Result<Response, RequestProcessorError>>,
        tx_db: ReqSender<ReqCmd>,
        prelude: bool,
    },
}

async fn process_command(cmd: ProcessorCmd) -> Result<()> {
    match cmd {
        ProcessorCmd::Http {
            request: req,
            cmd_tx,
            tx_db,
        } => {
            let req_proc_res = create_request_processor(
                tx_db.clone(),
                RequestProcessor {
                    id: Uuid::new_v4(),
                    name: "temporary".to_string(),
                    language: RequestProcessorLanguage::Javascript,
                    runtime: RequestProcessorRuntime::V8,
                    code: prepare_user_code(include_str!("flow_heater.js"), true),
                },
            )
            .await;

            let req_proc = match req_proc_res {
                Err(err) => {
                    cmd_tx.send(Err(err)).map_err(|e| {
                        Error::msg(format!(
                            "Unable to send Response to server handler: {:?}",
                            e
                        ))
                    })?;

                    return Ok(());
                }
                Ok(procc) => procc,
            };

            let conversation_res = create_request_conversation(tx_db.clone(), req_proc.id).await;
            let conversation_id = match conversation_res {
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

            let res = process_request(tx_db.clone(), req, conversation_id, req_proc.code)
                .await
                .map_err(RequestProcessorError::Processing);

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
            prelude,
        } => {
            let conversation_res = create_request_conversation(tx_db.clone(), id).await;
            let conversation_id = match conversation_res {
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

            let req_proc_res = get_request_processor(tx_db.clone(), id).await;

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

            if prelude {}

            let r = process_request(
                tx_db.clone(),
                request,
                conversation_id,
                prepare_user_code(&request_processor.code, prelude),
            )
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
async fn create_request_processor(
    tx_db: ReqSender<ReqCmd>,
    proc: RequestProcessor,
) -> Result<RequestProcessor, RequestProcessorError> {
    let mut tx_db2 = tx_db
        .lock()
        .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
        .clone();

    let (cmd_tx2, cmd_rx2) = oneshot::channel();

    tx_db2
        .send(ReqCmd::CreateRequestProcessor {
            cmd_tx: cmd_tx2,
            proc,
        })
        .await
        .map_err(anyhow::Error::new)?;

    cmd_rx2
        .await
        .map_err(|_| Error::msg(format!("Unable to send () to server handler")))?
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

pub async fn process_request(
    tx_db: ReqSender<ReqCmd>,
    req: Request,
    conversation_id: Uuid,
    code: String,
) -> Result<Response> {
    let mut js_runtime = prepare_runtime(tx_db, req.clone(), conversation_id);
    js_runtime.execute("custom_code.js", &prepare_user_code(&code, false))?;
    js_runtime.run_event_loop().await?;

    // extract the requests
    let state = js_runtime.op_state();
    let op_state = state.borrow();
    let rt_state = op_state.borrow::<RuntimeState>();

    // println!("Requests: {:?}", requests);

    let mut response_headers = HashMap::new();
    response_headers.insert(
        "FH-Conversation-Id".to_string(),
        conversation_id.to_string(),
    );

    Ok(Response {
        code: 200,
        headers: response_headers,
        body: Some(
            rt_state
                .request_list
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
