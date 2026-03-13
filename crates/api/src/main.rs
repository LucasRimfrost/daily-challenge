use db::connection;
use shared::config::Config;
use sqlx::PgPool;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod middleware;
mod routes;

#[allow(dead_code)]
#[derive(Clone)]
struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = Config::from_env().expect("Failed to load configuration");

    let pool = connection::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let addr = format!("{}:{}", config.host, config.port);

    let state = AppState { pool, config };
    let router = routes::router(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", addr));

    tracing::info!("Listening on {addr}");
    axum::serve(listener, router).await.expect("Server error");
}

/// Initialize the tracing subscriber with env-filter support.
///
/// Reads `RUST_LOG` from the environment, falling back to sensible defaults.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,codeforge=debug,tower_http=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}
