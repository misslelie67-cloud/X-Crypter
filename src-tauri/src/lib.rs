mod auth;
mod crypter;
mod crypto;
mod scanner;
mod storage;
mod telegram;
mod updater;

use auth::{
    check_auth_status, create_user_account, exchange_token, generate_qr_code, validate_session,
};
use crypter::encrypt_exe;
use scanner::commands::{scan_file, scan_with_clamav, scan_with_windows_defender};
use std::sync::Arc;
use storage::UpstashClient;
use telegram::TelegramBot;
use updater::commands::{
    check_for_updates, check_update_approval, get_app_version, get_stored_version,
    get_update_check_interval, install_update, restart_application,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables from project root
    // Try multiple locations to find .env file
    let mut env_loaded = false;

    // Try current directory
    if let Ok(current_dir) = std::env::current_dir() {
        let env_path = current_dir.join(".env");
        if env_path.exists() {
            if let Ok(_) = dotenvy::from_path(&env_path) {
                env_loaded = true;
                eprintln!(" Loaded .env from: {}", env_path.display());
            }
        }

        // Try parent directory (project root)
        if !env_loaded {
            if let Some(parent) = current_dir.parent() {
                let parent_env = parent.join(".env");
                if parent_env.exists() {
                    if let Ok(_) = dotenvy::from_path(&parent_env) {
                        env_loaded = true;
                        eprintln!(" Loaded .env from: {}", parent_env.display());
                    }
                }
            }
        }

        // Try src-tauri directory
        if !env_loaded {
            let tauri_env = current_dir.join("src-tauri").join(".env");
            if tauri_env.exists() {
                if let Ok(_) = dotenvy::from_path(&tauri_env) {
                    env_loaded = true;
                    eprintln!(" Loaded .env from: {}", tauri_env.display());
                }
            }
        }
    }

    // Also try loading from project root relative to executable
    if !env_loaded {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Try multiple parent levels (for different build configurations)
                let mut current = exe_dir.to_path_buf();
                for _ in 0..5 {
                    let env_path = current.join(".env");
                    if env_path.exists() {
                        if let Ok(_) = dotenvy::from_path(&env_path) {
                            env_loaded = true;
                            eprintln!(" Loaded .env from: {}", env_path.display());
                            break;
                        }
                    }
                    // Also check parent directories
                    if let Some(parent) = current.parent() {
                        let parent_env = parent.join(".env");
                        if parent_env.exists() {
                            if let Ok(_) = dotenvy::from_path(&parent_env) {
                                env_loaded = true;
                                eprintln!(" Loaded .env from: {}", parent_env.display());
                                break;
                            }
                        }
                        current = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    // Fallback: try dotenvy::dotenv() which looks in current and parent directories
    if !env_loaded {
        if let Ok(_) = dotenvy::dotenv() {
            env_loaded = true;
            eprintln!(" Loaded .env using dotenvy::dotenv()");
        }
    }

    // Also try loading from CARGO_MANIFEST_DIR if available (set by Cargo)
    if !env_loaded {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let manifest_path = std::path::Path::new(&manifest_dir);
            // Go up from src-tauri to project root
            if let Some(parent) = manifest_path.parent() {
                let env_path = parent.join(".env");
                if env_path.exists() {
                    if let Ok(_) = dotenvy::from_path(&env_path) {
                        env_loaded = true;
                        eprintln!(
                            " Loaded .env from CARGO_MANIFEST_DIR parent: {}",
                            env_path.display()
                        );
                    }
                }
            }
        }
    }

    // Debug: Print environment variable status
    if !env_loaded {
        eprintln!("⚠️ Warning: Could not find .env file");
        eprintln!("   The app will look for .env in:");
        eprintln!("   - Current directory");
        eprintln!("   - Parent directory (project root)");
        eprintln!("   - src-tauri directory");
        eprintln!("   - Executable directory and parent directories");
        eprintln!("   - CARGO_MANIFEST_DIR parent directory");
    }

    match std::env::var("WITNESS_SECRET") {
        Ok(val) => eprintln!("✓ WITNESS_SECRET is set (length: {})", val.len()),
        Err(_) => eprintln!("⚠️ WITNESS_SECRET is NOT set - check .env file location"),
    }

    match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(_) => eprintln!("✓ TELEGRAM_BOT_TOKEN is set"),
        Err(_) => eprintln!("⚠️ TELEGRAM_BOT_TOKEN is NOT set"),
    }

    match std::env::var("UPSTASH_REDIS_REST_URL") {
        Ok(_) => eprintln!("✓ UPSTASH_REDIS_REST_URL is set"),
        Err(_) => eprintln!("⚠️ UPSTASH_REDIS_REST_URL is NOT set"),
    }

    // Initialize Upstash client
    let upstash = match UpstashClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Failed to initialize Upstash client: {}", e);
            eprintln!("   Make sure UPSTASH_REDIS_REST_URL and UPSTASH_REDIS_REST_TOKEN are set in your .env file");
            eprintln!("   .env file should be in the project root directory");
            std::process::exit(1);
        }
    };

    // Initialize Telegram bot
    let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("❌ TELEGRAM_BOT_TOKEN not set");
            eprintln!("   Please set TELEGRAM_BOT_TOKEN in your .env file");
            eprintln!("   .env file should be in the project root directory");
            std::process::exit(1);
        }
    };
    let bot = Arc::new(TelegramBot::new(bot_token, upstash.clone()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(upstash)
        .setup(move |_app| {
            // Check for pending updates on startup
            if let Err(e) = updater::install::apply_pending_update() {
                eprintln!("⚠️ Failed to apply pending update: {}", e);
            }

            // Start bot polling in background using Tauri's async runtime
            let bot_clone = Arc::clone(&bot);
            tauri::async_runtime::spawn(async move {
                bot_clone.start_polling().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_qr_code,
            check_auth_status,
            exchange_token,
            validate_session,
            create_user_account,
            encrypt_exe,
            crypter::save_uploaded_file,
            crypter::save_uploaded_icon,
            crypter::validate_file_for_encryption,
            check_for_updates,
            check_update_approval,
            install_update,
            get_app_version,
            restart_application,
            get_update_check_interval,
            get_stored_version,
            scan_file,
            scan_with_clamav,
            scan_with_windows_defender,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
