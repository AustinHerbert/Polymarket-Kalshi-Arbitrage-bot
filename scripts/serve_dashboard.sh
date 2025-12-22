#!/bin/bash
#
# Serve the metrics dashboard locally
#
# Usage:
#   ./scripts/serve_dashboard.sh [port]
#
# Opens the dashboard at http://localhost:8080 (or specified port)
#

PORT=${1:-8080}
DASHBOARD_DIR="$(dirname "$0")/../dashboard"

echo "════════════════════════════════════════════════════════════════"
echo "  Arbitrage Bot Dashboard"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "  Starting server at: http://localhost:${PORT}"
echo ""
echo "  To load metrics, either:"
echo "    1. Use the file upload buttons in the dashboard"
echo "    2. Copy metrics files to the dashboard directory:"
echo "       cp metrics_*.json ${DASHBOARD_DIR}/"
echo ""
echo "  Press Ctrl+C to stop the server"
echo "════════════════════════════════════════════════════════════════"
echo ""

cd "$DASHBOARD_DIR"

# Try Python 3 first, then Python 2
if command -v python3 &> /dev/null; then
    python3 -m http.server "$PORT"
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer "$PORT"
else
    echo "Error: Python not found. Please install Python to serve the dashboard."
    exit 1
fi
