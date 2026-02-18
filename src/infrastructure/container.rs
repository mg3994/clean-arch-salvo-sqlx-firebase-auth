use std::sync::Arc;
use sqlx::PgPool;
use crate::application::auth::AuthenticateFirebaseUseCase;
use crate::application::user::{GetProfileUseCase, UpdateProfileUseCase};
use crate::core::services::{I18nService, AuthService};
use crate::infrastructure::persistence::{PostgresUserRepository, PostgresSessionRepository};
use crate::infrastructure::external::firebase_adapter::FirebaseAdapter;
use crate::infrastructure::i18n::TranslationService;
use crate::infrastructure::config::ServerConfig;
use crate::core::errors::{AppError, AppResult};

use firebase_admin_sdk::FirebaseApp;

pub struct AppContainer {
    pub config: Arc<ServerConfig>,
    pub db_pool: PgPool,
    pub i18n: Arc<dyn I18nService>,
    pub firebase_app: Option<Arc<FirebaseApp>>,
    pub authenticate_firebase_use_case: AuthenticateFirebaseUseCase,
    pub get_profile_use_case: GetProfileUseCase,
    pub update_profile_use_case: UpdateProfileUseCase,
}

impl AppContainer {
    pub fn new(config: ServerConfig, db_pool: PgPool, firebase_app: Option<FirebaseApp>) -> Self {
        let config = Arc::new(config);
        let i18n = Arc::new(TranslationService::new(
            config.default_locale.clone(),
            config.fallback_locale.clone(),
        ));
        let firebase_app = firebase_app.map(Arc::new);
        
        // Create repositories
        let user_repo = Arc::new(PostgresUserRepository { pool: db_pool.clone() });
        let session_repo = Arc::new(PostgresSessionRepository { pool: db_pool.clone() });
        
        // Use repos for use cases
        let get_profile_use_case = GetProfileUseCase::new(user_repo.clone());
        let update_profile_use_case = UpdateProfileUseCase::new(user_repo.clone());
        

        // Handle optional FirebaseApp for adapter
        let auth_service: Arc<dyn AuthService> = if let Some(app) = &firebase_app {
            Arc::new(FirebaseAdapter { app: app.clone() })
        } else {
            // Provide a Noop or failing adapter if Firebase is disabled
            // For now, let's just use a dummy that will fail if used
            struct MissingFirebaseAdapter;
            #[async_trait::async_trait]
            impl AuthService for MissingFirebaseAdapter {
                async fn verify_token(&self, _id_token: &str) -> AppResult<crate::core::services::auth_service::ExternalAuthUser> {
                    Err(AppError::internal("Firebase is not configured"))
                }
            }
            Arc::new(MissingFirebaseAdapter)
        };
        
        let authenticate_firebase_use_case = AuthenticateFirebaseUseCase::new(
            user_repo,
            session_repo,
            auth_service,
            config.jwt.clone(),
        );
        
        Self {
            config,
            db_pool,
            i18n,
            firebase_app,
            authenticate_firebase_use_case,
            get_profile_use_case,
            update_profile_use_case,
        }
    }
}
