pub mod request;
pub mod response;
mod runtime;

use crate::request::{Request, RequestList};
use crate::response::Response;
use crate::runtime::{prepare_runtime, prepare_user_code};
use anyhow::Result;
use std::collections::HashMap;

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
