#!/usr/bin/env bash
# Builds the web app, opens it in the default browser, and stops serving
# once the page has fetched the wasm module.
#
# Usage: ./build-and-open.sh [port]   (default 8000)
set -euo pipefail

port="${1:-8000}"
web="$(cd "$(dirname "$0")" && pwd)"
root="$(dirname "$web")"

cargo build --target wasm32-unknown-unknown --manifest-path "$web/Cargo.toml"

# The wasm-bindgen CLI must match the wasm-bindgen crate version in Cargo.lock.
want="$(grep -A1 '^name = "wasm-bindgen"$' "$root/Cargo.lock" | sed -n 's/^version = "\(.*\)"$/\1/p')"
have=none
if command -v wasm-bindgen >/dev/null 2>&1; then
    have="$(wasm-bindgen --version | sed 's/^wasm-bindgen //')"
fi
if [ "$have" != "$want" ]; then
    echo "wasm-bindgen $want is required (found: $have). Install it with:"
    echo "  cargo install --locked wasm-bindgen-cli --version $want"
    exit 1
fi

wasm-bindgen --target web --out-dir "$web/pkg" \
    "$root/target/wasm32-unknown-unknown/debug/incredicalculator_web.wasm"

# Pick a browser opener: WSL helpers first, then the desktop default.
if command -v wslview >/dev/null 2>&1; then
    opener=wslview
elif command -v xdg-open >/dev/null 2>&1; then
    opener=xdg-open
elif command -v explorer.exe >/dev/null 2>&1; then
    opener=explorer.exe
else
    opener=
fi

exec python3 - "$web" "$port" "$opener" <<'EOF'
import http.server, os, subprocess, sys, time

web, port, opener = sys.argv[1], int(sys.argv[2]), sys.argv[3]
url = f"http://localhost:{port}/"

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".html": "text/html; charset=utf-8",
        ".js": "text/javascript",
        ".wasm": "application/wasm",
    }
    wasm_served = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=web, **kwargs)

    def end_headers(self):
        # Never cache, so every run picks up the fresh build.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_GET(self):
        super().do_GET()
        if self.path.split("?")[0].endswith(".wasm"):
            Handler.wasm_served = True

    def log_message(self, *args):
        pass

server = http.server.ThreadingHTTPServer(("localhost", port), Handler)
server.timeout = 0.25
print(f"Serving {url} until the page loads...")
if opener:
    # explorer.exe exits non-zero even on success, so ignore the status.
    subprocess.Popen([opener, url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
else:
    print(f"No browser opener found; open {url} manually.")

# Give up if the browser never fetches the wasm module.
deadline = time.monotonic() + 30
try:
    while not Handler.wasm_served:
        if time.monotonic() > deadline:
            sys.exit("Timed out waiting for the browser to load the page.")
        server.handle_request()
    print("Page loaded; server stopped.")
finally:
    server.server_close()
EOF
