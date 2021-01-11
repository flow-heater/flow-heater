#[macro_export]
macro_rules! execute_command {
    ($tx_db: expr, $cmd: expr, $cmd_rx: ident) => {{
        let mut tx_db2 = $tx_db
            .lock()
            .map_err(|e| RequestProcessorError::Locking(e.to_string()))?
            .clone();

        tx_db2.send($cmd).await.map_err(anyhow::Error::new)?;

        // HINT: never omit awaiting here... this leads to runtime hangs!
        // TODO: error handling?
        $cmd_rx.await??;
    }};
}
