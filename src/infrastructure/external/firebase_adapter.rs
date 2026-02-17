use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime};

use crate::core::services::auth_service::{AuthService, ExternalAuthUser, ExternalIdentity};
use crate::infrastructure::external::firebase::firebase_admin;

pub struct FirebaseAdapter;

#[async_trait]
impl AuthService for FirebaseAdapter {
    async fn verify_token(&self, id_token: &str) -> Result<ExternalAuthUser> {
        let token_claims = firebase_admin().auth()
            .verify_id_token(id_token)
            .await
            .map_err(|e| anyhow!("Firebase verification failed: {}", e))?;

        let firebase_internal = token_claims.claims.get("firebase")
            .and_then(|v| v.as_object());
            
        let provider_id = firebase_internal
            .and_then(|f| f.get("sign_in_provider"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut identities = Vec::new();
        
        // Extract top-level identifier hints
        let email = token_claims.email.clone();
        let phone_number = token_claims.claims.get("phone_number")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(ids_map) = firebase_internal
            .and_then(|f: &serde_json::Map<String, serde_json::Value>| f.get("identities"))
            .and_then(|v: &serde_json::Value| v.as_object()) 
        {
            for (slug, ids) in ids_map {
                if let Some(p_uid) = ids.as_array()
                    .and_then(|arr: &Vec<serde_json::Value>| arr.get(0))
                    .and_then(|v: &serde_json::Value| v.as_str()) 
                {
                    let provider_slug = match slug.as_str() {
                        "email" => "password",
                        "phone" => "phone",
                        other => other,
                    };

                    // Determine identifier for this provider
                    let identifier = match provider_slug {
                        "password" => email.clone(),
                        "phone" => phone_number.clone(),
                        _ => None, // For Google/Apple, provider_uid is often the only unique ID we care about here, 
                                   // though we could store email if available.
                    };
                    
                    identities.push(ExternalIdentity {
                        provider_slug: provider_slug.to_string(),
                        provider_uid: p_uid.to_string(),
                        identifier,
                    });
                }
            }
        }

        let expiration = DateTime::from_timestamp(token_claims.exp as i64, 0)
            .ok_or_else(|| anyhow!("Invalid expiration timestamp from Firebase"))?;

        Ok(ExternalAuthUser {
            uid: token_claims.sub,
            email,
            email_verified: token_claims.email_verified.unwrap_or(false),
            display_name: token_claims.name,
            photo_url: token_claims.picture,
            phone_number: phone_number.clone(),
            expiration,
            identities,
            provider_id,
        })
    }
}
