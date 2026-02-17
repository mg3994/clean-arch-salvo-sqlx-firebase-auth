use salvo::cors::{Cors, CorsHandler};
use salvo::http::Method;
use salvo::prelude::*;

pub fn cors_hoop() -> CorsHandler {
    Cors::new()
        .allow_origin("*")
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers("*")
        .into_handler()
}

#[handler]
pub async fn error_404(res: &mut Response) {
    res.status_code(StatusCode::NOT_FOUND);
    res.render(Json(serde_json::json!({
        "code": 404,
        "message": "Not Found"
    })));
}
