test:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo audit
    cargo deny check all
    cargo test

pre-commit:
    ./scripts/scan-staged-secrets.sh
    cargo fmt --all
    cargo clippy --all-targets --all-features -- -D warnings
    cargo audit
    cargo deny check all
    cargo test

release *args:
    git pull --rebase
    cargo release {{args}}

install:
    cargo install --path .
