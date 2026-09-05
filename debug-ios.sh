#!/usr/bin/env bash
#
# debug-ios.sh — Start the iOS Safari remote debugging environment from Linux.
#
# Starts:
#   1. ios-webkit-debug-proxy (bridges USB <-> WebSocket)
#   2. A static server serving the WebKit WebInspector UI
#   3. Opens the browser at the inspector overview page
#
# Prerequisites (one-time):
#   - ~/tools/ios-safari-remote-debug-kit set up (run generate.sh once)
#   - iPhone connected via USB, unlocked, and trusted
#   - Settings > Safari > Advanced > Web Inspector = ON
#   - A Safari tab open with the page you want to debug
#
# Usage:
#   ./debug-ios.sh                # start everything and open the browser
#   ./debug-ios.sh --no-browser   # start without opening the browser
#   ./debug-ios.sh <UDID>         # target a specific device
#
set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
TOOL_DIR="${IOS_DEBUG_TOOL_DIR:-$HOME/tools/ios-safari-remote-debug-kit/src}"
WEBINSPECTOR_DIR="$TOOL_DIR/WebKit/Source/WebInspectorUI/UserInterface"
HTTP_PORT="${IOS_DEBUG_HTTP_PORT:-8080}"
PROXY_PORT=9222
OPEN_BROWSER=true

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
info()  { echo -e "${CYAN}[debug-ios]${NC} $*"; }
ok()    { echo -e "${GREEN}[debug-ios]${NC} $*"; }
warn()  { echo -e "${YELLOW}[debug-ios]${NC} $*"; }
fail()  { echo -e "${RED}[debug-ios]${NC} $*" >&2; exit 1; }

cleanup() {
    info "Stopping proxy and server..."
    [[ -n "${PROXY_PID:-}" ]] && kill "$PROXY_PID" 2>/dev/null || true
    [[ -n "${SERVER_PID:-}" ]] && kill "$SERVER_PID" 2>/dev/null || true
    exit 0
}
trap cleanup INT TERM

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
DEVICE_UDID=""
for arg in "$@"; do
    case "$arg" in
        --no-browser) OPEN_BROWSER=false ;;
        -h|--help)
            sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        --*) warn "Unknown option: $arg" ;;
        *) DEVICE_UDID="$arg" ;;
    esac
done

# ---------------------------------------------------------------------------
# 1. Check prerequisites
# ---------------------------------------------------------------------------
info "Checking prerequisites..."

command -v idevice_id >/dev/null 2>&1 || fail "libimobiledevice not found. Install usbmuxd + libimobiledevice."
command -v ios_webkit_debug_proxy >/dev/null 2>&1 || fail "ios-webkit-debug-proxy not found."
command -v python3 >/dev/null 2>&1 || fail "python3 not found."

[[ -d "$WEBINSPECTOR_DIR" ]] || fail "WebInspector not found at $WEBINSPECTOR_DIR. Run generate.sh in $TOOL_DIR first."

# ---------------------------------------------------------------------------
# 2. Detect device
# ---------------------------------------------------------------------------
if [[ -z "$DEVICE_UDID" ]]; then
    DEVICE_UDID="$(idevice_id -l 2>/dev/null | head -1)"
fi
[[ -z "$DEVICE_UDID" ]] && fail "No iOS device detected. Connect your iPhone via USB and unlock it."

DEVICE_NAME="$(ideviceinfo -u "$DEVICE_UDID" -k DeviceName 2>/dev/null || echo "$DEVICE_UDID")"
ok "Device: $DEVICE_NAME ($DEVICE_UDID)"

# ---------------------------------------------------------------------------
# 3. Stop anything already running
# ---------------------------------------------------------------------------
info "Stopping existing proxy/server instances..."
pkill -f ios_webkit_debug_proxy 2>/dev/null || true
pkill -f "http.server $HTTP_PORT" 2>/dev/null || true
sleep 1

# ---------------------------------------------------------------------------
# 4. Start the proxy
# ---------------------------------------------------------------------------
# Use an explicit config so BOTH ports are set up:
#   - 9221: device list (needed by the overview page)
#   - 9222: the device's pages
info "Starting ios-webkit-debug-proxy..."
ios_webkit_debug_proxy -c "null:9221,$DEVICE_UDID:$PROXY_PORT" --no-frontend &
PROXY_PID=$!

# ---------------------------------------------------------------------------
# 5. Start the WebInspector static server
# ---------------------------------------------------------------------------
info "Starting WebInspector server on http://localhost:$HTTP_PORT ..."
python3 -m http.server "$HTTP_PORT" --bind 127.0.0.1 --directory "$WEBINSPECTOR_DIR" &
SERVER_PID=$!

# ---------------------------------------------------------------------------
# 6. Wait for the proxy to attach and list pages
# ---------------------------------------------------------------------------
sleep 3

info "Checking available pages..."
PAGES="$(curl -s "http://localhost:$PROXY_PORT/json" 2>/dev/null || true)"

if [[ -z "$PAGES" || "$PAGES" == "[]" ]]; then
    warn "No inspectable pages found. Check that:"
    warn "  - Safari is open with the page you want to debug"
    warn "  - Settings > Safari > Advanced > Web Inspector is ON"
    warn "  - The device screen is unlocked"
else
    ok "Inspectable pages:"
    echo "$PAGES" | python3 -c '
import sys, json
for p in json.load(sys.stdin):
    print(f"  - {p[\"title\"]}: {p[\"url\"]}")
' 2>/dev/null || echo "$PAGES"
fi

# ---------------------------------------------------------------------------
# 7. Open the browser
# ---------------------------------------------------------------------------
OVERVIEW_URL="http://localhost:$HTTP_PORT/"
ok "Debug environment ready!"
echo ""
echo "  Overview page:  $OVERVIEW_URL"
echo "  Proxy targets:  http://localhost:$PROXY_PORT/json"
echo "  Press Ctrl+C to stop everything."
echo ""

if [[ "$OPEN_BROWSER" == true ]]; then
    if command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$OVERVIEW_URL" >/dev/null 2>&1 || true
    else
        warn "Open $OVERVIEW_URL in your browser."
    fi
fi

# ---------------------------------------------------------------------------
# 8. Keep running until Ctrl+C
# ---------------------------------------------------------------------------
wait