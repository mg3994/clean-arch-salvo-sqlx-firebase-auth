use async_trait::async_trait;
use sqlx::{self, PgPool};
use uuid::Uuid;

use crate::core::entities::{User, FullUserRecord, AuthIdentity};
use crate::core::repository::UserRepository;
use crate::core::errors::{AppError, AppResult};
use crate::infrastructure::persistence::models::{UserRow, FullUserRecordRow, GenderDb};

pub struct PostgresUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT 
                u.id, u.firebase_uid, un.username, u.display_name, u.bio, u.avatar_url, 
                ai.identifier as phone_number,
                u.gender, u.dob, u.embedding_dirty, 
                u.created_at, u.updated_at, u.deleted_at
            FROM users u
            LEFT JOIN usernames un ON u.id = un.user_id
            LEFT JOIN auth_identities ai ON u.id = ai.user_id AND ai.provider = 'phone'
            WHERE u.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        
        Ok(user.map(Into::into))
    }

    async fn find_by_email(&self, _email: &str) -> AppResult<Option<User>> {
        Ok(None)
    }

    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT 
                u.id, u.firebase_uid, un.username, u.display_name, u.bio, u.avatar_url, 
                ai.identifier as phone_number,
                u.gender, u.dob, u.embedding_dirty, 
                u.created_at, u.updated_at, u.deleted_at
            FROM users u
            LEFT JOIN usernames un ON u.id = un.user_id
            LEFT JOIN auth_identities ai ON u.id = ai.user_id AND ai.provider = 'phone'
            WHERE u.firebase_uid = $1
            "#,
        )
        .bind(firebase_uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        
        Ok(user.map(Into::into))
    }

    async fn create(&self, user: User) -> AppResult<User> {
        let gender_db: Option<GenderDb> = user.gender.clone().map(Into::into);
        let created = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (firebase_uid, display_name, avatar_url, gender, bio, dob)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, firebase_uid, NULL as username, display_name, bio, avatar_url,
                NULL as phone_number,
                gender, dob, embedding_dirty, 
                created_at, updated_at, deleted_at
            "#,
        )
        .bind(user.firebase_uid)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .bind(gender_db)
        .bind(user.bio)
        .bind(user.dob)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        
        Ok(created.into())
    }

    async fn update(&self, user: User) -> AppResult<User> {
        let gender_db: Option<GenderDb> = user.gender.clone().map(Into::into);
        let updated = sqlx::query_as::<_, UserRow>(
            r#"
            WITH updated_user AS (
                UPDATE users
                SET 
                    display_name = $2,
                    avatar_url = $3,
                    gender = $4,
                    dob = $5,
                    bio = $6,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING *
            )
            SELECT 
                u.id, u.firebase_uid, un.username, u.display_name, u.bio, u.avatar_url, 
                ai.identifier as phone_number,
                u.gender, u.dob, u.embedding_dirty, 
                u.created_at, u.updated_at, u.deleted_at
            FROM updated_user u
            LEFT JOIN usernames un ON u.id = un.user_id
            LEFT JOIN auth_identities ai ON u.id = ai.user_id AND ai.provider = 'phone'
            "#,
        )
        .bind(user.id)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .bind(gender_db)
        .bind(user.dob)
        .bind(user.bio)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        
        Ok(updated.into())
    }

    async fn delete(&self, id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query("UPDATE users SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn upsert_user_with_identities(
        &self,
        firebase_uid: &str,
        display_name: Option<String>,
        avatar_url: Option<String>,
        phone_number: Option<String>,
        identities: Vec<AuthIdentity>,
    ) -> AppResult<FullUserRecord> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        // 1. Upsert User (No phone_number column here)
        let row = sqlx::query_as::<_, FullUserRecordRow>(
            r#"
            WITH upserted_user AS (
                INSERT INTO users (firebase_uid, display_name, avatar_url)
                VALUES ($1, $2, $3)
                ON CONFLICT (firebase_uid) DO UPDATE
                SET 
                    firebase_uid = EXCLUDED.firebase_uid
                RETURNING *
            )
            SELECT
                u.id,
                u.firebase_uid,
                u.display_name,
                un.username,
                u.bio,
                u.avatar_url,
                u.gender,
                u.dob,
                u.embedding_dirty,
                u.created_at,
                u.updated_at,
                u.deleted_at,
                CAST(NULL AS TEXT) as phone_number
            FROM upserted_user u
            LEFT JOIN usernames un ON u.id = un.user_id
            "#,
        )
        .bind(firebase_uid)
        .bind(display_name)
        .bind(avatar_url)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        // 2. Establish RLS Session
        let current_user_id = row.id.to_string();
        sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(&current_user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        // 3. Sync Identities
        for identity in identities {
            sqlx::query(
                r#"
                INSERT INTO auth_identities (user_id, provider, provider_uid, identifier, verified_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (provider, provider_uid) DO UPDATE
                SET 
                    identifier = EXCLUDED.identifier,
                    verified_at = COALESCE(auth_identities.verified_at, EXCLUDED.verified_at)
                "#,
            )
            .bind(row.id)
            .bind(identity.provider_slug)
            .bind(identity.provider_uid)
            .bind(identity.identifier)
            .bind(identity.verified_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
        }

        tx.commit().await.map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let mut user_record = FullUserRecord::from(row);
        user_record.phone_number = phone_number; // Use the one we just verified/synced

        Ok(user_record)
    }
}
