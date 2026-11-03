#!/usr/bin/env bash
# Runs from ui/
set -euo pipefail

rustup target add wasm32-unknown-unknown

# fetch the official Elm compiler binary.
curl -sSfL https://github.com/elm/compiler/releases/download/0.19.2/elm-0.19.2-linux-x64.gz \
  | gunzip > /usr/local/bin/elm
chmod +x /usr/local/bin/elm

# Prebuilt wasm-pack
curl -sSfL https://github.com/rustwasm/wasm-pack/releases/download/v0.15.1/wasm-pack-v0.15.1-x86_64-unknown-linux-musl.tar.gz \
  | tar xz --strip-components=1 -C /usr/local/bin wasm-pack-v0.15.1-x86_64-unknown-linux-musl/wasm-pack

(cd ../binding && wasm-pack build --target web --release)

corepack enable pnpm
COREPACK_ENABLE_DOWNLOAD_PROMPT=0 corepack pnpm install --frozen-lockfile
