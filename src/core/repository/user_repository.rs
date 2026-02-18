use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::{User, FullUserRecord, AuthIdentity};
use crate::core::errors::AppResult;

/// User repository trait - defines the contract for user persistence
/// This trait should be implemented by infrastructure layer
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their ID
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<User>>;
    
    /// Find a user by their email address
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    
    /// Find a user by Firebase UID
    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> AppResult<Option<User>>;
    
    /// Create a new user
    async fn create(&self, user: User) -> AppResult<User>;
    
    /// Update an existing user
    async fn update(&self, user: User) -> AppResult<User>;
    
    /// Delete a user by ID (soft delete)
    async fn delete(&self, id: &Uuid) -> AppResult<bool>;
    
    /// Upsert user with Firebase identities (for authentication flow)
    /// This handles the complex transaction of:
    /// 1. Upserting user record
    /// 2. Setting RLS context
    /// 3. Upserting auth_identities
    async fn upsert_user_with_identities(
        &self,
        firebase_uid: &str,
        display_name: Option<String>,
        avatar_url: Option<String>,
        phone_number: Option<String>,
        identities: Vec<AuthIdentity>,
    ) -> AppResult<FullUserRecord>;
}
