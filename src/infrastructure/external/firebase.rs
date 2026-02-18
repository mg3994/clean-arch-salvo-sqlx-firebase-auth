// Firebase initialization logic
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

/// Initialize Firebase Admin SDK with service account JSON
pub async fn init(config: &FirebaseAdminConfig) -> Option<FirebaseApp> {
    // Load the service account key (e.g., from a file)
    if let Some(firebase_service_account_path) = &config.service_account_path {
        // Load the service account key (JSON)
        let service_account_key =
            yup_oauth2::read_service_account_key(firebase_service_account_path)
                .await
                .expect("Failed to read Firebase service account file");

        // Initialize FirebaseApp
        Some(FirebaseApp::new(service_account_key))
    } else {
        None
    }
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
