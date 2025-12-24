#!/bin/bash

set -e

echo "=========================================="
echo "Polymarket-Kalshi Arbitrage Bot Fork Setup"
echo "=========================================="
echo ""

# Default fork URL
DEFAULT_FORK_URL="https://github.com/terauss/Polymarket-Kalshi-Arbitrage-bot.git"

# Prompt for fork URL
echo "Enter your forked repository URL (or press Enter to use original):"
read -r FORK_URL
FORK_URL=${FORK_URL:-$DEFAULT_FORK_URL}

# Prompt for directory name
echo ""
echo "Enter directory name for the fork (default: original-bot):"
read -r DIR_NAME
DIR_NAME=${DIR_NAME:-original-bot}

# Clone the repository
echo ""
echo "Cloning repository..."
if [ -d "$DIR_NAME" ]; then
    echo "Directory '$DIR_NAME' already exists. Remove it? (y/n)"
    read -r REMOVE
    if [ "$REMOVE" = "y" ]; then
        rm -rf "$DIR_NAME"
    else
        echo "Aborting."
        exit 1
    fi
fi

git clone "$FORK_URL" "$DIR_NAME"
cd "$DIR_NAME"

# Create .env file
echo ""
echo "=========================================="
echo "API Configuration"
echo "=========================================="
echo ""

# Check if parent .env exists
if [ -f "../.env" ]; then
    echo "Found existing .env in parent directory. Copy credentials? (y/n)"
    read -r COPY_ENV
    if [ "$COPY_ENV" = "y" ]; then
        cp ../.env .env
        # Ensure DRY_RUN is set to 1
        if grep -q "^DRY_RUN=" .env; then
            sed -i 's/^DRY_RUN=.*/DRY_RUN=1/' .env
        else
            echo "DRY_RUN=1" >> .env
        fi
        echo "✓ Credentials copied and DRY_RUN enabled"
    else
        CREATE_NEW=true
    fi
else
    CREATE_NEW=true
fi

if [ "$CREATE_NEW" = true ]; then
    echo "Creating new .env file..."
    echo ""

    echo "Enter Kalshi API Key ID:"
    read -r KALSHI_API_KEY_ID

    echo "Enter path to Kalshi private key (PEM file):"
    read -r KALSHI_PRIVATE_KEY_PATH

    echo "Enter Polymarket private key (with 0x prefix):"
    read -r POLY_PRIVATE_KEY

    echo "Enter Polymarket wallet address (with 0x prefix):"
    read -r POLY_FUNDER

    cat > .env << EOF
# === KALSHI CREDENTIALS ===
KALSHI_API_KEY_ID=$KALSHI_API_KEY_ID
KALSHI_PRIVATE_KEY_PATH=$KALSHI_PRIVATE_KEY_PATH

# === POLYMARKET CREDENTIALS ===
POLY_PRIVATE_KEY=$POLY_PRIVATE_KEY
POLY_FUNDER=$POLY_FUNDER

# === SYSTEM CONFIGURATION ===
DRY_RUN=1
RUST_LOG=info
PRICE_LOGGING=0
EOF

    echo "✓ .env file created with DRY_RUN enabled"
fi

# Build the project
echo ""
echo "=========================================="
echo "Building the bot..."
echo "=========================================="
echo ""

if ! command -v cargo &> /dev/null; then
    echo "Rust/Cargo not found. Install Rust? (y/n)"
    read -r INSTALL_RUST
    if [ "$INSTALL_RUST" = "y" ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo "Skipping build. Install Rust manually with:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 0
    fi
fi

cargo build --release

echo ""
echo "=========================================="
echo "✓ Setup Complete!"
echo "=========================================="
echo ""
echo "The forked bot is ready in: $DIR_NAME"
echo ""
echo "To run the bot in DRY RUN mode (paper trading):"
echo "  cd $DIR_NAME"
echo "  cargo run --release"
echo ""
echo "Or with dotenvx (if installed):"
echo "  cd $DIR_NAME"
echo "  dotenvx run -- cargo run --release"
echo ""
echo "To run with verbose logging:"
echo "  RUST_LOG=debug cargo run --release"
echo ""
echo "To test with synthetic arbitrage:"
echo "  TEST_ARB=1 cargo run --release"
echo ""
echo "=========================================="
