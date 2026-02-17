use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::{Session, SessionInput};

/// Session repository trait - defines the contract for session persistence
/// This trait should be implemented by infrastructure layer
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Get an active session by ID
    /// Returns None if session doesn't exist, is revoked, or expired
    async fn get_active_session(&self, session_id: &Uuid) -> Result<Option<Session>>;
    
    /// Upsert a session (insert or update on conflict)
    /// On conflict with (user_id, device_id), updates the session
    async fn upsert_session(&self, input: SessionInput) -> Result<Session>;
    
    /// Revoke a session by ID
    async fn revoke_session(&self, session_id: &Uuid) -> Result<bool>;
    
    /// Revoke all sessions for a user
    async fn revoke_all_user_sessions(&self, user_id: &Uuid) -> Result<usize>;
    
    /// Delete expired sessions (cleanup task)
    async fn delete_expired_sessions(&self) -> Result<usize>;
}
