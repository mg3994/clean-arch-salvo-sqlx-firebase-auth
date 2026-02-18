use salvo::jwt_auth::{ConstDecoder, JwtAuth, JwtAuthState, JwtAuthDepotExt};
use salvo::prelude::*;

use crate::infrastructure::config::JwtConfig;
use crate::infrastructure::auth::JwtClaims;
use crate::utils;

use crate::infrastructure::container::AppContainer;
use std::sync::Arc;

pub fn jwt_auth_handler(config: &JwtConfig) -> JwtAuth<JwtClaims, ConstDecoder> {
    JwtAuth::new(ConstDecoder::from_secret(
        config.secret.as_bytes(),
    ))
    .finders(utils::get_token_finders())
    .force_passed(false)
}

pub struct DbRlsMiddleware {
    pub auth: JwtAuth<JwtClaims, ConstDecoder>,
}

#[handler]
impl DbRlsMiddleware {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 1. Run the JWT Auth extraction manually
        self.auth.handle(req, depot, res, ctrl).await;

        // 2. Get AppContainer
        let container = match depot.obtain::<Arc<AppContainer>>() {
            Ok(c) => c,
            Err(_) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                ctrl.skip_rest();
                return;
            }
        };

        // 3. Check if JwtAuth succeeded
        if depot.jwt_auth_state() != JwtAuthState::Authorized {
            // Error handling via Translation Service
            let lang = req.header::<String>("accept-language").unwrap_or_else(|| "en".to_string());
            let lang = lang.split(',').next().unwrap_or("en").split('-').next().unwrap_or("en");
            
            let msg = container.i18n.get("errors.unauthorized", lang);
            
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "code": 401,
                "message": msg
            })));
            ctrl.skip_rest();
            return;
        }

        // 4. Get the claims
        let uid = if let Some(data) = depot.jwt_auth_data::<JwtClaims>() {
            data.claims.uid.clone()
        } else {
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
            return;
        };

        // 5. Start Transaction
        let pool = &container.db_pool;
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                ctrl.skip_rest();
                return;
            }
        };

        // 6. RLS setup: Set the session variable in Postgres
        let setup = sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(&uid)
            .execute(&mut *tx)
            .await;

        if setup.is_err() {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            ctrl.skip_rest();
            return;
        }

        // 7. Store transaction in Depot for the downstream handlers
        depot.insert("tx", tx);

        ctrl.call_next(req, depot, res).await;
    }
}

pub fn auth_db_rls_middleware(config: &JwtConfig) -> DbRlsMiddleware {
    DbRlsMiddleware {
        auth: jwt_auth_handler(config)
    }
}
pub fn get_current_user_id(depot: &Depot) -> crate::core::errors::AppResult<uuid::Uuid> {
    let claims = depot.jwt_auth_data::<crate::infrastructure::auth::jwt::JwtClaims>()
        .ok_or_else(|| {
            crate::core::errors::AppError::unauthorized("Unauthorized: No JWT data found")
        })?;

    uuid::Uuid::parse_str(&claims.claims.uid).map_err(|_| {
        crate::core::errors::AppError::unauthorized("Invalid user ID in token")
    })
}
