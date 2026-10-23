//! Gateway service main entry point

use anyhow::Result;
use axum::{routing::{get, post}, Router};
use clap::Parser;
use config::{Config, File, Environment};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod settings;
mod auth;
mod routes;
mod websocket;
mod tcp_client;
mod router;

#[derive(Parser, Debug)]
#[command(name = "gateway", version, about = "War2 Gateway Service")]
struct Args {
    #[arg(short, long, default_value = "config/gateway.toml")]
    config: String,
}

#[derive(Clone)]
struct AppState {
    config: Arc<settings::GatewayConfig>,
    pool: PgPool,
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
    
    // Connect to PostgreSQL
    let pool = PgPool::connect(&cfg.postgres.url).await?;
    
    // Run migrations
    sqlx::migrate!("../migrations").run(&pool).await?;
    
    let state = Arc::new(AppState {
        config: Arc::new(cfg.clone()),
        pool,
    });
    
    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(routes::register))
        .route("/auth/login", post(routes::login))
        .route("/auth/refresh", post(routes::refresh))
        .route("/auth/logout", post(routes::logout))
        .layer(cors)
        .with_state(state);
    
    let listener = TcpListener::bind((cfg.gateway.host.as_str(), cfg.gateway.port)).await?;
    
    info!("Starting War2 Gateway on {}:{}", cfg.gateway.host, cfg.gateway.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}