#!/bin/bash
#
# View Arbitrage Bot Dashboard
# Opens a simple web server to view the dashboard
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if Python is available
if command -v python3 &> /dev/null; then
    PYTHON=python3
elif command -v python &> /dev/null; then
    PYTHON=python
else
    echo "Error: Python is required to serve the dashboard"
    echo "Please install Python 3"
    exit 1
fi

# Find available port
PORT=8080
while lsof -i:$PORT &>/dev/null 2>&1; do
    PORT=$((PORT + 1))
done

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   Arbitrage Bot Dashboard${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Starting web server on port ${YELLOW}$PORT${NC}..."
echo ""
echo -e "${GREEN}Open this URL in your browser:${NC}"
echo ""
echo -e "   ${YELLOW}http://localhost:$PORT/dashboard/${NC}"
echo ""
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Create dashboard_data directory if it doesn't exist
mkdir -p "$PROJECT_DIR/dashboard_data"

# Start simple HTTP server
cd "$PROJECT_DIR"
$PYTHON -m http.server $PORT
