#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

if ! command -v nix-shell >/dev/null 2>&1; then
  echo "nix-shell is required to run this local browser server."
  echo "Install Nix first, then rerun this script."
  exit 1
fi

if [ ! -f "pkg/incredicalculator_web.js" ] || [ ! -f "pkg/incredicalculator_web_bg.wasm" ]; then
  echo "Browser bundle not found; building it now..."
  nix-shell -p wasm-pack --run 'wasm-pack build --target web --out-dir pkg'
fi

echo "Serving the app at http://127.0.0.1:8000"
echo "Press Ctrl+C to stop."

nix-shell -p python3 --run 'python3 -m http.server 8000 --bind 127.0.0.1'
