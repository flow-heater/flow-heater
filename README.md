# Flow Heater

## Synopsis
This project provides an HTTP service, which runs custom programmable (JavaScript) code on incoming HTTP requests.
Users can write their own JavaScript snippets and forward HTTP requests to these programmable endpoints. 

Flow Heater is open source, licensed under GNU AGPL v3. Currently, you can run only run Flow Heater on self-hosted installations. Flow Heater is written in Rust and therefore aims to be secure, fast and easily cross-platform deployable as a single static binary. 

Flow Heater currently uses SQLite as a database, which will soon be replaced by a fully-fledged PostgreSQL solution.

## Usage
### Install Prerequesites
For running Flow Heater, you need 
- a full rust toolchain (incl. cargo) installed via [rustup](https://rustup.rs/)
- the awesome Command Runner [Just](https://github.com/casey/just#installation).
- For local development:
  - python3.8 for the e2e-test suite

### Running Flow Heater
After the installation of the prerequesites, you can just run Flow Heater with:
```bash
just run
```

This builds and starts the local http server on port `3030`.

## REST API
The Flow Heater REST API is documented in [API.md](fh-http/API.md).

## Architecture
Please see details in [architecture.md](docs/architecture.md).

## Local development / testing
We use for the development currently the vscode editor + rust-analyzer extension. To start the vscode with the correct environment variables run:
```bash
just code
```

Invoke unit tests:
```bash
just test
```

Invoke end-to-end tests:
```bash
just test-e2e
```

Invoke specific tests:
```bash
# Address tests having `example` in their name.
just test-e2e -k example

# Address tests marked with `@pytest.mark.admin`.
just test-e2e -m admin
```

## Documentation
Building graphs + documentation usually works with:
```bash
just docs
```

If you work on on WSL2, you usually need to install these libs before mermaid / puppeteer is working (thanks to [this article](https://chaosandpenguins.com/blog/2021/01/15/troubleshooting-puppeteer-in-wsl2/)):
- libnss3
- libatk-adaptor
- libcups2
- libxkbcommon0
- libgtk-3-0
- libgbm1


## Credits
Flow Heater uses the following great libraries / projects
- [tokio](https://tokio.rs/) as the async runtime
- [warp](https://github.com/seanmonstar/warp) as http server
- [deno_core](https://github.com/denoland/deno) as v8 runtime wrapper
- [sqlx](https://github.com/launchbadge/sqlx) as database abstraction

## Example
For an example, please see [example.md](docs/example.md).
