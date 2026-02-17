use anyhow::Result;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::config;

/// JWT Claims structure with DateTime instead of primitive timestamps
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims {
    pub uid: String,            // User ID stored as String in JWT
    pub sid: Uuid,              // Session ID
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,     // Expiration timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,     // Issued at timestamp
}

/// Generate a JWT token for a user session
/// Returns the token string and expiration DateTime
pub fn generate_jwt_token(
    uid: impl Into<Uuid>, 
    sid: impl Into<Uuid>,
    external_exp: Option<DateTime<Utc>>
) -> Result<(String, DateTime<Utc>)> {
    let now = Utc::now();
    
    // Use external expiration if provided (e.g. from Firebase), 
    // otherwise fallback to internal config
    let exp = external_exp.unwrap_or_else(|| {
        now + chrono::Duration::seconds(config::get().jwt.expiry)
    });
    
    let claims = JwtClaims {
        uid: uid.into().to_string(),
        sid: sid.into(),
        exp,
        iat: now,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config::get().jwt.secret.as_bytes()),
    )?;
    
    Ok((token, exp))
}

/// Decode and validate JWT token, returning claims if valid
pub fn get_token_claims(token: &str) -> Option<JwtClaims> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(config::get().jwt.secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims)
}

/// Fast signature validation check (for UI checks)
/// Only validates the JWT signature and expiration
pub fn is_jwt_token_signature_valid(token: &str) -> bool {
    get_token_claims(token).is_some()
}

/// Secure session validation (for API checks)
/// Validates both signature and checks database session state
pub async fn is_jwt_session_active(token: &str, pool: &PgPool) -> bool {
    // 1. Validate token signature (checks JWT expiration internally)
    let claims = match get_token_claims(token) {
        Some(c) => c,
        None => return false,
    };
    
    // 2. Check database session state
    // We check if the session is revoked OR if the original provider (Firebase) 
    // grant has expired (auth_exp).
    let result = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM users_sessions
        WHERE id = $1
          AND revoked_at IS NULL
          AND auth_exp > $2
        "#,
    )
    .bind(claims.sid)
    .bind(Utc::now()) // Check against current time
    .fetch_optional(pool)
    .await;
    
    match result {
        Ok(Some(user_id)) => user_id.to_string() == claims.uid,
        _ => false,
    }
}

