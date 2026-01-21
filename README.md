# Aerodrome TWAP Calculator

A Rust command-line tool to calculate Time-Weighted Average Price (TWAP) for Aerodrome pools on Base network.

## Features

- ✅ Calculate TWAP over customizable time periods (default: 7 days)
- ✅ Configurable sampling intervals for accuracy vs speed
- ✅ Automatic token information retrieval (symbols, decimals)
- ✅ Statistical analysis (min, max, current price, deviation)
- ✅ Support for any Aerodrome pool on Base

## Installation

### Prerequisites

- Rust 1.70 or later ([install here](https://rustup.rs/))
- Access to a Base RPC endpoint (defaults to public endpoint)

### Build

```bash
cargo build --release
```

The compiled binary will be available at `target/release/twap`

## Usage

### Basic Usage

Calculate 1-week TWAP for a pool:

```bash
cargo run --release -- --pool 0xYourPoolAddress
```

### Advanced Options

```bash
cargo run --release -- \
  --pool 0xYourPoolAddress \
  --rpc https://mainnet.base.org \
  --days 7 \
  --samples 168 \
  --end-date 2024-01-15
```

### Parameters

- `--pool, -p`: Aerodrome pool address (required)
- `--rpc, -r`: Base RPC URL (default: `https://mainnet.base.org`)
- `--days, -d`: Number of days for TWAP calculation (default: 7)
- `--samples, -s`: Number of sample points (default: 168, i.e., hourly samples for a week)
- `--end-date, -e`: End date for TWAP range in YYYY-MM-DD format (midnight US Central Time). If not specified, uses current time

### Examples

#### 1. Calculate 1-week TWAP with hourly samples:

```bash
cargo run --release -- --pool 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d
```

#### 2. Calculate 30-day TWAP with daily samples:

```bash
cargo run --release -- --pool 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d --days 30 --samples 30
```

#### 3. High-precision 7-day TWAP with 15-minute intervals:

```bash
cargo run --release -- --pool 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d --days 7 --samples 672
```

#### 4. Calculate historical TWAP ending at a specific date:

```bash
cargo run --release -- \
  --pool 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d \
  --days 7 \
  --end-date 2024-01-15
```

This calculates the 7-day TWAP ending at midnight US Central Time on January 15, 2024.

#### 5. Using custom RPC endpoint:

```bash
cargo run --release -- \
  --pool 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d \
  --rpc https://base-mainnet.g.alchemy.com/v2/your-api-key
```

## Output

The tool provides comprehensive statistics:

```
🚀 Aerodrome TWAP Calculator
📍 Pool: 0x6cDcb1C4A4D1C3C6d054b27AC5B77e89eAFb971d
⏰ Period: 7 days
📊 Samples: 168

📌 Token0: WETH (0x4200000000000000000000000000000000000006)
📌 Token1: USDC (0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913)

⏱️  Collecting price data...
✓ Collected 168/168 samples

📈 RESULTS
═══════════════════════════════════════
🎯 7-Day TWAP: 2345.67890123 USDC per WETH
💵 Current Price: 2350.12345678 USDC per WETH
📊 Min Price: 2320.00000000
📊 Max Price: 2380.00000000
📉 Price Range: 2.59%
📍 Deviation from TWAP: 0.19%
═══════════════════════════════════════
```

## How It Works

1. **Connection**: Connects to Base network via RPC
2. **Pool Analysis**: Retrieves token addresses and decimals from the pool contract
3. **Time Range Setup**:
   - If `--end-date` is provided, uses binary search to find the block at that timestamp (midnight US Central Time)
   - Otherwise, uses the current block as the end point
4. **Historical Sampling**: Queries pool reserves at regular block intervals over the specified period
5. **Price Calculation**: Calculates spot price at each sample point (reserve1/reserve0)
6. **TWAP Computation**: Applies time-weighting to calculate the true time-weighted average price

## Finding Pool Addresses

You can find Aerodrome pool addresses:

1. [Aerodrome Finance](https://aerodrome.finance/) - Official interface
2. [Base Explorer](https://basescan.org/) - Search for Aerodrome pools
3. Aerodrome Factory contract events

## Performance Considerations

- **Samples**: More samples = higher accuracy but slower execution
  - 168 samples (hourly) for 7 days: ~30-60 seconds
  - 672 samples (15-min) for 7 days: ~2-4 minutes
  - 30 samples (daily) for 30 days: ~10-20 seconds

- **RPC Rate Limits**: Public RPC endpoints may rate limit. Consider using:
  - Alchemy, Infura, or QuickNode for production use
  - Reducing sample count if you hit rate limits

## Technical Details

- Uses `ethers-rs` for Ethereum/Base interactions
- Block time on Base: ~2 seconds average
- TWAP formula: Σ(price_i × time_i) / Σ(time_i)
- Prices are normalized by token decimals for accuracy

## Troubleshooting

### "Failed to connect to RPC"
- Check your internet connection
- Verify the RPC URL is correct
- Try a different RPC provider

### "Failed to get reserves"
- Ensure the pool address is correct
- Verify it's an Aerodrome pool contract
- Check if the pool exists and has liquidity

### Rate limiting errors
- Reduce the number of samples with `--samples`
- Use a private RPC endpoint with higher limits
- Add delays between requests (requires code modification)

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

## Disclaimer

This tool is for informational purposes only. Always verify critical data through multiple sources before making financial decisions.
