use std::sync::Arc;
use salvo::prelude::*;
use crate::interface::http::controllers::{auth_controller, user_controller};
use crate::interface::http::middleware;
use crate::infrastructure::container::AppContainer;

struct DiHandler {
    container: Arc<AppContainer>,
}

#[handler]
impl DiHandler {
    async fn handle(&self, depot: &mut Depot) {
        depot.insert("app_container", self.container.clone());
    }
}


pub fn root(container: Arc<AppContainer>) -> Router {
    let config = container.config.clone();
    
    // Auth route with CORS
    let auth_router = Router::with_path("auth")
        .hoop(middleware::cors_middleware())
        .push(Router::with_path("authenticate").post(auth_controller::post_authenticate))
        .push(Router::with_path("me")
            .get(user_controller::get_me)
            .post(user_controller::update_me));

    // API router with RLS and JWT middleware
    // (Example for future protected routes)
    let _api_router = Router::with_path("api")
        .hoop(middleware::cors_middleware())
        .hoop(middleware::auth_db_rls_middleware(&config.jwt));

    Router::new()
        .hoop(DiHandler { container })
        .push(auth_router)
        .push(Router::with_path("health").get(hello))
}

#[handler]
async fn hello() -> &'static str {
    "Backend is healthy"
}

pub fn openapi_specification() -> salvo::oapi::OpenApi {
    salvo::oapi::OpenApi::new("RotiRide API", "0.1.0")
}
