use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FirebaseAdminConfig {
    // Preferred: path to service account JSON file
    pub service_account_path: Option<String>,
    // Alternative: inline service account JSON string
    pub service_account_json: Option<String>,
    
    // Optional: inline service account fields (if not using path or json string)
    pub project_id: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: Option<String>,
    pub client_email: Option<String>,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
    pub auth_provider_x509_cert_url: Option<String>,
    pub client_x509_cert_url: Option<String>,
}
