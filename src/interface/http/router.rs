use salvo::prelude::*;

#[handler]
async fn hello() -> &'static str {
    "Hello World from salvo"
}

pub fn root() -> Router {
    Router::new().get(hello)
}
