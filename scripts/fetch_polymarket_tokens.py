#!/usr/bin/env python3
"""
Fetch active Polymarket markets and their token IDs for spread farming.

This script queries the Polymarket Gamma API to find active, high-volume markets
suitable for market making, and outputs their token IDs.

Usage:
    python3 fetch_polymarket_tokens.py
    python3 fetch_polymarket_tokens.py --search "bitcoin"
    python3 fetch_polymarket_tokens.py --min-volume 1000000
"""

import argparse
import json
import sys
from typing import Optional
from urllib.request import urlopen, Request
from urllib.error import HTTPError, URLError

GAMMA_API_BASE = "https://gamma-api.polymarket.com"


def fetch_json(url: str) -> dict:
    """Fetch JSON from URL with proper headers."""
    headers = {
        "Accept": "application/json",
        "User-Agent": "PolymarketBot/1.0"
    }
    req = Request(url, headers=headers)
    try:
        with urlopen(req, timeout=30) as response:
            return json.loads(response.read().decode())
    except HTTPError as e:
        print(f"HTTP Error {e.code}: {e.reason}", file=sys.stderr)
        raise
    except URLError as e:
        print(f"URL Error: {e.reason}", file=sys.stderr)
        raise


def fetch_all_events(closed: bool = False, limit: int = 100) -> list:
    """Fetch all active events with pagination."""
    all_events = []
    offset = 0

    while True:
        url = f"{GAMMA_API_BASE}/events?closed={str(closed).lower()}&limit={limit}&offset={offset}&order=volume24hr&ascending=false"
        print(f"Fetching events (offset={offset})...", file=sys.stderr)

        events = fetch_json(url)
        if not events:
            break

        all_events.extend(events)

        if len(events) < limit:
            break

        offset += limit

        # Safety limit
        if offset > 1000:
            break

    return all_events


def fetch_markets_by_slug(slug: str) -> list:
    """Fetch markets by slug."""
    url = f"{GAMMA_API_BASE}/markets?slug={slug}"
    return fetch_json(url)


def fetch_event_by_slug(slug: str) -> list:
    """Fetch event by slug."""
    url = f"{GAMMA_API_BASE}/events?slug={slug}"
    return fetch_json(url)


def parse_token_ids(clob_token_ids: str) -> list:
    """Parse clobTokenIds JSON string to list."""
    if not clob_token_ids:
        return []
    try:
        return json.loads(clob_token_ids)
    except json.JSONDecodeError:
        return []


def format_market(market: dict, event: Optional[dict] = None) -> dict:
    """Format market info for output."""
    token_ids = parse_token_ids(market.get("clobTokenIds", "[]"))
    outcomes = market.get("outcomes", "[]")
    if isinstance(outcomes, str):
        outcomes = json.loads(outcomes)

    outcome_prices = market.get("outcomePrices", "[]")
    if isinstance(outcome_prices, str):
        try:
            outcome_prices = json.loads(outcome_prices)
        except:
            outcome_prices = []

    return {
        "slug": market.get("slug", event.get("slug", "") if event else ""),
        "question": market.get("question", ""),
        "description": market.get("description", "")[:200] if market.get("description") else "",
        "condition_id": market.get("conditionId", ""),
        "volume_24h": float(market.get("volume24hr", 0) or 0),
        "volume_total": float(market.get("volume", 0) or 0),
        "liquidity": float(market.get("liquidity", 0) or 0),
        "active": market.get("active", False),
        "closed": market.get("closed", False),
        "enable_order_book": market.get("enableOrderBook", False),
        "outcomes": outcomes,
        "outcome_prices": outcome_prices,
        "token_ids": token_ids,
        "yes_token": token_ids[0] if len(token_ids) > 0 else None,
        "no_token": token_ids[1] if len(token_ids) > 1 else None,
    }


def format_event(event: dict) -> dict:
    """Format event with all its markets."""
    markets = event.get("markets", [])
    formatted_markets = []

    for m in markets:
        formatted_markets.append(format_market(m, event))

    return {
        "slug": event.get("slug", ""),
        "title": event.get("title", ""),
        "description": event.get("description", "")[:200] if event.get("description") else "",
        "volume_24h": float(event.get("volume24hr", 0) or 0),
        "volume_total": float(event.get("volume", 0) or 0),
        "liquidity": float(event.get("liquidity", 0) or 0),
        "active": event.get("active", False),
        "closed": event.get("closed", False),
        "markets": formatted_markets,
    }


def search_events(events: list, query: str) -> list:
    """Filter events by search query."""
    query = query.lower()
    results = []
    for event in events:
        title = event.get("title", "").lower()
        description = event.get("description", "").lower()
        slug = event.get("slug", "").lower()

        if query in title or query in description or query in slug:
            results.append(event)

    return results


def print_market_summary(formatted: dict, indent: str = ""):
    """Print a formatted market summary."""
    print(f"{indent}Market: {formatted['question'][:80]}")
    print(f"{indent}  Slug: {formatted['slug']}")
    print(f"{indent}  Active: {formatted['active']}, OrderBook: {formatted['enable_order_book']}")
    print(f"{indent}  Volume 24h: ${formatted['volume_24h']:,.0f}")
    print(f"{indent}  Total Volume: ${formatted['volume_total']:,.0f}")
    print(f"{indent}  Liquidity: ${formatted['liquidity']:,.0f}")
    print(f"{indent}  Condition ID: {formatted['condition_id'][:20]}..." if formatted['condition_id'] else "")

    if formatted['outcomes'] and formatted['outcome_prices']:
        for i, (outcome, price) in enumerate(zip(formatted['outcomes'], formatted['outcome_prices'])):
            token_id = formatted['token_ids'][i] if i < len(formatted['token_ids']) else "N/A"
            print(f"{indent}  [{outcome}] Price: {float(price):.2f}, Token: {token_id[:40]}...")
    elif formatted['token_ids']:
        print(f"{indent}  YES Token: {formatted['yes_token']}")
        print(f"{indent}  NO Token: {formatted['no_token']}")
    print()


def print_event_summary(event: dict):
    """Print a formatted event summary."""
    formatted = format_event(event)
    print("=" * 80)
    print(f"EVENT: {formatted['title']}")
    print(f"Slug: {formatted['slug']}")
    print(f"Volume 24h: ${formatted['volume_24h']:,.0f} | Total: ${formatted['volume_total']:,.0f} | Liquidity: ${formatted['liquidity']:,.0f}")
    print("-" * 80)

    for m in formatted['markets']:
        print_market_summary(m, indent="  ")


def output_config_format(markets: list):
    """Output markets in a format suitable for config files."""
    print("\n" + "=" * 80)
    print("TOKEN IDS FOR CONFIG (Rust format)")
    print("=" * 80)

    for m in markets:
        if m.get('yes_token') and m.get('no_token'):
            print(f"""
// Market: {m['question'][:60]}
// Slug: {m['slug']}
// Volume 24h: ${m['volume_24h']:,.0f}
MarketConfig {{
    name: "{m['slug']}",
    yes_token: "{m['yes_token']}",
    no_token: "{m['no_token']}",
    min_spread: 0.02,
    order_size: 10.0,
}},""")


def main():
    parser = argparse.ArgumentParser(description="Fetch Polymarket markets and token IDs")
    parser.add_argument("--search", "-s", type=str, help="Search for specific markets")
    parser.add_argument("--slug", type=str, help="Look up specific market by slug")
    parser.add_argument("--min-volume", type=float, default=10000, help="Minimum 24h volume filter")
    parser.add_argument("--limit", type=int, default=20, help="Max number of events to show")
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--config", action="store_true", help="Output in config format")

    args = parser.parse_args()

    try:
        if args.slug:
            # Look up specific slug
            print(f"Looking up slug: {args.slug}", file=sys.stderr)

            # Try as event first
            events = fetch_event_by_slug(args.slug)
            if events:
                for event in events:
                    if args.json:
                        print(json.dumps(format_event(event), indent=2))
                    else:
                        print_event_summary(event)
                return

            # Try as market
            markets = fetch_markets_by_slug(args.slug)
            if markets:
                for market in markets:
                    formatted = format_market(market)
                    if args.json:
                        print(json.dumps(formatted, indent=2))
                    else:
                        print_market_summary(formatted)
                return

            print(f"No market or event found for slug: {args.slug}")
            return

        # Fetch all active events
        events = fetch_all_events(closed=False)
        print(f"Fetched {len(events)} events", file=sys.stderr)

        # Filter by search query
        if args.search:
            events = search_events(events, args.search)
            print(f"Found {len(events)} matching events", file=sys.stderr)

        # Filter by volume
        events = [e for e in events if float(e.get("volume24hr", 0) or 0) >= args.min_volume]
        print(f"{len(events)} events with volume >= ${args.min_volume:,.0f}", file=sys.stderr)

        # Sort by 24h volume
        events.sort(key=lambda e: float(e.get("volume24hr", 0) or 0), reverse=True)

        # Limit results
        events = events[:args.limit]

        if args.json:
            formatted_events = [format_event(e) for e in events]
            print(json.dumps(formatted_events, indent=2))
        else:
            for event in events:
                print_event_summary(event)

            # Also output config format if requested
            if args.config:
                all_markets = []
                for event in events:
                    for m in event.get("markets", []):
                        fm = format_market(m, event)
                        if fm.get('yes_token') and fm.get('no_token'):
                            all_markets.append(fm)
                output_config_format(all_markets)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
