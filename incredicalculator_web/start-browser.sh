#!/usr/bin/env bash
set -eu

PORT="${PORT:-8000}"

cd "$(dirname "$0")"

if ! command -v nix-shell >/dev/null 2>&1; then
  echo "nix-shell is required to run this local browser server."
  echo "Install Nix first, then rerun this script."
  exit 1
fi

release_port() {
  local port="$1"
  local pids=""

  if command -v ss >/dev/null 2>&1; then
    pids="$(ss -lntp "sport = :$port" 2>/dev/null | awk 'NR>1 {print $NF}' | sed -n 's/.*pid=\([0-9][0-9]*\).*/\1/p' | sort -u | tr '\n' ' ' || true)"
  fi

  if [ -z "$pids" ] && command -v lsof >/dev/null 2>&1; then
    pids="$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"
  fi

  if [ -z "$pids" ] && command -v fuser >/dev/null 2>&1; then
    pids="$(fuser -n tcp "$port" 2>/dev/null || true)"
  fi

  if [ -n "$pids" ]; then
    echo "Port $port is in use; terminating stale listener(s): $pids"
    kill -9 $pids || true
  fi
}

release_port "$PORT"

if [ ! -f "pkg/incredicalculator_web.js" ] || [ ! -f "pkg/incredicalculator_web_bg.wasm" ]; then
  echo "Browser bundle not found; building it now..."
  nix-shell -p wasm-pack --run 'wasm-pack build --target web --out-dir pkg'
fi

echo "Serving the app at http://127.0.0.1:$PORT"
echo "Press Ctrl+C to stop."

echo "If you prefer another port, run: PORT=4173 ./start-browser.sh"

nix-shell -p python3 --run "python3 -m http.server $PORT --bind 127.0.0.1"
