#!/usr/bin/env sh

set -eux

rustup toolchain install nightly
rustup component add miri --toolchain nightly

MIRIFLAGS=-Zmiri-backtrace=full cargo +nightly miri test
