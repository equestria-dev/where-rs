#!/bin/bash

whered_version=$(cargo pkgid whered | tr "#" " " | awk '{print $2;}')
where_version=$(cargo pkgid where-rs | tr "#" " " | awk '{print $2;}')
rustc_version=$(rustc --version | awk '{print $2;}')

cargo clean
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release
cargo build --target x86_64-unknown-linux-gnu --release
