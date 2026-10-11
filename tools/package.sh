#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --offline
app="dist/Cinema Black.app"
mkdir -p "$app/Contents/MacOS"
cp target/release/cinema-black "$app/Contents/MacOS/cinema-black"
cp resources/Info.plist "$app/Contents/Info.plist"
codesign --force --sign - "$app"
codesign --verify --strict "$app"
ditto -c -k --sequesterRsrc --keepParent "$app" dist/cinema-black-v1.0.0-macos-arm64.zip
(cd dist && shasum -a 256 cinema-black-v1.0.0-macos-arm64.zip > cinema-black-v1.0.0-macos-arm64.zip.sha256)
