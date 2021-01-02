export RUST_BACKTRACE := "1"

dotenv:
    [ -f .env ] && echo "Will not overwrite existing .env file ..." || cp .env.example .env

db:
    sqlx db create
    sqlx migrate run

build:
    cargo build --verbose

run: dotenv db
    cargo run --bin fh-http

test:
    cargo test --verbose

test-e2e:
    pip install --requirement tests/requirements.txt
    pytest tests
