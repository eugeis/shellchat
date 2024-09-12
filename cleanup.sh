#!/bin/bash
cargo test --all --all-targets
cargo clippy --all --all-targets -- -D warnings
cargo fmt --all