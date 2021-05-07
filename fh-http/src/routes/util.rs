#[macro_export]
macro_rules! db_cmd {
    ($tx_db: expr, $cmd: expr, $cmd_rx: ident) => {{
        let tx2 = $tx_db.clone();
        // TODO: replace unwrap() with correct actix specific error handling

        tx2.send($cmd).await.unwrap();

        $cmd_rx.await.unwrap().unwrap()
    }};
}

#[macro_export]
macro_rules! proc_cmd {
    ($tx_proc: expr, $cmd: expr, $cmd_rx: ident) => {{
        let tx2 = $tx_proc.clone();
        // TODO: replace unwrap() with correct actix specific error handling

        tx2.send($cmd).await.unwrap();

        $cmd_rx.await.unwrap().unwrap()
    }};
}
