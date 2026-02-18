use std::sync::Arc;
use salvo::prelude::*;
use salvo::http::{StatusCode, StatusError};
use salvo::oapi::{self, EndpointOutRegister, ToSchema};
use crate::core::errors::AppError;
use crate::infrastructure::container::AppContainer;

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        let code = match &self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        res.status_code(code);

        // Get AppContainer from Depot
        let container = depot.obtain::<Arc<AppContainer>>().ok();

        // Get language from header or default from config
        let lang = req.header::<String>("accept-language").unwrap_or_else(|| {
            container.as_ref()
                .map(|c| c.config.default_locale.clone())
                .unwrap_or_else(|| "en".to_string())
        });
        let lang = lang.split(',').next().unwrap_or("en").split('-').next().unwrap_or("en");

        let (key, details) = match &self {
            AppError::NotFound(msg) => ("errors.not_found", Some(msg.clone())),
            AppError::Unauthorized(msg) => ("errors.unauthorized", Some(msg.clone())),
            AppError::Forbidden(msg) => ("errors.forbidden", Some(msg.clone())),
            AppError::BadRequest(msg) => ("errors.bad_request", Some(msg.clone())),
            AppError::Conflict(msg) => ("errors.conflict", Some(msg.clone())),
            AppError::Validation(msg) => ("errors.validation_error", Some(msg.clone())),
            AppError::Internal(msg) => {
                tracing::error!(error = msg, "internal error");
                ("errors.internal_error", None)
            }
        };
        
        let mut message = if let Some(container) = container {
            container.i18n.get(key, lang).to_string()
        } else {
            key.to_string()
        };

        if let Some(d) = details {
            if key == "errors.validation_error" || key == "errors.bad_request" || key == "errors.not_found" {
                 if !d.is_empty() {
                    message = format!("{}: {}", message, d);
                 }
            }
        }

        #[derive(serde::Serialize, ToSchema)]
        struct ErrorResponse {
            code: u16,
            message: String,
        }
        res.render(Json(ErrorResponse {
            code: code.as_u16(),
            message,
        }));
    }
}

impl EndpointOutRegister for AppError {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal server error")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Not found")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad request")
                .add_content("application/json", StatusError::to_schema(components)),
        );
         operation.responses.insert(
            StatusCode::UNAUTHORIZED.as_str(),
            oapi::Response::new("Unauthorized")
                .add_content("application/json", StatusError::to_schema(components)),
        );
    }
}
