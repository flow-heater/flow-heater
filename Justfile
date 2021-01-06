export RUST_BACKTRACE := "1"

dotenv:
    @[ -f .env ] && echo "INFO: Will not overwrite existing .env file ..." || cp .env.example .env

db: dotenv
    @test -e ~/.cargo/bin/sqlx || cargo install sqlx-cli
    @mkdir -p var/lib
    sqlx db create
    sqlx migrate run

build: db
    @echo
    @echo "INFO: Building the Rust program. On the first invocation, this might take quite a while."
    @echo
    cargo build

run: dotenv db
    cargo run --bin fh-http

test:
    cargo test --verbose


# ------------------------------------------
#          pytest-based e2e tests
# ------------------------------------------

venvpath     := ".venv"
pip          := venvpath + "/bin/pip"
python       := venvpath + "/bin/python"
pytest       := venvpath + "/bin/pytest"

test-e2e: setup-virtualenv build
    @echo "Running e2e tests."
    @{{pytest}} tests -vvvv

# Setup Python virtualenv
setup-virtualenv:
    #!/usr/bin/env sh
    if test ! -e {{python}}; then
        echo "INFO: Setting up Python virtualenv, this will take a moment."
        python3 -m venv {{venvpath}}
        {{pip}} install --upgrade pip
        {{pip}} install --requirement tests/requirements.txt
    fi
