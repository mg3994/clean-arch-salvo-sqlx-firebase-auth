use std::sync::Arc;
use uuid::Uuid;
use crate::core::repository::user_repository::UserRepository;
use crate::application::user::dtos::{UpdateProfileRequest, UserProfileResponse};
use crate::core::errors::{AppError, AppResult};

pub struct UpdateProfileUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl UpdateProfileUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, user_id: Uuid, req: UpdateProfileRequest) -> AppResult<UserProfileResponse> {
        let mut user = self.user_repo.find_by_id(&user_id).await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        if let Some(username) = req.username { user.username = Some(username); }
        if let Some(display_name) = req.display_name { user.display_name = Some(display_name); }
        if let Some(bio) = req.bio { user.bio = Some(bio); }
        if let Some(avatar_url) = req.avatar_url { user.avatar_url = Some(avatar_url); }
        if let Some(phone_number) = req.phone_number { user.phone_number = Some(phone_number); }
        if let Some(gender) = req.gender { user.gender = Some(gender); }
        if let Some(dob) = req.dob { user.dob = Some(dob); }

        let updated_user = self.user_repo.update(user).await?;
        Ok(UserProfileResponse::from(updated_user))
    }
}
