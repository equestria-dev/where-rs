#!/bin/bash

cargo clean
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release
cargo build --target x86_64-unknown-linux-gnu --release
