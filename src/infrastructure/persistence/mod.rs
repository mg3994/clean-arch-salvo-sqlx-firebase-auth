use std::sync::OnceLock;
use sqlx::postgres::PgPool;

use crate::infrastructure::config::DbConfig;

pub mod user_repo_impl;
pub mod session_repo_impl;
pub mod models;

pub use user_repo_impl::PostgresUserRepository;
pub use session_repo_impl::PostgresSessionRepository;

pub static SQLX_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init(config: &DbConfig) {
    let sqlx_pool = PgPool::connect(&config.url).await
        .expect("Database connection failed.");
    SQLX_POOL
        .set(sqlx_pool)
        .expect("sqlx pool should be set")
}

pub fn pool() -> &'static PgPool {
    SQLX_POOL.get().expect("sqlx pool should be set")
}
