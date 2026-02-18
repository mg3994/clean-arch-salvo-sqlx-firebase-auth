use std::sync::Arc;
use chrono::Utc;

use crate::core::entities::{AuthIdentity, SessionInput};
use crate::core::repository::{UserRepository, SessionRepository};
use crate::core::services::AuthService;
use crate::infrastructure::auth::generate_jwt_token;
use crate::application::auth::dtos::AuthenticationResult;
use crate::core::errors::AppResult;

use crate::infrastructure::config::JwtConfig;

pub struct AuthenticateFirebaseUseCase {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    auth_service: Arc<dyn AuthService>,
    jwt_config: JwtConfig,
}

impl AuthenticateFirebaseUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        auth_service: Arc<dyn AuthService>,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            auth_service,
            jwt_config,
        }
    }

    pub async fn execute(
        &self,
        id_token: &str,
        device_id: String,
        fcm_token: Option<String>,
        user_agent: String,
        ip_address: String,
    ) -> AppResult<AuthenticationResult> {
        // 1. Verify Firebase Token via AuthService Port
        let external_user = self.auth_service.verify_token(id_token).await?;

        // 2. Normalize identities
        let identities: Vec<AuthIdentity> = external_user.identities.into_iter()
            .map(|id| AuthIdentity {
                provider_slug: id.provider_slug,
                provider_uid: id.provider_uid,
                identifier: id.identifier,
                verified_at: Some(Utc::now()), 
            })
            .collect();

        // 3. Sync User with Database
        let full_user = self.user_repo.upsert_user_with_identities(
            &external_user.uid,
            external_user.display_name,
            external_user.photo_url,
            external_user.phone_number.clone(),
            identities,
        ).await?;

        // 4. Manage Session
        let session_input = SessionInput {
            user_id: full_user.id,
            device_id,
            fcm_token,
            user_agent,
            ip_address,
            auth_exp: external_user.expiration,
        };

        let session = self.session_repo.upsert_session(session_input).await?;

        // 5. Generate Internal JWT
        let (token, expiry) = generate_jwt_token(
            full_user.id, 
            session.id, 
            &self.jwt_config.secret,
            self.jwt_config.expiry,
            Some(external_user.expiration)
        )?;

        Ok(AuthenticationResult {
            user: full_user,
            session,
            jwt_token: token,
            exp: expiry,
            current_provider_name: external_user.provider_id,
            current_provider_internal_uid: Some(external_user.uid),
            is_verified: external_user.email_verified,
        })
    }
}
