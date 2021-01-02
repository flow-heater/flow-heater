build:
    cargo build --verbose

run:
    cargo run --bin fh-http

test:
    cargo test --verbose

test-e2e:
    pip install --requirement tests/requirements.txt
    pytest tests
