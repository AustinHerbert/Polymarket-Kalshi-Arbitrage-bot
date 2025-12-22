# Easy Setup Guide for Arbitrage Bot

This guide will help you get the bot running step-by-step. No coding experience needed!

---

## Prerequisites (What You Need First)

### 1. Install Rust (The programming language the bot uses)

**On Mac/Linux:** Open Terminal and paste this command:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Then restart your terminal.

**On Windows:** Download and run the installer from: https://rustup.rs

### 2. Get Your API Credentials

#### Kalshi API:
1. Go to https://kalshi.com/settings/api
2. Create a new API key
3. Download the private key file (save it somewhere safe like your Documents folder)
4. Copy the API Key ID (you'll need this later)

#### Polymarket:
1. You need your wallet's private key and address
2. Your private key starts with `0x` followed by 64 characters
3. Your wallet address also starts with `0x`

---

## Step-by-Step Setup

### Step 1: Open Terminal

**Mac:** Press `Cmd + Space`, type "Terminal", press Enter

**Windows:** Press `Win + R`, type "cmd", press Enter

**Linux:** Press `Ctrl + Alt + T`

### Step 2: Navigate to the Bot Folder

Type this command and press Enter:
```bash
cd /home/user/Polymarket-Kalshi-Arbitrage-bot
```

(Replace with your actual path if different)

### Step 3: Run the Easy Setup

Type this command and press Enter:
```bash
./scripts/easy_setup.sh
```

Follow the prompts:
1. Enter your Kalshi API Key ID
2. Enter the full path to your Kalshi private key file
3. Enter your Polymarket private key
4. Enter your Polymarket wallet address
5. Choose whether to enable Priority Mode (type `y` for yes)

### Step 4: Start the Bot (Test Mode)

Type this command and press Enter:
```bash
./scripts/run_bot.sh
```

The bot will start in **DRY RUN** mode, which means:
- It connects to real markets
- It finds real arbitrage opportunities
- It does **NOT** make any real trades
- Perfect for testing!

### Step 5: Watch the Output

You'll see messages like:
- `[INFO] Connecting to Kalshi...`
- `[INFO] Priority Mode ENABLED`
- `[INFO] Found arbitrage opportunity: X%`

Press `Ctrl + C` to stop the bot at any time.

---

## Going Live (Real Trading)

**Only do this after you've tested and you're confident!**

### Step 1: Edit the .env File

Open the `.env` file in any text editor and change:
```
DRY_RUN=1
```
to:
```
DRY_RUN=0
```

### Step 2: Run the Bot
```bash
./scripts/run_bot.sh
```

It will ask you to confirm since real money is involved.

---

## Adjusting Settings

All settings are in the `.env` file. Open it with any text editor.

### Priority Mode Settings:

| Setting | What It Does | Default |
|---------|-------------|---------|
| `PRIORITY_MODE=1` | Enable priority sorting | 1 (on) |
| `MIN_LIQUIDITY_CENTS=25000` | Minimum trade size ($250) | 25000 |
| `MAX_LIQUIDITY_CENTS=250000` | Maximum trade size ($2,500) | 250000 |
| `MIN_ARB_PERCENT=1.0` | Only trade if profit >= 1% | 1.0 |
| `QUEUE_SORT_INTERVAL_SECS=5` | Re-sort queue every 5 seconds | 5 |
| `LIVE_PRIORITY_BOOST=10.0` | How much to prioritize live games | 10.0 |

### Examples:

**More conservative (smaller trades, higher profit required):**
```
MIN_LIQUIDITY_CENTS=10000
MAX_LIQUIDITY_CENTS=100000
MIN_ARB_PERCENT=2.0
```

**More aggressive (larger trades, lower threshold):**
```
MIN_LIQUIDITY_CENTS=50000
MAX_LIQUIDITY_CENTS=500000
MIN_ARB_PERCENT=0.5
```

---

## Troubleshooting

### "Command not found" error
- Make sure Rust is installed: `rustc --version`
- If not, install it (see Prerequisites above)

### "KALSHI_API_KEY_ID not set" error
- Run the setup script: `./scripts/easy_setup.sh`
- Or make sure your `.env` file exists and has your credentials

### "Permission denied" error
- Run: `chmod +x scripts/*.sh`

### Bot connects but finds no opportunities
- This is normal! Arbitrage opportunities are rare
- The bot will keep scanning until it finds one

### Bot crashes immediately
- Check your API credentials are correct
- Make sure your Kalshi private key file path is correct

---

## Need Help?

1. Check the error message carefully
2. Make sure all your credentials are correct
3. Try running in DRY_RUN=1 mode first

---

## Quick Reference

| Command | What It Does |
|---------|--------------|
| `./scripts/easy_setup.sh` | Set up your credentials |
| `./scripts/run_bot.sh` | Start the bot |
| `Ctrl + C` | Stop the bot |

