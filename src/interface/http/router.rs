use salvo::prelude::*;

use crate::interface::http::controllers::auth_controller;
use crate::interface::http::middleware;
use crate::infrastructure::config;

pub fn root() -> Router {
    let config = config::get();
    
    // Auth route with CORS
    let auth_router = Router::with_path("auth")
        .hoop(middleware::cors_middleware())
        .push(Router::with_path("authenticate").post(auth_controller::post_authenticate));

    // API router with RLS and JWT middleware
    // (Example for future protected routes)
    let _api_router = Router::with_path("api")
        .hoop(middleware::cors_middleware())
        .hoop(middleware::auth_db_rls_middleware(&config.jwt));

    Router::new()
        .push(auth_router)
        .push(Router::with_path("health").get(hello))
}

#[handler]
async fn hello() -> &'static str {
    "Backend is healthy"
}

pub fn openapi_specification() -> salvo::oapi::OpenApi {
    salvo::oapi::OpenApi::new("RotiRide API", "0.1.0")
        .merge_router(&root())
}
