use std::sync::Arc;
use cookie::Cookie;
use salvo::prelude::*;
use salvo::oapi::extract::*;

use crate::application::auth::{
    AuthenticateFirebaseUseCase, FirebaseLoginRequest, FirebaseLoginResponse
};
use crate::infrastructure::persistence::{PostgresUserRepository, PostgresSessionRepository};
use crate::infrastructure::persistence;
use crate::core::errors::{json_ok, JsonResult};
use crate::{utils};

#[endpoint(tags("auth"))]
pub async fn post_authenticate(
    idata: JsonBody<FirebaseLoginRequest>,
    req: &mut Request,
    res: &mut Response,
) -> JsonResult<FirebaseLoginResponse> {
    let idata = idata.into_inner();
    
    // 1. Initialize dependencies (Adapters)
    let pool = persistence::pool();
    let user_repo = Arc::new(PostgresUserRepository { pool });
    let session_repo = Arc::new(PostgresSessionRepository { pool });
    let auth_service = Arc::new(crate::infrastructure::external::firebase_adapter::FirebaseAdapter);
    
    let use_case = AuthenticateFirebaseUseCase::new(user_repo, session_repo, auth_service);

    // 2. Extract request metadata
    let user_agent = idata.user_agent.clone().or_else(|| {
        req.headers()
            .get(salvo::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }).unwrap_or_else(|| "unknown".to_string());

    let ip_address = req.remote_addr().to_string();

    // 3. Execute Use Case
    let result = use_case.execute(
        &idata.id_token,
        idata.device_id.clone(),
        idata.fcm_token.clone(),
        user_agent,
        ip_address
    )
    .await
    .map_err(|e| StatusError::unauthorized().brief(e.to_string()))?;

    // 4. Set Cookie
    let cookie = Cookie::build(("jwt_token", result.jwt_token.clone()))
        .path("/")
        .http_only(true)
        .secure(utils::is_secure_context())
        .build();
    res.add_cookie(cookie);

    // 5. Map to Response DTO
    let resp = FirebaseLoginResponse {
        id: result.user.id,
        firebase_uid: result.user.firebase_uid,
        session_id: result.session.id,
        username: result.user.username,
        display_name: result.user.display_name,
        bio: result.user.bio,
        avatar_url: result.user.avatar_url,
        gender: result.user.gender,
        dob: result.user.dob,
        current_auth_provider_slug: result.current_provider_name,
        current_provider_internal_uid: result.current_provider_internal_uid,
        is_verified: result.is_verified,
        token: result.jwt_token,
        exp: result.exp,
    };

    json_ok(resp)
}
