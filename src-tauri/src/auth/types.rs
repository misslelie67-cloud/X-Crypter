use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthStatus {
    Created,
    Pending,
    Approved,
    Denied,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRSession {
    pub token: String,
    pub chat_id: Option<String>,
    pub status: AuthStatus,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub chat_id: String,
    pub key_hash: String,
    pub expires_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct QRResponse {
    pub token: String,
    pub qr_code: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub status: AuthStatus,
    pub expired: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub chat_id: String,
    pub expires_at: i64,
}
