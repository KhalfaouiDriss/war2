//! Shard service main entry point

use anyhow::Result;
use clap::Parser;
use config::{Config, File, Environment};
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

mod settings;
mod simulation;
mod tcp_server;
mod redis_client;

#[derive(Parser, Debug)]
#[command(name = "shard", version, about = "War2 World Shard")]
struct Args {
    #[arg(short, long, default_value = "config/shard.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    
    let args = Args::parse();
    let settings = Config::builder()
        .add_source(File::with_name(&args.config).required(false))
        .add_source(Environment::with_prefix("SHARD"))
        .build()?;
    
    let cfg: settings::ShardConfig = settings.try_deserialize()?;
    
    info!("Starting War2 Shard {} on {}:{}", cfg.shard.id, cfg.shard.host, cfg.shard.port);
    
    // TODO: Implement shard
    // - TCP server for gateway communication
    // - Simulation loop (20 Hz)
    // - Region ownership (quadtree)
    // - Redis Streams event sourcing
    // - Supply line BFS
    
    Ok(())
}