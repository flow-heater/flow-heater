export RUST_BACKTRACE := "1"

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
#          pytest-based e2e tests
# ------------------------------------------


venvpath     := ".venv"
pip          := venvpath + "/bin/pip"
python       := venvpath + "/bin/python"
pytest       := venvpath + "/bin/pytest"

# Run e2e tests.
test-e2e *args: setup-virtualenv build
    @echo
    @echo "INFO: Running e2e tests."
    @echo
    {{pytest}} tests -vvvv {{args}}

# Setup Python virtualenv. This is used for running e2e tests with pytest.
setup-virtualenv:
    #!/usr/bin/env sh
    if test ! -e {{python}}; then
        echo "INFO: Setting up Python virtualenv, this will take a moment."
        python3 -m venv {{venvpath}}
        {{pip}} install --upgrade pip
        {{pip}} install --requirement tests/requirements.txt
    fi

# Tim's personal entrypoint for VSCODE. This way, it takes the freakin' DATABASE_URL env variable into account.
code:
    code ./workspace.code-workspace

sqlite:
    sqlite3 ./var/lib/fh-http.db

mermaid:
    yarn add @mermaid-js/mermaid-cli
    ./node_modules/.bin/mmdc --input docs/crates.mmd --output docs/crates.png --backgroundColor transparent --theme forest

docs: mermaid
    cargo doc --no-deps
