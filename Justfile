export RUST_BACKTRACE := "1"
export RUST_LOG := "fh-core"

# The default recipe. Show all available recipes.
default:
    @echo
    @just --list --unsorted
    @echo

# Automatically create `.env` file from `.env.example` blueprint.
dotenv:
    @[ -f .env ] && echo "INFO: Will not overwrite existing .env file ..." || cp .env.example .env

# Invoke DB migrations. Also create database if it doesn't exist yet.
db: dotenv
    @test -e ~/.cargo/bin/sqlx || cargo install sqlx-cli
    @mkdir -p var/lib fh-db/var/lib
    sqlx db create
    sqlx migrate run

clean-db:
    rm -f ./var/lib/fh-http.db

# Build the Rust program.
build: db
    @echo
    @echo "INFO: Building the Rust program. On the first invocation, this might take quite a while."
    @echo
    cargo build

# Run the Rust program.
run: db
    cargo run --bin fh-http

# Run unit tests.
test:
    cargo test --lib --bin fh-http  


# ------------------------------------------
#          python environment
# ------------------------------------------
venvpath     := ".venv"
pip          := venvpath + "/bin/pip"
python       := venvpath + "/bin/python"
pytest       := venvpath + "/bin/pytest"
uvicorn      := venvpath + "/bin/uvicorn"

# Run e2e tests.
#  GATEWAY_PORT=3131 {{pytest}} tests -vvvv {{args}}
test-e2e *args: setup-virtualenv build
    @echo
    @echo "INFO: Running e2e tests."
    @echo
    CORE_PORT=3130 {{pytest}} tests -vvvv {{args}}

# Setup Python virtualenv. This is used for running e2e tests with pytest.
setup-virtualenv:
    #!/usr/bin/env sh
    if test ! -e {{python}}; then
        echo "INFO: Setting up Python virtualenv, this will take a moment."
        python3 -m venv {{venvpath}}
        {{pip}} install --upgrade pip
        {{pip}} install --requirement tests/requirements.txt
        {{pip}} install --requirement fh-gateway/requirements.txt
    fi

# Runs the fh-gateway python project
run-gateway:
    {{uvicorn}} fh.gateway.app:app --reload --app-dir fh-gateway --port 3031

# Tim's personal entrypoint for VSCODE. This way, it takes the freakin' DATABASE_URL env variable into account.
code:
    code .

# Jump into the sqlite3 shell
sqlite:
    sqlite3 ./var/lib/fh-http.db

# Install mermaid and generate mermaid graphs
mermaid:
    yarn add @mermaid-js/mermaid-cli
    ./node_modules/.bin/mmdc --input docs/crates.mmd --output docs/crates.png --backgroundColor transparent --theme forest

# Generate all docs
docs: mermaid
    cargo doc --no-deps
