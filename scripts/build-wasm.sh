#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

WASM_BINDGEN_VERSION="$(
  awk '
    $0 == "name = \"wasm-bindgen\"" { found = 1; next }
    found && $1 == "version" {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' Cargo.lock
)"

WASM_BINDGEN_BIN="${WASM_BINDGEN:-}"
if [[ -z "$WASM_BINDGEN_BIN" ]]; then
  if command -v wasm-bindgen >/dev/null 2>&1; then
    WASM_BINDGEN_BIN="$(command -v wasm-bindgen)"
  elif [[ -x "$HOME/.cargo/bin/wasm-bindgen" ]]; then
    WASM_BINDGEN_BIN="$HOME/.cargo/bin/wasm-bindgen"
  fi
fi

if [[ -z "$WASM_BINDGEN_BIN" ]]; then
  echo "wasm-bindgen CLI is required: cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION}" >&2
  exit 1
fi

INSTALLED_VERSION="$("$WASM_BINDGEN_BIN" --version | awk '{ print $2 }')"
if [[ "$INSTALLED_VERSION" != "$WASM_BINDGEN_VERSION" ]]; then
  echo "wasm-bindgen ${WASM_BINDGEN_VERSION} is required, found ${INSTALLED_VERSION}" >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION}" >&2
  exit 1
fi

cargo build --release --target wasm32-unknown-unknown --lib
"$WASM_BINDGEN_BIN" \
  --target web \
  --out-dir "$ROOT/client/pkg" \
  "$ROOT/target/wasm32-unknown-unknown/release/packet_hacker.wasm"
