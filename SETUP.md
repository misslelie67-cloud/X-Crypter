# X-Crypter QR Login System

## Setup

### 1. Environment Variables

Copy `.env.example` to `.env` and fill in your credentials:

```bash
cp .env.example .env
```

### 2. User Management

Users must be added to Upstash Redis before they can login. Each user entry should be stored with their Telegram chat_id as the key.

**User Data Structure:**

```json
{
  "chat_id": "123456789",
  "key_hash": "user_encryption_key_hash",
  "expires_at": 1735689600,
  "created_at": 1704067200
}
```

**To add a user to Upstash:**

```bash
# Using Upstash REST API
curl -X POST https://your-redis-url.upstash.io/set/user:CHAT_ID \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"chat_id":"123456789","key_hash":"hash","expires_at":1735689600,"created_at":1704067200}'
```

## How It Works

1. **User opens app** → Loading screen (5 seconds)
2. **Auth screen appears** → QR code is generated
3. **User scans QR with Telegram** → Opens bot with `/start <token>`
4. **Bot checks:**
   - Is chat_id in database?
   - Is account expired?
5. **If valid** → Automatically approves login
6. **Frontend detects approval** → Exchanges token for session → User logged in

## File Structure

```
src-tauri/src/
├── auth/           # Authentication logic
│   ├── mod.rs
│   ├── types.rs    # Auth data structures
│   └── commands.rs # Tauri commands (generate_qr_code, check_auth_status, exchange_token)
├── storage/        # Data persistence
│   ├── mod.rs
│   └── upstash.rs  # Upstash Redis client
├── telegram/       # Telegram bot integration
│   ├── mod.rs
│   └── bot.rs      # Bot polling and /start command handler
└── lib.rs          # Main entry point
```

## Testing

1. Create a `.env` file with your credentials
2. Add your Telegram chat_id to Upstash (see User Management above)
3. Run the app: `npm run tauri dev`
4. Scan the QR code with your Telegram
5. Bot will automatically approve if your account is valid
6. You'll be logged in!

## Environment Variables Needed

- `TELEGRAM_BOT_TOKEN` - Your Telegram bot token from @BotFather
- `TELEGRAM_BOT_USERNAME` - Your bot's username (without @)
- `UPSTASH_REDIS_REST_URL` - Your Upstash Redis REST URL
- `UPSTASH_REDIS_REST_TOKEN` - Your Upstash Redis REST token
