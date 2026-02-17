use anyhow::Result;
use async_trait::async_trait;
use sqlx::{self, PgPool};
use uuid::Uuid;

use crate::core::entities::{Session, SessionInput};
use crate::core::repository::SessionRepository;
use crate::infrastructure::persistence::models::SessionRow;

pub struct PostgresSessionRepository {
    pub pool: &'static PgPool,
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn get_active_session(&self, session_id: &Uuid) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT 
                id, user_id, device_id, fcm_token, user_agent, ip_address, 
                auth_exp, revoked_at, created_at, updated_at
            FROM users_sessions
            WHERE id = $1
              AND revoked_at IS NULL
              AND auth_exp > CURRENT_TIMESTAMP
            "#,
        )
        .bind(session_id)
        .fetch_optional(self.pool)
        .await?;
        
        Ok(session.map(Into::into))
    }

    async fn upsert_session(&self, input: SessionInput) -> Result<Session> {
        let session = sqlx::query_as::<_, SessionRow>(
            r#"
            INSERT INTO users_sessions (user_id, device_id, fcm_token, user_agent, ip_address, auth_exp)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, device_id) DO UPDATE
            SET
                fcm_token = COALESCE(EXCLUDED.fcm_token, users_sessions.fcm_token),
                user_agent = EXCLUDED.user_agent,
                ip_address = EXCLUDED.ip_address,
                auth_exp = EXCLUDED.auth_exp,
                revoked_at = NULL,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, user_id, device_id, fcm_token, user_agent, ip_address, 
                      auth_exp, revoked_at, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.device_id)
        .bind(input.fcm_token)
        .bind(input.user_agent)
        .bind(input.ip_address)
        .bind(input.auth_exp)
        .fetch_one(self.pool)
        .await?;
        
        Ok(session.into())
    }

    async fn revoke_session(&self, session_id: &Uuid) -> Result<bool> {
        let result = sqlx::query("UPDATE users_sessions SET revoked_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(session_id)
            .execute(self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_user_sessions(&self, user_id: &Uuid) -> Result<usize> {
        let result = sqlx::query("UPDATE users_sessions SET revoked_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id)
            .execute(self.pool)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }

    async fn delete_expired_sessions(&self) -> Result<usize> {
        let result = sqlx::query("DELETE FROM users_sessions WHERE auth_exp < CURRENT_TIMESTAMP OR revoked_at < CURRENT_TIMESTAMP - INTERVAL '30 days'")
            .execute(self.pool)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
}
