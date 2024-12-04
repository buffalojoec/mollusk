#!/bin/bash

agave-install init 1.18.26
rm -rf target
cargo build
./scripts/build-test-programs.sh
cargo +nightly fmt --all -- --check
cargo +nightly clippy --all --all-features -- -D warnings
cargo test --all-features
