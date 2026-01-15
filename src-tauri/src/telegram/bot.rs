use crate::auth::types::{AuthStatus, QRSession, User};
use crate::crypto::{decrypt_data, hash_chat_id};
use crate::storage::UpstashClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Deserialize)]
struct UpdatesResponse {
    result: Vec<Update>,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Deserialize)]
struct Message {
    chat: Chat,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    id: String,
    from: CallbackUser,
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CallbackUser {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Debug, Serialize)]
struct InlineButton {
    text: String,
    callback_data: String,
}

pub struct TelegramBot {
    client: Client,
    bot_token: String,
    upstash: UpstashClient,
}

impl TelegramBot {
    pub fn new(bot_token: String, upstash: UpstashClient) -> Self {
        Self {
            client: Client::new(),
            bot_token,
            upstash,
        }
    }

    pub async fn start_polling(self: Arc<Self>) {
        // Delete webhook first if it exists
        if let Err(e) = self.delete_webhook().await {
            eprintln!("⚠️ Warning: Failed to delete webhook: {}", e);
        } else {
            println!("✅ Webhook deleted successfully");
        }

        println!("🤖 Telegram bot polling started...");
        let mut offset: i64 = 0;
        let mut poll_interval = interval(Duration::from_secs(2));

        loop {
            poll_interval.tick().await;

            match self.get_updates(offset).await {
                Ok(updates) => {
                    for update in updates {
                        // Handle text messages
                        if let Some(message) = update.message {
                            if let Some(text) = message.text {
                                if text.starts_with("/start ") {
                                    let token = text.trim_start_matches("/start ").trim();
                                    let chat_id = message.chat.id.to_string();

                                    println!(
                                        "📱 Received /start command from chat_id: {}",
                                        chat_id
                                    );

                                    if let Err(e) = self.handle_start_command(token, &chat_id).await
                                    {
                                        eprintln!("❌ Error handling /start: {}", e);
                                    }
                                }
                            }
                        }

                        // Handle callback queries (button clicks)
                        if let Some(callback) = update.callback_query {
                            let chat_id = callback.from.id.to_string();

                            if let Some(data) = callback.data {
                                println!("🔘 Received callback from chat_id: {}", chat_id);

                                // Answer callback query immediately
                                let _ = self.answer_callback(&callback.id, "").await;

                                if let Err(e) = self.handle_callback(&data, &chat_id).await {
                                    eprintln!("❌ Error handling callback: {}", e);
                                }
                            }
                        }

                        offset = update.update_id + 1;
                    }
                }
                Err(e) => {
                    eprintln!("Error getting updates: {}", e);
                    // Add a longer delay on error to avoid spamming
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn delete_webhook(&self) -> Result<(), String> {
        let url = format!(
            "https://api.telegram.org/bot{}/deleteWebhook",
            self.bot_token
        );

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(format!("Failed to delete webhook: {}", text))
        }
    }

    async fn get_updates(&self, offset: i64) -> Result<Vec<Update>, String> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30",
            self.bot_token, offset
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        // Get response text first for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to get response text: {}", e))?;

        // Try to parse as JSON
        let updates: UpdatesResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("JSON parse error: {}. Response: {}", e, response_text))?;

        Ok(updates.result)
    }

    async fn handle_start_command(&self, token: &str, chat_id: &str) -> Result<(), String> {
        // Get QR session from Upstash
        let key = format!("qr:{}", token);
        let session: Option<QRSession> = self.upstash.get_json(&key).await?;

        let mut session = match session {
            Some(s) => s,
            None => {
                self.send_message(chat_id, "❌ QR code expired or invalid.")
                    .await?;
                return Ok(());
            }
        };

        // Check if session is still valid
        let now = chrono::Utc::now().timestamp();
        if now > session.expires_at {
            self.send_message(chat_id, "❌ QR code has expired.")
                .await?;
            return Ok(());
        }

        // Check if user exists in Upstash (using hash as key)
        let chat_id_hash = hash_chat_id(chat_id);
        let user_key = format!("user:{}", chat_id_hash);
        let user: Option<User> = self.upstash.get_json(&user_key).await?;

        let user = match user {
            Some(u) => u,
            None => {
                self.send_message(
                    chat_id,
                    "❌ No account found for your Telegram.\nPlease contact admin to create an account.",
                ).await?;
                return Ok(());
            }
        };

        // Decrypt and verify chat_id matches
        let decrypted_chat_id = decrypt_data(&user.chat_id)?;
        if decrypted_chat_id != chat_id {
            self.send_message(chat_id, "❌ Account verification failed.")
                .await?;
            return Ok(());
        }

        // Check if account is expired
        if now > user.expires_at {
            self.send_message(
                chat_id,
                "❌ Your account has expired.\nPlease contact admin to renew.",
            )
            .await?;
            return Ok(());
        }

        // Update session to pending and attach user info
        session.chat_id = Some(chat_id.to_string());
        session.status = AuthStatus::Pending;

        // Save updated session
        let ttl = (session.expires_at - now) as u64;
        self.upstash.set_json(&key, &session, ttl).await?;

        // Send approval buttons
        let message = format!(
            "🔐 *Login Request*\n\n\
            ⚠️ Do you approve this login?\n\n\
            ⏱️ Expires in {} seconds",
            ttl
        );

        self.send_message_with_buttons(
            chat_id,
            &message,
            vec![vec![
                InlineButton {
                    text: "✅ Approve".to_string(),
                    callback_data: format!("approve:{}", token),
                },
                InlineButton {
                    text: "❌ Decline".to_string(),
                    callback_data: format!("decline:{}", token),
                },
            ]],
        )
        .await?;

        Ok(())
    }

    async fn handle_callback(&self, data: &str, chat_id: &str) -> Result<(), String> {
        // Parse callback data: "approve:token", "decline:token", "approve_update:witness", "reject_update:witness"
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid callback data".to_string());
        }

        let action = parts[0];
        let identifier = parts[1];

        // Handle update approvals
        if action == "approve_update" || action == "reject_update" {
            return self
                .handle_update_approval(action, identifier, chat_id)
                .await;
        }

        // Handle auth approvals (existing logic)
        let token = identifier;
        let key = format!("qr:{}", token);
        let session: Option<QRSession> = self.upstash.get_json(&key).await?;

        let mut session = match session {
            Some(s) => s,
            None => {
                self.send_message(chat_id, "❌ Session expired or invalid.")
                    .await?;
                return Ok(());
            }
        };

        // Verify chat_id matches
        if session.chat_id.as_ref() != Some(&chat_id.to_string()) {
            self.send_message(chat_id, "❌ Unauthorized.").await?;
            return Ok(());
        }

        // Check if already processed
        if session.status == AuthStatus::Approved || session.status == AuthStatus::Denied {
            self.send_message(chat_id, "✅ Already processed.").await?;
            return Ok(());
        }

        let now = chrono::Utc::now().timestamp();
        let ttl = (session.expires_at - now) as u64;

        if action == "approve" {
            session.status = AuthStatus::Approved;
            self.upstash.set_json(&key, &session, ttl).await?;
            self.send_message(chat_id, "✅ Login approved! You can now close this window.")
                .await?;
        } else if action == "decline" {
            session.status = AuthStatus::Denied;
            self.upstash.set_json(&key, &session, ttl).await?;
            self.send_message(chat_id, "❌ Login declined.").await?;
        }

        Ok(())
    }

    async fn handle_update_approval(
        &self,
        action: &str,
        witness: &str,
        chat_id: &str,
    ) -> Result<(), String> {
        use crate::updater::telegram::ApprovalStatus;

        let approval_key = format!("update_approval:{}", witness);
        let approval: Option<ApprovalStatus> = self.upstash.get_json(&approval_key).await?;

        let approval = match approval {
            Some(a) => a,
            None => {
                self.send_message(chat_id, "❌ Update approval request not found or expired.")
                    .await?;
                return Ok(());
            }
        };

        // Verify authorized user (you can add user ID check here)
        // For now, we'll allow any authenticated user

        let approved = action == "approve_update";
        let new_status = ApprovalStatus {
            witness: approval.witness.clone(),
            approved,
            timestamp: chrono::Utc::now().timestamp(),
            version: approval.version.clone(),
        };

        // Store approval status (1 hour TTL)
        self.upstash
            .set_json(&approval_key, &new_status, 3600)
            .await?;

        if approved {
            self.send_message(
                chat_id,
                &format!(
                    "✅ Update {} approved! The app will download and install it.",
                    approval.version
                ),
            )
            .await?;
        } else {
            self.send_message(
                chat_id,
                &format!("❌ Update {} rejected.", approval.version),
            )
            .await?;
        }

        Ok(())
    }

    async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        #[derive(Serialize)]
        struct SendMessageRequest {
            chat_id: String,
            text: String,
        }

        self.client
            .post(&url)
            .json(&SendMessageRequest {
                chat_id: chat_id.to_string(),
                text: text.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn send_message_with_buttons(
        &self,
        chat_id: &str,
        text: &str,
        buttons: Vec<Vec<InlineButton>>,
    ) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        #[derive(Serialize)]
        struct SendMessageWithButtonsRequest {
            chat_id: String,
            text: String,
            parse_mode: String,
            reply_markup: InlineKeyboard,
        }

        #[derive(Serialize)]
        struct InlineKeyboard {
            inline_keyboard: Vec<Vec<InlineButton>>,
        }

        let request = SendMessageWithButtonsRequest {
            chat_id: chat_id.to_string(),
            text: text.to_string(),
            parse_mode: "Markdown".to_string(),
            reply_markup: InlineKeyboard {
                inline_keyboard: buttons,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to send message: {}", response.status()))
        }
    }

    async fn answer_callback(&self, callback_id: &str, text: &str) -> Result<(), String> {
        let url = format!(
            "https://api.telegram.org/bot{}/answerCallbackQuery",
            self.bot_token
        );

        #[derive(Serialize)]
        struct AnswerCallbackRequest {
            callback_query_id: String,
            text: String,
        }

        let request = AnswerCallbackRequest {
            callback_query_id: callback_id.to_string(),
            text: text.to_string(),
        };

        let _ = self.client.post(&url).json(&request).send().await;
        Ok(())
    }
}
