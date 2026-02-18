use std::sync::Arc;
use backend::infrastructure::{config, persistence, external::firebase, container::AppContainer};
use backend::interface::http::{router, middleware};
use salvo::catcher::Catcher;
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::server::ServerHandle;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Initialize configuration
    let cfg = config::init();

    // Initialize database
    let pool = persistence::init(&cfg.db).await;

    // Initialize Firebase Admin (if configured)
    let firebase_app = if let Some(firebase_admin_cfg) = &cfg.firebase.admin {
        firebase::init(firebase_admin_cfg).await
    } else {
        None
    };

    // Initialize logging
    let _guard = backend::infrastructure::logging::init(&cfg.log);
    tracing::info!("log level: {}", &cfg.log.filter_level);

    // Create AppContainer
    let container = Arc::new(AppContainer::new(cfg.clone(), pool, firebase_app));

    // Create service with routers and hoops
    let service = Service::new(router::root(container))
        .catcher(Catcher::default().hoop(middleware::error_404))
        .hoop(middleware::cors_middleware());

    println!("🔄 listen on {}", &cfg.listen_addr);
    println!("Debug: TLS config is {:?}", cfg.tls);

    // Start server with or without TLS
    if let Some(tls) = &cfg.tls {
        println!(
            "📖 Open API Page (test Quinn): https://{}/scalar",
            cfg.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Auth Page (test Quinn): https://{}/auth",
            cfg.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let config = RustlsConfig::new(Keycert::new()
            .cert(std::fs::read(&tls.cert).expect("cert file not found"))
            .key(std::fs::read(&tls.key).expect("key file not found")));
        let acceptor = QuinnListener::new(config.clone().build_quinn_config().unwrap(), cfg.listen_addr.clone())
            .join(TcpListener::new(cfg.listen_addr.clone()).rustls(config))
            .bind()
            .await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    } else {
        println!(
            "📖 Open API Page: http://{}/scalar",
            cfg.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Login Page: http://{}/login",
            cfg.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let acceptor = TcpListener::new(cfg.listen_addr.clone()).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    }
}

async fn shutdown_signal(handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("ctrl_c signal received"),
        _ = terminate => info!("terminate signal received"),
    }
    handle.stop_graceful(std::time::Duration::from_secs(60));
}

#[cfg(test)]
mod tests {
    use backend::infrastructure::{config, persistence};
    use backend::interface::http::router;
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    #[tokio::test]
    async fn test_hello_world() {
        use std::sync::Arc;
        use backend::infrastructure::container::AppContainer;
        
        let cfg = config::init();
        let pool = persistence::init(&cfg.db).await;
        let container = Arc::new(AppContainer::new(cfg.clone(), pool, None));

        let service = Service::new(router::root(container));

        let content = TestClient::get(format!(
            "http://{}",
            cfg.listen_addr.replace("0.0.0.0", "127.0.0.1")
        ))
        .send(&service)
        .await
        .take_string()
        .await
        .unwrap();
        // Adjust expectation to current health check logic if it changed
        // Previous root() had health check "Backend is healthy"
        assert_eq!(content, "Backend is healthy");
    }
}
