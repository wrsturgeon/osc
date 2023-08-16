#!/usr/bin/env sh

set -eux

rustup toolchain install nightly
rustup component add miri --toolchain nightly

cargo fmt --check
cargo clippy --verbose --all-targets --all-features
RUST_BACKTRACE=full QUICKCHECK_TESTS=10000 cargo test --verbose --features=quickcheck
MIRIFLAGS=-Zmiri-backtrace=full cargo +nightly miri test
