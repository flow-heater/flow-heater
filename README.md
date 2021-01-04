# Flow Heater HTTP

## Setup
install sqlx-cli:
```bash
cargo install sqlx-cli
```

Migrations are run automatically, when running the `just run` target.

## Example
Compile and run the `fh-http` binary with:

```bash
just run
```

On another terminal, issue an HTTP request to `localhost:3030/hello/foo`:

```bash
$> curl -iX OPTIONS localhost:3030/hello/xxx -d'{"a":"b"}'
HTTP/1.1 200 OK
content-type: application/json
content-length: 123
date: Sat, 02 Jan 2021 12:52:27 GMT

{"code":200,"headers":{},"body":[116,104,105,115,32,105,115,32,116,104,101,32,112,97,116,99,104,101,100,32,98,111,100,121]}
```

The `fh-http` process spits out some lines to stdout.
```bash
$> cargo run --bin fh-http
   Compiling fh-http v0.1.0 (/home/tim/projects/flow-heater/workspace/fh-http)
    Finished dev [unoptimized + debuginfo] target(s) in 3.75s
     Running `target/debug/fh-http`
DENO: Got request body: {"a":"b"}, content-type header: application/x-www-form-urlencoded, method: OPTIONS
RUST: modified request is: Request { headers: {"content-length": "9", "user-agent": "curl/7.58.0", "accept": "*/*", "content-type": "application/json", "host": "localhost:3030"}, body: "this is the patched body", method: "POST", path: "/hello/xxx", query: "" }
```

As you can see in the `modified request`, the method and the body is patched, as well as the `content-type` header.


## Tests
Invoke unit tests:
```bash
just test
```

Invoke end-to-end tests:
```bash
just test-e2e
```
