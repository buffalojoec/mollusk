#!/bin/bash

set -e

agave-install init 2.0.9
rm -rf target
cargo build
./scripts/build-test-programs.sh
cargo +nightly-2023-10-05 fmt --all -- --check
cargo +nightly-2023-10-05 clippy --all --all-features -- -D warnings
cargo test --all-features
