#!/bin/bash
#
# Bot Monitor with Email Alerts
# Monitors the bot and sends email alerts if it stops
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOG_FILE="$PROJECT_DIR/bot.log"
PID_FILE="$PROJECT_DIR/bot.pid"
ALERT_EMAIL=""

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
                grep -E "(Monitoring|opportunities|matched|arbitrage)" "$LOG_FILE" | tail -5
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

start_bot() {
    echo -e "${YELLOW}Starting bot...${NC}"

    cd "$PROJECT_DIR"

    # Check if already running
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}Bot is already running (PID: $PID)${NC}"
            return 0
        fi
    fi

    # Start the bot in background
    nohup ./target/release/prediction-market-arbitrage >> "$LOG_FILE" 2>&1 &
    PID=$!
    echo $PID > "$PID_FILE"

    sleep 2

    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Bot started successfully (PID: $PID)${NC}"
        echo -e "Logs: $LOG_FILE"
    else
        echo -e "${RED}✗ Bot failed to start. Check logs:${NC}"
        tail -20 "$LOG_FILE"
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

send_alert() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "[$timestamp] ALERT: $message" >> "$PROJECT_DIR/alerts.log"

    # Mac notification
    if [[ "$OSTYPE" == "darwin"* ]]; then
        osascript -e "display notification \"$message\" with title \"Arbitrage Bot Alert\" sound name \"Basso\""
    fi

    # Email alert if configured
    if [ -n "$ALERT_EMAIL" ]; then
        echo "Sending email alert to $ALERT_EMAIL..."

        # Try mail command first (works on most Unix systems)
        if command -v mail &> /dev/null; then
            echo "$message

Time: $timestamp
Server: $(hostname)

Check the bot status with:
cd $PROJECT_DIR && ./scripts/monitor_bot.sh status

View logs with:
tail -100 $LOG_FILE
" | mail -s "🚨 Arbitrage Bot Alert" "$ALERT_EMAIL"
        fi
    fi
}

watch_bot() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      BOT WATCHDOG MODE${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "This will monitor the bot and:"
    echo "  - Show a Mac notification if it stops"
    echo "  - Send email alert (if configured)"
    echo "  - Automatically restart the bot"
    echo ""
    echo "Press Ctrl+C to stop monitoring"
    echo ""

    # Make sure bot is running first
    if [ ! -f "$PID_FILE" ] || ! ps -p "$(cat "$PID_FILE" 2>/dev/null)" > /dev/null 2>&1; then
        echo -e "${YELLOW}Bot not running, starting it first...${NC}"
        start_bot
    fi

    local restart_count=0
    local last_check=$(date +%s)

    while true; do
        if [ -f "$PID_FILE" ]; then
            PID=$(cat "$PID_FILE")
            if ! ps -p "$PID" > /dev/null 2>&1; then
                restart_count=$((restart_count + 1))
                echo ""
                echo -e "${RED}⚠️  Bot stopped unexpectedly! (restart #$restart_count)${NC}"

                # Get last error from log
                last_error=$(tail -50 "$LOG_FILE" | grep -i "error\|panic\|fatal" | tail -3)

                send_alert "Bot stopped unexpectedly! Restart #$restart_count. Last error: $last_error"

                echo -e "${YELLOW}Restarting in 5 seconds...${NC}"
                sleep 5
                start_bot
            fi
        else
            start_bot
        fi

        # Every 60 seconds, show a status heartbeat
        current_time=$(date +%s)
        if [ $((current_time - last_check)) -ge 60 ]; then
            echo -e "[$(date '+%H:%M:%S')] ${GREEN}✓${NC} Bot running (PID: $(cat "$PID_FILE" 2>/dev/null))"
            last_check=$current_time
        fi

        sleep 10
    done
}

setup_email() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}      EMAIL ALERT SETUP${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    read -p "Enter your email address for alerts: " email

    if [ -n "$email" ]; then
        # Add to .env file
        if grep -q "BOT_ALERT_EMAIL" "$PROJECT_DIR/.env" 2>/dev/null; then
            sed -i '' "s/BOT_ALERT_EMAIL=.*/BOT_ALERT_EMAIL=$email/" "$PROJECT_DIR/.env"
        else
            echo "" >> "$PROJECT_DIR/.env"
            echo "# Email for bot alerts" >> "$PROJECT_DIR/.env"
            echo "BOT_ALERT_EMAIL=$email" >> "$PROJECT_DIR/.env"
        fi
        echo -e "${GREEN}✓ Email saved: $email${NC}"
        echo ""
        echo "Note: Email alerts require 'mail' command to be configured."
        echo "On Mac, you can also use the Mac notification alerts."
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
case "${1:-status}" in
    status)
        show_status
        ;;
    start)
        start_bot
        ;;
    stop)
        stop_bot
        ;;
    restart)
        stop_bot
        sleep 2
        start_bot
        ;;
    watch)
        watch_bot
        ;;
    logs)
        logs
        ;;
    setup-email)
        setup_email
        ;;
    *)
        echo "Usage: $0 {status|start|stop|restart|watch|logs|setup-email}"
        echo ""
        echo "Commands:"
        echo "  status      - Show if bot is running and recent activity"
        echo "  start       - Start the bot in background"
        echo "  stop        - Stop the bot"
        echo "  restart     - Restart the bot"
        echo "  watch       - Monitor bot and auto-restart if it stops (with alerts)"
        echo "  logs        - Show live log output"
        echo "  setup-email - Configure email alerts"
        exit 1
        ;;
esac
