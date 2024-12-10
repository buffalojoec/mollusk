#!/bin/bash

agave-install init 2.1.0
rm -rf target
cargo build
./scripts/build-test-programs.sh
cargo +nightly-2024-05-02 fmt --all -- --check
cargo +nightly-2024-05-02 clippy --all --all-features -- -D warnings
cargo test --all-features
