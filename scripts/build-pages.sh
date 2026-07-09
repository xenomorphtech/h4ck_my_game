#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${PAGES_DIST:-$ROOT/dist}"

"$ROOT/scripts/build-wasm.sh"

rm -rf "$DIST"
mkdir -p "$DIST"
cp -R "$ROOT/client/." "$DIST/"

# The Axum app serves the checked-in client under /client/*. GitHub Pages serves
# this artifact from the site root, including project paths like /repo-name/.
perl -0pi -e 's#/client/#./#g' "$DIST/index.html"
perl -0pi -e 's#/client/icons/#./icons/#g' "$DIST/scene.js"

touch "$DIST/.nojekyll"
