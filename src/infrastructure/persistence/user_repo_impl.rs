use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::entities::{User, FullUserRecord, AuthIdentity};
use crate::core::repository::UserRepository;

pub struct PostgresUserRepository {
    pool: &'static PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, firebase_uid, username, display_name, bio, avatar_url, 
                gender as "gender: Gender", dob, embedding_dirty, 
                created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }

    async fn find_by_email(&self, _email: &str) -> Result<Option<User>> {
        Ok(None)
    }

    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, firebase_uid, username, display_name, bio, avatar_url, 
                gender as "gender: Gender", dob, embedding_dirty, 
                created_at, updated_at, deleted_at
            FROM users
            WHERE firebase_uid = $1
            "#,
        )
        .bind(firebase_uid)
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }

    async fn create(&self, user: User) -> Result<User> {
        let created = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (firebase_uid, display_name, avatar_url, gender)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, firebase_uid, username, display_name, bio, avatar_url, 
                gender as "gender: Gender", dob, embedding_dirty, 
                created_at, updated_at, deleted_at
            "#,
        )
        .bind(user.firebase_uid)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .bind(user.gender)
        .fetch_one(self.pool)
        .await?;
        
        Ok(created)
    }

    async fn update(&self, user: User) -> Result<User> {
        let updated = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET 
                display_name = $2,
                avatar_url = $3,
                gender = $4,
                dob = $5,
                bio = $6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING 
                id, firebase_uid, username, display_name, bio, avatar_url, 
                gender as "gender: Gender", dob, embedding_dirty, 
                created_at, updated_at, deleted_at
            "#,
        )
        .bind(user.id)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .bind(user.gender)
        .bind(user.dob)
        .bind(user.bio)
        .fetch_one(self.pool)
        .await?;
        
        Ok(updated)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query("UPDATE users SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn upsert_user_with_identities(
        &self,
        firebase_uid: &str,
        display_name: Option<String>,
        avatar_url: Option<String>,
        identities: Vec<AuthIdentity>,
    ) -> Result<FullUserRecord> {
        let mut tx = self.pool.begin().await?;

        // 1. Upsert User
        let user_record = sqlx::query_as::<_, FullUserRecord>(
            r#"
            WITH upserted_user AS (
                INSERT INTO users (firebase_uid, display_name, avatar_url)
                VALUES ($1, $2, $3)
                ON CONFLICT (firebase_uid) DO UPDATE
                SET firebase_uid = EXCLUDED.firebase_uid
                RETURNING *
            )
            SELECT
                u.id,
                u.firebase_uid,
                u.display_name,
                un.username AS "username?",
                u.bio,
                u.avatar_url,
                u.gender AS "gender: Gender",
                u.dob,
                u.embedding_dirty,
                u.created_at,
                u.updated_at,
                u.deleted_at
            FROM upserted_user u
            LEFT JOIN usernames un ON u.id = un.user_id
            "#,
        )
        .bind(firebase_uid)
        .bind(display_name)
        .bind(avatar_url)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Establish RLS Session
        let current_user_id = user_record.id.to_string();
        sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(&current_user_id)
            .execute(&mut *tx)
            .await?;

        // 3. Sync Identities
        for identity in identities {
            sqlx::query(
                r#"
                INSERT INTO auth_identities (user_id, provider, provider_uid, verified_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (provider, provider_uid) DO UPDATE
                SET verified_at = COALESCE(auth_identities.verified_at, EXCLUDED.verified_at)
                "#,
            )
            .bind(user_record.id)
            .bind(identity.provider_slug)
            .bind(identity.provider_uid)
            .bind(identity.verified_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(user_record)
    }
}

