use anyhow::{Context, Result};
use clap::Parser;
use ethers::prelude::*;
use std::sync::Arc;

// Aerodrome Pool ABI (simplified - includes the methods we need)
abigen!(
    AerodromePool,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function token0() external view returns (address)
        function token1() external view returns (address)
        function decimals() external view returns (uint8)
    ]"#,
);

abigen!(
    ERC20,
    r#"[
        function decimals() external view returns (uint8)
        function symbol() external view returns (string)
    ]"#,
);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Aerodrome pool address
    #[arg(short, long)]
    pool: String,

    /// Base RPC URL (defaults to public Base RPC)
    #[arg(short, long, default_value = "https://mainnet.base.org")]
    rpc: String,

    /// Number of days to calculate TWAP (defaults to 7)
    #[arg(short, long, default_value = "7")]
    days: u64,

    /// Number of sample points (defaults to 168 = hourly for a week)
    #[arg(short, long, default_value = "168")]
    samples: u64,
}

#[derive(Debug)]
struct PricePoint {
    timestamp: u64,
    price: f64,
    block_number: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🚀 Aerodrome TWAP Calculator");
    println!("📍 Pool: {}", args.pool);
    println!("⏰ Period: {} days", args.days);
    println!("📊 Samples: {}", args.samples);
    println!();

    // Connect to Base network
    let provider = Provider::<Http>::try_from(&args.rpc)
        .context("Failed to connect to RPC")?;
    let provider = Arc::new(provider);

    // Parse pool address
    let pool_address: Address = args.pool.parse().context("Invalid pool address")?;
    let pool = AerodromePool::new(pool_address, provider.clone());

    // Get token information
    let token0_addr = pool.token_0().call().await.context("Failed to get token0")?;
    let token1_addr = pool.token_1().call().await.context("Failed to get token1")?;

    let token0 = ERC20::new(token0_addr, provider.clone());
    let token1 = ERC20::new(token1_addr, provider.clone());

    let token0_decimals = token0.decimals().call().await.context("Failed to get token0 decimals")?;
    let token1_decimals = token1.decimals().call().await.context("Failed to get token1 decimals")?;
    let token0_symbol = token0.symbol().call().await.unwrap_or_else(|_| "UNKNOWN".to_string());
    let token1_symbol = token1.symbol().call().await.unwrap_or_else(|_| "UNKNOWN".to_string());

    println!("📌 Token0: {} ({})", token0_symbol, token0_addr);
    println!("📌 Token1: {} ({})", token1_symbol, token1_addr);
    println!();

    // Get current block
    let current_block = provider
        .get_block_number()
        .await
        .context("Failed to get current block")?;

    // Calculate time period
    let seconds_per_day = 86400u64;
    let total_seconds = args.days * seconds_per_day;
    let interval_seconds = total_seconds / args.samples;

    // Base has ~2 second block time on average
    let blocks_per_second = 0.5f64;
    let blocks_per_interval = (interval_seconds as f64 * blocks_per_second) as u64;

    println!("⏱️  Collecting price data...");

    let mut price_points = Vec::new();
    let mut total_weighted_price = 0.0f64;
    let mut total_time = 0u64;

    for i in 0..args.samples {
        let blocks_back = (args.samples - i) * blocks_per_interval;
        let target_block = if blocks_back > current_block.as_u64() {
            U64::from(1) // Genesis block if we go too far back
        } else {
            current_block - blocks_back
        };

        // Get block timestamp
        let block = provider
            .get_block(target_block)
            .await
            .context("Failed to get block")?
            .context("Block not found")?;

        let timestamp = block.timestamp.as_u64();

        // Get reserves at this block
        let (reserve0, reserve1, _) = pool
            .get_reserves()
            .block(BlockId::Number(BlockNumber::Number(target_block)))
            .call()
            .await
            .context(format!("Failed to get reserves at block {}", target_block))?;

        // Calculate price (token1 per token0)
        if reserve0 > 0 {
            let reserve0_f64 = reserve0 as f64 / 10f64.powi(token0_decimals as i32);
            let reserve1_f64 = reserve1 as f64 / 10f64.powi(token1_decimals as i32);
            let price = reserve1_f64 / reserve0_f64;

            price_points.push(PricePoint {
                timestamp,
                price,
                block_number: target_block.as_u64(),
            });

            // Calculate time weight for TWAP
            if i > 0 {
                let time_diff = timestamp - price_points[i - 1].timestamp;
                let weighted_price = price_points[i - 1].price * time_diff as f64;
                total_weighted_price += weighted_price;
                total_time += time_diff;
            }

            if (i + 1) % 10 == 0 || i == args.samples - 1 {
                print!("\r✓ Collected {}/{} samples", i + 1, args.samples);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
    }

    println!();
    println!();

    if price_points.is_empty() {
        anyhow::bail!("No price data collected");
    }

    // Calculate final TWAP
    let twap = if total_time > 0 {
        total_weighted_price / total_time as f64
    } else {
        price_points.last().unwrap().price
    };

    // Calculate current price (spot price)
    let current_price = price_points.last().unwrap().price;

    // Calculate min and max prices
    let min_price = price_points.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
    let max_price = price_points.iter().map(|p| p.price).fold(f64::NEG_INFINITY, f64::max);

    // Results
    println!("📈 RESULTS");
    println!("═══════════════════════════════════════");
    println!("🎯 {}-Day TWAP: {:.8} {} per {}",
        args.days, twap, token1_symbol, token0_symbol);
    println!("💵 Current Price: {:.8} {} per {}",
        current_price, token1_symbol, token0_symbol);
    println!("📊 Min Price: {:.8}", min_price);
    println!("📊 Max Price: {:.8}", max_price);
    println!("📉 Price Range: {:.2}%",
        ((max_price - min_price) / min_price * 100.0));
    println!("📍 Deviation from TWAP: {:.2}%",
        ((current_price - twap) / twap * 100.0));
    println!("═══════════════════════════════════════");

    Ok(())
}
