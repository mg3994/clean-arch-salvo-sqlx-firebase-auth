use std::sync::Arc;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use firebase_admin_sdk::auth::verifier::FirebaseTokenClaims;

use crate::core::entities::{AuthIdentity, SessionInput};
use crate::core::repository::{UserRepository, SessionRepository};
use crate::infrastructure::external::firebase::firebase_admin;
use crate::infrastructure::auth::generate_jwt_token;
use crate::application::auth::dtos::AuthenticationResult;

pub struct AuthenticateFirebaseUseCase {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
}

impl AuthenticateFirebaseUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>, session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { user_repo, session_repo }
    }

    pub async fn execute(
        &self,
        id_token: &str,
        device_id: &str,
        fcm_token: Option<String>,
        user_agent: &str,
        ip_address: &str,
    ) -> Result<AuthenticationResult> {
        // 1. Verify Firebase ID token
        let token_claims: FirebaseTokenClaims = firebase_admin().auth()
            .verify_id_token(id_token)
            .await
            .map_err(|_| anyhow!("Invalid Firebase Token"))?;

        let firebase_uid = &token_claims.sub;
        
        // 2. Extract Provider Info
        let firebase_internal = token_claims.claims.get("firebase")
            .and_then(|v| v.as_object());
            
        let current_provider_name = firebase_internal
            .and_then(|f| f.get("sign_in_provider"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let current_provider_internal_uid = firebase_internal
            .and_then(|f: &serde_json::Map<String, serde_json::Value>| f.get("identities"))
            .and_then(|i: &serde_json::Value| i.get(current_provider_name))
            .and_then(|arr: &serde_json::Value| arr.get(0))
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s: &str| s.to_string());

        let is_email_verified = token_claims.email_verified.unwrap_or(false);
        let signal_time = Some(Utc::now());

        // 3. Extract and Normalize Identities
        let mut identities = Vec::new();
        if let Some(ids_map) = firebase_internal
            .and_then(|f: &serde_json::Map<String, serde_json::Value>| f.get("identities"))
            .and_then(|v: &serde_json::Value| v.as_object()) 
        {
            for (slug, ids) in ids_map {
                if let Some(p_uid) = ids.as_array()
                    .and_then(|arr: &Vec<serde_json::Value>| arr.get(0))
                    .and_then(|v: &serde_json::Value| v.as_str()) 
                {
                    let provider_slug = if slug == "email" { "password" } else { slug.as_str() };
                    
                    let verified_at = match provider_slug {
                        "google.com" | "apple.com" | "phone" => signal_time,
                        "password" => if is_email_verified { signal_time } else { None },
                        _ => None,
                    };

                    identities.push(AuthIdentity {
                        provider_slug: provider_slug.to_string(),
                        provider_uid: p_uid.to_string(),
                        verified_at,
                    });
                }
            }
        }

        // 4. Upsert User using specialized repository method
        let full_user = self.user_repo.upsert_user_with_identities(
            firebase_uid,
            token_claims.name.clone(),
            token_claims.picture.clone(),
            identities
        ).await?;

        // 5. Security Check: Block access if the user is soft-deleted
        if full_user.deleted_at.is_some() {
            return Err(anyhow!("This account has been deactivated."));
        }

        // 6. Upsert Session
        let auth_exp = DateTime::from_timestamp(token_claims.exp as i64, 0)
            .ok_or_else(|| anyhow!("Invalid expiration timestamp from Firebase"))?;

        let session_input = SessionInput {
            user_id: full_user.id,
            device_id: device_id.to_string(),
            fcm_token,
            user_agent: user_agent.to_string(),
            ip_address: ip_address.to_string(),
            auth_exp,
        };

        let session = self.session_repo.upsert_session(session_input).await?;

        // 7. Generate JWT
        // We use the Firebase auth_exp as our JWT expiration
        let (jwt_token, exp) = generate_jwt_token(full_user.id, session.id, Some(auth_exp))?;

        let is_verified = if current_provider_name == "password" {
            is_email_verified
        } else {
            true // Social/Phone providers are implicitly verified by Firebase
        };

        // 8. Convert FullUserRecord to User entity for the result
        // (Assuming FullUserRecord fields map to User or we use FullUserRecord in result)
        // For simplicity, let's assume we can construct a User entity from result
        let user = crate::core::entities::User {
            id: full_user.id,
            firebase_uid: full_user.firebase_uid,
            username: full_user.username,
            display_name: full_user.display_name,
            bio: full_user.bio,
            avatar_url: full_user.avatar_url,
            gender: full_user.gender,
            dob: full_user.dob,
            embedding_dirty: full_user.embedding_dirty,
            created_at: full_user.created_at,
            updated_at: full_user.updated_at,
            deleted_at: full_user.deleted_at,
        };

        Ok(AuthenticationResult {
            user,
            session_id: session.id,
            jwt_token,
            exp,
            current_provider_name: current_provider_name.to_string(),
            current_provider_internal_uid,
            is_verified,
        })
    }
}
