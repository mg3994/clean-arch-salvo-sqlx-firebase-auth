use salvo::http::{ParseError, StatusCode, StatusError};
use salvo::oapi::{self, EndpointOutRegister, ToSchema};
use salvo::prelude::*;
use thiserror::Error;
use crate::infrastructure::i18n::get_translator;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("public: `{0}`")]
    Public(String),
    #[error("internal: `{0}`")]
    Internal(String),
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),
    #[error("http parse error:`{0}`")]
    HttpParse(#[from] ParseError),
    #[error("anyhow error:`{0}`")]
    Anyhow(#[from] anyhow::Error),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("validation error:`{0}`")]
    Validation(#[from] validator::ValidationErrors),
}
impl AppError {
    pub fn public<S: Into<String>>(msg: S) -> Self {
        Self::Public(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let code = match &self {
            Self::HttpStatus(e) => e.code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        res.status_code(code);

        // Get language from header
        let lang = req.header::<String>("accept-language").unwrap_or_else(|| "en".to_string());
        // Simple language negotiation: take the first part of 'en-US' or just 'en'
        let lang = lang.split(',').next().unwrap_or("en").split('-').next().unwrap_or("en");

        let translator = get_translator();

        let (key, details) = match &self {
            Self::Salvo(e) => {
                tracing::error!(error = ?e, "salvo error");
                ("salvo_error", Some(format!("Unknown error happened in salvo.")))
            }
            Self::Public(msg) => ("public_error", Some(msg.clone())), // You might want a specific key or just pass msg
            Self::Internal(msg) => {
                tracing::error!(msg = msg, "internal error");
                ("internal_error", None)
            }
             Self::HttpStatus(e) => {
                 match e.code {
                     StatusCode::NOT_FOUND => ("not_found", None),
                     StatusCode::BAD_REQUEST => ("bad_request", None),
                     _ => ("unknown_error", Some(e.brief.clone()))
                 }
             },
             Self::Validation(e) => ("validation_error", Some(e.to_string())),
            _ => ("unknown_error", Some(self.to_string())),
        };
        
        // If it's a public error with a custom message, we might return it directly or try to translate if it's a key
        // For simplicity, if it's "public_error", we use the details as the message (assuming it's already a message)
        // Or if the details match a key, we translate it.
        
        let message = if key == "public_error" {
             details.unwrap_or_default()
        } else {
             let mut msg = translator.get(key, lang).to_string();
             if let Some(d) = details {
                 if key == "validation_error" {
                     msg = format!("{}: {}", msg, d);
                 } else if key == "unknown_error" {
                      msg = format!("{}: {}", msg, d);
                 }
             }
             msg
        };

        // We can return a consistent JSON structure
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
    }
}
