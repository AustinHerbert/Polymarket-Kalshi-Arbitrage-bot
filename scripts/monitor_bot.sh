#!/bin/bash
#
# Bot Monitor with Auto-Restart
# Automatically keeps the bot running 24/7
# Only alerts on CRITICAL errors (not normal reconnects)
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOG_FILE="$PROJECT_DIR/bot.log"
PID_FILE="$PROJECT_DIR/bot.pid"
ALERT_LOG="$PROJECT_DIR/alerts.log"
ALERT_EMAIL=""

# Critical error patterns (these trigger alerts)
CRITICAL_ERRORS="panic|FATAL|out of memory|API key invalid|authentication failed|insufficient funds|account suspended"

# Normal errors (just restart, no alert)
NORMAL_ERRORS="Connection reset|WebSocket error|reconnecting|timed out|connection closed"

# Load email from .env if set
if [ -f "$PROJECT_DIR/.env" ]; then
    source "$PROJECT_DIR/.env"
    ALERT_EMAIL="${ALERT_EMAIL:-$BOT_ALERT_EMAIL}"
fi

show_status() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      BOT STATUS MONITOR${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""

    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "Status: ${GREEN}✓ RUNNING${NC}"
            echo -e "PID: $PID"

            # Show uptime
            if [[ "$OSTYPE" == "darwin"* ]]; then
                START_TIME=$(ps -p "$PID" -o lstart= 2>/dev/null)
                echo -e "Started: $START_TIME"
            else
                UPTIME=$(ps -p "$PID" -o etime= 2>/dev/null | xargs)
                echo -e "Uptime: $UPTIME"
            fi

            # Show recent log activity
            echo ""
            echo -e "${YELLOW}Last 10 log lines:${NC}"
            if [ -f "$LOG_FILE" ]; then
                tail -10 "$LOG_FILE"
            else
                echo "(No log file found)"
            fi

            # Show market stats from log
            echo ""
            echo -e "${YELLOW}Recent Activity:${NC}"
            if [ -f "$LOG_FILE" ]; then
                grep -E "(Monitoring|opportunities|matched|arbitrage|DRY RUN)" "$LOG_FILE" 2>/dev/null | tail -5
            fi
        else
            echo -e "Status: ${RED}✗ NOT RUNNING${NC} (stale PID file)"
            rm -f "$PID_FILE"
        fi
    else
        echo -e "Status: ${RED}✗ NOT RUNNING${NC}"
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
}

start_bot_once() {
    cd "$PROJECT_DIR"

    # Check if already running
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Bot is already running (PID: $PID)${NC}"
            return 0
        fi
    fi

    # Rotate log if it's too big (>10MB)
    if [ -f "$LOG_FILE" ] && [ $(stat -f%z "$LOG_FILE" 2>/dev/null || stat -c%s "$LOG_FILE" 2>/dev/null) -gt 10485760 ]; then
        mv "$LOG_FILE" "$LOG_FILE.old"
    fi

    # Start the bot in background
    nohup ./target/release/prediction-market-arbitrage >> "$LOG_FILE" 2>&1 &
    PID=$!
    echo $PID > "$PID_FILE"

    sleep 2

    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Bot started (PID: $PID)${NC}"
        return 0
    else
        echo -e "${RED}✗ Bot failed to start${NC}"
        return 1
    fi
}

stop_bot() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Stopping bot (PID: $PID)...${NC}"
            kill "$PID"
            sleep 2
            if ps -p "$PID" > /dev/null 2>&1; then
                kill -9 "$PID"
            fi
            rm -f "$PID_FILE"
            echo -e "${GREEN}✓ Bot stopped${NC}"
        else
            echo -e "${YELLOW}Bot was not running${NC}"
            rm -f "$PID_FILE"
        fi
    else
        echo -e "${YELLOW}Bot is not running${NC}"
    fi
}

send_critical_alert() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "[$timestamp] CRITICAL: $message" >> "$ALERT_LOG"

    # Mac notification with critical sound
    if [[ "$OSTYPE" == "darwin"* ]]; then
        osascript -e "display notification \"$message\" with title \"⚠️ CRITICAL: Arbitrage Bot\" sound name \"Basso\""
    fi

    # Email alert if configured
    if [ -n "$ALERT_EMAIL" ]; then
        if command -v mail &> /dev/null; then
            echo "CRITICAL BOT ALERT

$message

Time: $timestamp
Server: $(hostname)

Last 50 lines of log:
$(tail -50 "$LOG_FILE" 2>/dev/null)

This is a CRITICAL error that requires your attention.
The bot will attempt to restart, but please check the logs.
" | mail -s "⚠️ CRITICAL: Arbitrage Bot Alert" "$ALERT_EMAIL"
        fi
    fi
}

check_for_critical_error() {
    # Check last 100 lines for critical errors
    if [ -f "$LOG_FILE" ]; then
        local critical=$(tail -100 "$LOG_FILE" | grep -iE "$CRITICAL_ERRORS" | tail -1)
        if [ -n "$critical" ]; then
            echo "$critical"
            return 0
        fi
    fi
    return 1
}

# Main function: Start bot with auto-restart (this is what you'll use)
run() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}   ARBITRAGE BOT - AUTO-RESTART MODE${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "The bot will run continuously and auto-restart on errors."
    echo -e "You will ${YELLOW}ONLY be alerted for CRITICAL errors${NC} like:"
    echo -e "  - Authentication failures"
    echo -e "  - Insufficient funds"
    echo -e "  - Panics/crashes"
    echo ""
    echo -e "Normal disconnections (WebSocket drops) restart ${GREEN}silently${NC}."
    echo ""
    echo "Press Ctrl+C to stop"
    echo ""

    local restart_count=0
    local last_restart_time=0
    local rapid_restart_threshold=5  # Alert if 5+ restarts in 5 minutes

    # Trap Ctrl+C to stop gracefully
    trap 'echo ""; echo "Stopping bot..."; stop_bot; exit 0' INT TERM

    while true; do
        # Start bot if not running
        if [ ! -f "$PID_FILE" ] || ! ps -p "$(cat "$PID_FILE" 2>/dev/null)" > /dev/null 2>&1; then
            current_time=$(date +%s)

            # Check if this is a restart (not first start)
            if [ $restart_count -gt 0 ]; then
                # Check for critical errors
                critical_error=$(check_for_critical_error)
                if [ -n "$critical_error" ]; then
                    echo ""
                    echo -e "${RED}⚠️  CRITICAL ERROR DETECTED:${NC}"
                    echo -e "${RED}$critical_error${NC}"
                    send_critical_alert "Critical error: $critical_error"
                fi

                # Check for rapid restarts (possible crash loop)
                time_since_last=$((current_time - last_restart_time))
                if [ $time_since_last -lt 60 ]; then
                    # Restarted within 1 minute
                    rapid_count=$((rapid_count + 1))
                    if [ $rapid_count -ge $rapid_restart_threshold ]; then
                        echo -e "${RED}⚠️  TOO MANY RAPID RESTARTS ($rapid_count in 5 min)${NC}"
                        send_critical_alert "Bot crashed $rapid_count times in 5 minutes - possible critical issue!"
                        echo -e "${YELLOW}Waiting 60 seconds before next restart...${NC}"
                        sleep 60
                        rapid_count=0
                    fi
                else
                    rapid_count=0
                fi

                # Normal restart message (not an alert)
                echo -e "[$(date '+%H:%M:%S')] ${YELLOW}↻${NC} Restarting bot (restart #$restart_count)..."
                sleep 3
            fi

            start_bot_once
            restart_count=$((restart_count + 1))
            last_restart_time=$current_time
        fi

        # Heartbeat every 5 minutes
        current_time=$(date +%s)
        if [ -z "$last_heartbeat" ]; then
            last_heartbeat=$current_time
        fi
        if [ $((current_time - last_heartbeat)) -ge 300 ]; then
            # Show brief status
            trades=$(grep -c "DRY RUN\|EXEC" "$LOG_FILE" 2>/dev/null || echo "0")
            echo -e "[$(date '+%H:%M:%S')] ${GREEN}✓${NC} Running | Trades logged: ~$trades"
            last_heartbeat=$current_time
        fi

        sleep 10
    done
}

# Simple start (just starts once, no auto-restart)
start_simple() {
    echo -e "${YELLOW}Starting bot (one-time, no auto-restart)...${NC}"
    echo -e "Use ${GREEN}./scripts/monitor_bot.sh run${NC} for auto-restart mode"
    echo ""
    start_bot_once
}

setup_email() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      EMAIL ALERT SETUP${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    read -p "Enter your email address for critical alerts: " email

    if [ -n "$email" ]; then
        # Add to .env file
        if grep -q "BOT_ALERT_EMAIL" "$PROJECT_DIR/.env" 2>/dev/null; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/BOT_ALERT_EMAIL=.*/BOT_ALERT_EMAIL=$email/" "$PROJECT_DIR/.env"
            else
                sed -i "s/BOT_ALERT_EMAIL=.*/BOT_ALERT_EMAIL=$email/" "$PROJECT_DIR/.env"
            fi
        else
            echo "" >> "$PROJECT_DIR/.env"
            echo "# Email for critical bot alerts" >> "$PROJECT_DIR/.env"
            echo "BOT_ALERT_EMAIL=$email" >> "$PROJECT_DIR/.env"
        fi
        echo -e "${GREEN}✓ Email saved: $email${NC}"
        echo ""
        echo "You'll only receive emails for CRITICAL errors, not normal restarts."
    fi
}

logs() {
    if [ -f "$LOG_FILE" ]; then
        echo -e "${GREEN}Showing live logs (Ctrl+C to stop):${NC}"
        tail -f "$LOG_FILE"
    else
        echo -e "${RED}No log file found at $LOG_FILE${NC}"
    fi
}

# Main command handler
case "${1:-run}" in
    run)
        run
        ;;
    status)
        show_status
        ;;
    start)
        start_simple
        ;;
    stop)
        stop_bot
        ;;
    restart)
        stop_bot
        sleep 2
        run
        ;;
    logs)
        logs
        ;;
    setup-email)
        setup_email
        ;;
    *)
        echo "Usage: $0 {run|status|start|stop|restart|logs|setup-email}"
        echo ""
        echo "Commands:"
        echo "  run         - Start bot with AUTO-RESTART (recommended)"
        echo "  status      - Show if bot is running and recent activity"
        echo "  start       - Start the bot once (no auto-restart)"
        echo "  stop        - Stop the bot"
        echo "  restart     - Restart with auto-restart mode"
        echo "  logs        - Show live log output"
        echo "  setup-email - Configure email for critical alerts only"
        echo ""
        echo "Example: ./scripts/monitor_bot.sh run"
        exit 1
        ;;
esac
