use std::sync::Arc;
use uuid::Uuid;
use crate::core::repository::user_repository::UserRepository;
use crate::application::user::dtos::UserProfileResponse;
use crate::core::errors::AppResult;

pub struct GetProfileUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl GetProfileUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> AppResult<Option<UserProfileResponse>> {
        let user = self.user_repo.find_by_id(&user_id).await?;
        Ok(user.map(UserProfileResponse::from))
    }
}
