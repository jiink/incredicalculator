#!/usr/bin/env bash
# Builds the web app, opens it in the default browser, and keeps serving
# until Ctrl+C, so you can move between the home page, sim/ and editor/.
# build-and-open.sh is the same but stops serving once the page has loaded.
#
# Usage: ./build-and-serve.sh [port] [page]   (as for build-and-open.sh)
set -euo pipefail
INCREDICALC_KEEP_SERVING=1 exec "$(dirname "$0")/build-and-open.sh" "$@"
