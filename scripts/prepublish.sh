#!/bin/bash

set -e

agave-install init edge
rm -rf target
cargo build
./scripts/build-test-programs.sh
cargo +nightly-2024-08-08 fmt --all -- --check
cargo +nightly-2024-08-08 clippy --all --all-features -- -D warnings
cargo test --all-features
