use std::sync::Arc;
use salvo::prelude::*;
use crate::infrastructure::container::AppContainer;
use crate::application::user::dtos::{UpdateProfileRequest, UserProfileResponse};
use crate::interface::http::middleware::get_current_user_id;
use crate::core::errors::{AppError, AppResult};

#[handler]
pub async fn get_me(depot: &mut Depot) -> AppResult<Json<UserProfileResponse>> {
    let container = depot.obtain::<Arc<AppContainer>>()
        .map_err(|_| AppError::internal("DI error: AppContainer not found"))?;

    let user_id = get_current_user_id(depot)?;
    
    let profile = container.get_profile_use_case.execute(user_id).await?
        .ok_or_else(|| AppError::not_found("Profile not found"))?;
        
    Ok(Json(profile))
}

#[handler]
pub async fn update_me(
    req: &mut Request,
    depot: &mut Depot,
) -> AppResult<Json<UserProfileResponse>> {
    let container = depot.obtain::<Arc<AppContainer>>()
        .map_err(|_| AppError::internal("DI error: AppContainer not found"))?;

    let user_id = get_current_user_id(depot)?;

    let update_req: UpdateProfileRequest = req.parse_json().await
        .map_err(|e| AppError::bad_request(format!("Invalid JSON: {}", e)))?;

    let profile = container.update_profile_use_case.execute(user_id, update_req).await?;
    
    Ok(Json(profile))
}
