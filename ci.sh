#!/usr/bin/env sh

set -eux

rustup toolchain install nightly
rustup component add miri --toolchain nightly

cargo fmt --check
cargo clippy --verbose --all-targets
MIRIFLAGS=-Zmiri-backtrace=full cargo +nightly miri test
