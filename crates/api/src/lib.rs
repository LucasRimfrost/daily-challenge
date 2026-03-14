use shared::config::Config;
use sqlx::PgPool;

pub mod handlers;
pub mod middleware;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}
