use std::sync::OnceLock;
use crate::infrastructure::config::FirebaseAdminConfig;
use firebase_admin_sdk::{FirebaseApp, yup_oauth2};

//
// window.addEventListener("fb:token", async (e) => {
// const token = e.detail.idToken;
//
// await fetch("/api/auth/refresh_session", {
// method: "POST",
// headers: { "Content-Type": "application/json" },
// body: JSON.stringify({ id_token: token }),
// });
// });
//
// window.addEventListener("fb:logout", () => {
// console.log("User logged out");
// });

// Firebase Admin SDK
pub static FIREBASE_ADMIN: OnceLock<FirebaseApp> = OnceLock::new();

/// Initialize Firebase Admin SDK with service account JSON
pub async fn init(config: &FirebaseAdminConfig) {
    // Load the service account key (e.g., from a file)
    if let Some(firebase_service_account_path) = &config.service_account_path {
        // Load the service account key (JSON)
        let service_account_key =
            yup_oauth2::read_service_account_key(firebase_service_account_path)
                .await
                .expect("Failed to read Firebase service account file");

        // Initialize FirebaseApp
        let app = FirebaseApp::new(service_account_key);
        // Option 1: safer manual check
        if crate::infrastructure::external::firebase::FIREBASE_ADMIN.set(app).is_err() {
            println!("Firebase Admin already initialized, skipping");
        }
    }
}

/// Get global Firebase Admin instance
pub fn firebase_admin() -> &'static FirebaseApp {
    FIREBASE_ADMIN.get().expect("Firebase Admin is not initialized")
}

// Firebase features configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseFeatures {
    pub auth: bool,
    pub messaging: bool,
    pub storage: bool,
}

impl Default for FirebaseFeatures {
    fn default() -> Self {
        Self {
            auth: true,
            messaging: true,
            storage: false,
        }
    }
}
