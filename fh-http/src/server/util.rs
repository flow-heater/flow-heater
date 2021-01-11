#[macro_export]
macro_rules! execute_command {
    ($tx_db: ident, $tx_proc: ident, $cmd: expr, $cmd_rx: ident) => {{
        let mut tx2 = $tx_proc
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        tx2.send($cmd)
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        $cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
    }};
    ($tx_db: ident, $cmd: expr, $cmd_rx: ident) => {{
        let mut tx2 = $tx_db
            .lock()
            .map_err(|e| warp::reject::custom(FhLockingError::new(e.to_string())))?
            .clone();

        tx2.send($cmd)
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?;

        $cmd_rx
            .await
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
            .map_err(|e| warp::reject::custom(FhHttpError::new(e)))?
    }};
}
