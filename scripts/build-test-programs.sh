#!/bin/bash

cargo build-sbf --manifest-path test-programs/cpi-target/Cargo.toml
cargo build-sbf --manifest-path test-programs/primary/Cargo.toml