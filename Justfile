set shell := ["bash", "-c"]

mod devnet "just/devnet.just"
mod scripts "just/scripts.just"

check:
    cargo check --workspace
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace
