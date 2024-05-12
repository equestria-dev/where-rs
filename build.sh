#!/bin/bash

whered_version=$(cargo pkgid whered | tr "#" " " | awk '{print $2;}')
where_version=$(cargo pkgid where-rs | tr "#" " " | awk '{print $2;}')

cargo clean
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release
cargo build --target x86_64-unknown-linux-gnu --release

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/aarch64-apple-darwin/release/whered \
https://source.equestria.dev/api/v4/projects/178/packages/generic/whered/$whered_version/whered-darwin-aarch64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/aarch64-apple-darwin/release/where \
https://source.equestria.dev/api/v4/projects/178/packages/generic/where/$where_version/where-darwin-aarch64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/x86_64-apple-darwin/release/whered \
https://source.equestria.dev/api/v4/projects/178/packages/generic/whered/$whered_version/whered-darwin-x86_64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/x86_64-apple-darwin/release/where \
https://source.equestria.dev/api/v4/projects/178/packages/generic/where/$where_version/where-darwin-x86_64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/aarch64-unknown-linux-gnu/release/whered \
https://source.equestria.dev/api/v4/projects/178/packages/generic/whered/$whered_version/whered-linux-aarch64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/x86_64-unknown-linux-gnu/release/whered \
https://source.equestria.dev/api/v4/projects/178/packages/generic/whered/$whered_version/whered-linux-x86_64

curl -v --header "PRIVATE-TOKEN: $(cat ~/.deploy.txt)" \
--header "Content-Type: multipart/form-data" \
--upload-file ./target/x86_64-unknown-linux-gnu/release/where \
https://source.equestria.dev/api/v4/projects/178/packages/generic/where/$where_version/where-linux-x86_64
