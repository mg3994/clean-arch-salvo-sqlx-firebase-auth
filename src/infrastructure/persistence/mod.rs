use sqlx::postgres::PgPool;

use crate::infrastructure::config::DbConfig;

pub mod user_repo_impl;
pub mod session_repo_impl;
pub mod models;

pub use user_repo_impl::PostgresUserRepository;
pub use session_repo_impl::PostgresSessionRepository;

pub async fn init(config: &DbConfig) -> PgPool {
    PgPool::connect(&config.url).await
        .expect("Database connection failed.")
}
