//! State service main entry point

use anyhow::Result;
use clap::Parser;
use config::{Config, File, Environment};
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

mod settings;
mod snapshotter;
mod market_service;
mod chat_service;
mod leaderboard_service;

#[derive(Parser, Debug)]
#[command(name = "state-service", version, about = "War2 State Service")]
struct Args {
    #[arg(short, long, default_value = "config/state-service.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    
    let args = Args::parse();
    let settings = Config::builder()
        .add_source(File::with_name(&args.config).required(false))
        .add_source(Environment::with_prefix("STATE"))
        .build()?;
    
    let cfg: settings::StateServiceConfig = settings.try_deserialize()?;
    
    info!("Starting War2 State Service");
    
    // TODO: Implement state service
    // - Consume Redis Streams from all shards
    // - Write PostgreSQL snapshots
    // - Run global services (market, chat, leaderboards)
    // - Season transition orchestration
    
    Ok(())
}