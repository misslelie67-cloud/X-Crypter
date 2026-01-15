use crate::auth::types::*;
use crate::storage::UpstashClient;
use tauri::State;
use uuid::Uuid;

const QR_TTL_SECS: u64 = 300; // 5 minutes

#[tauri::command]
pub async fn generate_qr_code(upstash: State<'_, UpstashClient>) -> Result<QRResponse, String> {
    let token = Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + QR_TTL_SECS as i64;

    let session = QRSession {
        token: token.clone(),
        chat_id: None,
        status: AuthStatus::Created,
        created_at: now,
        expires_at,
    };

    // Store in Upstash
    let key = format!("qr:{}", token);
    upstash.set_json(&key, &session, QR_TTL_SECS).await?;

    // Generate QR code with Telegram deep link
    let bot_username = std::env::var("TELEGRAM_BOT_USERNAME")
        .map_err(|_| "TELEGRAM_BOT_USERNAME not set")?;
    let deep_link = format!("https://t.me/{}?start={}", bot_username, token);
    
    // Generate QR code SVG
    let qr_code = generate_qr_svg(&deep_link)?;

    Ok(QRResponse {
        token,
        qr_code,
        expires_in: QR_TTL_SECS as i64,
    })
}

#[tauri::command]
pub async fn check_auth_status(
    token: String,
    upstash: State<'_, UpstashClient>,
) -> Result<AuthStatusResponse, String> {
    let key = format!("qr:{}", token);
    let session: Option<QRSession> = upstash.get_json(&key).await?;

    match session {
        Some(session) => {
            let now = chrono::Utc::now().timestamp();
            let expired = now > session.expires_at;

            Ok(AuthStatusResponse {
                status: if expired {
                    AuthStatus::Expired
                } else {
                    session.status
                },
                expired,
            })
        }
        None => Ok(AuthStatusResponse {
            status: AuthStatus::Expired,
            expired: true,
        }),
    }
}

#[tauri::command]
pub async fn exchange_token(
    token: String,
    upstash: State<'_, UpstashClient>,
) -> Result<SessionResponse, String> {
    let key = format!("qr:{}", token);
    let session: Option<QRSession> = upstash.get_json(&key).await?;

    match session {
        Some(session) if session.status == AuthStatus::Approved => {
            let chat_id = session.chat_id.ok_or("No chat_id in session")?;
            
            // Create user session
            let session_id = Uuid::new_v4().simple().to_string();
            let now = chrono::Utc::now().timestamp();
            let expires_at = now + (30 * 86400); // 30 days

            let session_key = format!("session:{}", session_id);
            let user_session = serde_json::json!({
                "session_id": session_id,
                "chat_id": chat_id,
                "created_at": now,
                "expires_at": expires_at,
            });

            upstash.set_json(&session_key, &user_session, 30 * 86400).await?;

            // Delete QR session
            upstash.delete(&key).await?;

            Ok(SessionResponse {
                session_id,
                chat_id,
                expires_at,
            })
        }
        _ => Err("Session not approved or expired".to_string()),
    }
}

#[tauri::command]
pub async fn validate_session(
    session_id: String,
    upstash: State<'_, UpstashClient>,
) -> Result<SessionResponse, String> {
    let key = format!("session:{}", session_id);
    let session: Option<serde_json::Value> = upstash.get_json(&key).await?;

    match session {
        Some(session) => {
            let expires_at = session["expires_at"]
                .as_i64()
                .ok_or("Invalid expires_at")?;
            let chat_id = session["chat_id"]
                .as_str()
                .ok_or("Invalid chat_id")?
                .to_string();
            let session_id = session["session_id"]
                .as_str()
                .ok_or("Invalid session_id")?
                .to_string();

            let now = chrono::Utc::now().timestamp();
            if now > expires_at {
                upstash.delete(&key).await?;
                return Err("Session expired".to_string());
            }

            Ok(SessionResponse {
                session_id,
                chat_id,
                expires_at,
            })
        }
        None => Err("Session not found".to_string()),
    }
}

fn generate_qr_svg(data: &str) -> Result<String, String> {
    use qrcode::{QrCode, render::svg};
    
    let code = QrCode::new(data.as_bytes()).map_err(|e| e.to_string())?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(280, 280)
        .build();
    
    Ok(svg)
}
