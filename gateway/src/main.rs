//! Gateway service main entry point

use anyhow::Result;
use clap::Parser;
use config::{Config, File, Environment};
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

mod settings;
mod auth;
mod websocket;
mod tcp_client;
mod router;

#[derive(Parser, Debug)]
#[command(name = "gateway", version, about = "War2 Gateway Service")]
struct Args {
    #[arg(short, long, default_value = "config/gateway.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    
    let args = Args::parse();
    let settings = Config::builder()
        .add_source(File::with_name(&args.config).required(false))
        .add_source(Environment::with_prefix("GATEWAY"))
        .build()?;
    
    let cfg: settings::GatewayConfig = settings.try_deserialize()?;
    
    info!("Starting War2 Gateway on {}:{}", cfg.gateway.host, cfg.gateway.port);
    
    // TODO: Implement gateway
    // - TLS termination
    // - JWT validation
    // - WebSocket proxy to shards via TCP
    // - Auth endpoints (/auth/login, /auth/register, /auth/refresh)
    
    Ok(())
}