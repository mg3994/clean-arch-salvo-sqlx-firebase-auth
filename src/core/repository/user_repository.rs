use anyhow::Result;
use async_trait::async_trait;

/// User entity (this would typically be in core/entities/)
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// User repository trait - defines the contract for user persistence
/// This trait should be implemented by infrastructure layer
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their ID
    async fn find_by_id(&self, id: &str) -> Result<Option<User>>;
    
    /// Find a user by their email address
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    
    /// Create a new user
    async fn create(&self, user: User) -> Result<User>;
    
    /// Update an existing user
    async fn update(&self, user: User) -> Result<User>;
    
    /// Delete a user by ID
    async fn delete(&self, id: &str) -> Result<bool>;
}
