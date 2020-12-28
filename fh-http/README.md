# Flow Heater HTTP

## Example

Compile and run the `fh-http` binary with:

```bash
cargo run --bin fh-http
```

On another terminal, issue an HTTP request to `localhost:3030/hello/foo`:

```bash
curl "localhost:3030/hello/foo"
```

The `fh-http` process spits out some lines to stdout including a simple `"Hello from deno!"` line, which should indicate, that the deno runtime runs asynchronously behind the warp http server.
```bash
$> cargo run --bin fh-http
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/fh-http`
Got new CMD: Http { request: Request { headers: {}, body: None }, cmd_tx: Sender { inner: Some(Inner { state: State { is_complete: false, is_closed: false, is_rx_task_set: true, is_tx_task_set: false } }) } }
Hello from deno!
GOT response: Ok(Ok(Response { code: 200, headers: {}, body: None }))
```
