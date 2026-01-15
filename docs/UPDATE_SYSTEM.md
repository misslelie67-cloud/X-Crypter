# Update System Documentation

## Overview

The X-Crypter app supports remote updates with witness-based approval via Telegram. Updates are stored in **Upstash Storage** (100GB) and metadata is managed via **Upstash Redis**. This allows you to:

1. Test new stub generation techniques locally
2. Upload update files to Upstash Storage
3. Store update metadata in Upstash Redis
4. App checks for updates and sends approval request to Telegram
5. You approve/reject via Telegram
6. App downloads and installs approved updates

## Architecture

```
┌─────────────────────────────────────┐
│   Your Local Machine                │
│   - Develop new techniques          │
│   - Test locally                    │
│   - Upload to Upstash Storage       │
│   - Store metadata in Upstash Redis │
└──────────────┬──────────────────────┘
               │
               │ Upload
               │
┌──────────────▼──────────────────────┐
│   Upstash Storage (100GB)           │
│   - Stores update ZIP files         │
│   - Provides download URLs           │
└──────────────┬──────────────────────┘
               │
               │
┌──────────────▼──────────────────────┐
│   Upstash Redis                      │
│   - Stores update metadata           │
│   - Stores approval status           │
│   - Key: "update:latest"            │
└──────────────┬──────────────────────┘
               │
               │ HTTPS
               │
┌──────────────▼──────────────────────┐
│   Tauri App (User's Machine)        │
│   - Checks for updates (Upstash)    │
│   - Sends beacon to Telegram        │
│   - Waits for approval              │
│   - Downloads from Upstash Storage  │
└──────────────┬──────────────────────┘
               │
               │ Telegram API
               │
┌──────────────▼──────────────────────┐
│   Telegram Bot                       │
│   - Receives approval request        │
│   - Sends notification to you       │
│   - Handles approve/reject buttons  │
│   - Stores approval in Upstash      │
└──────────────┬──────────────────────┘
               │
               │ Your Approval
               │
┌──────────────▼──────────────────────┐
│   You (Telegram)                     │
│   - Review update                    │
│   - Approve/reject                   │
└─────────────────────────────────────┘
```

## Environment Variables

Add these to your `.env` file:

```env
# Upstash Configuration (already exists)
UPSTASH_REDIS_REST_URL=https://your-upstash-redis.upstash.io
UPSTASH_REDIS_REST_TOKEN=your-upstash-token

# Update Configuration
WITNESS_SECRET=your-secret-key-for-hmac-generation

# Telegram Configuration (already exists)
TELEGRAM_BOT_TOKEN=your-bot-token
TELEGRAM_CHAT_ID=your-chat-id

# Optional: Update Check Interval (default: 86400 = 24 hours)
UPDATE_CHECK_INTERVAL=86400

# Optional: External Update Server (fallback if Upstash not available)
# UPDATE_SERVER_URL=https://your-update-server.com
```

## Update Metadata Format

Update metadata is stored in Upstash Redis under the key `update:latest`:

```json
{
  "version": "0.2.0",
  "file_hash": "sha256-hash-of-update-file",
  "file_url": "https://your-upstash-storage.com/updates/update_0.2.0.zip",
  "timestamp": 1234567890,
  "witness": "hmac-sha256-witness",
  "changelog": "Optional changelog text",
  "size": 1024000
}
```

The `file_url` should point to your update file in Upstash Storage (or any publicly accessible URL).

### Witness Generation (Server-Side)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_witness(version: &str, file_hash: &str, timestamp: i64, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    let message = format!("{}:{}:{}", version, file_hash, timestamp);
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

## Workflow

### 1. Local Development

```bash
# Make changes to stub generation
# Test locally
cargo build --release

# Create update package
zip -r update_0.2.0.zip src-tauri/src/crypter/
```

### 2. Upload to Upstash Storage

```bash
# Upload update ZIP file to Upstash Storage
# Get the public URL for the file
# Example: https://your-upstash-storage.com/updates/update_0.2.0.zip
```

### 3. Store Update Metadata in Upstash Redis

You need to store the update metadata in Upstash Redis. You can use the Upstash REST API or a script:

```bash
# Using curl to store update metadata
curl -X POST "https://your-upstash-redis.upstash.io/set/update:latest" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "version": "0.2.0",
    "file_hash": "sha256-hash-of-update-file",
    "file_url": "https://your-upstash-storage.com/updates/update_0.2.0.zip",
    "timestamp": 1234567890,
    "witness": "hmac-sha256-witness",
    "changelog": "New features and improvements",
    "size": 1024000
  }'
```

**Note:** The `witness` must be generated using the `WITNESS_SECRET` environment variable. See "Witness Generation" section below.

### 4. App Checks for Updates

- App calls `check_for_updates()` command
- Fetches update info from Upstash Redis (key: `update:latest`)
- Falls back to external server if `UPDATE_SERVER_URL` is configured
- Verifies witness signature
- Sends approval request to Telegram

### 5. Telegram Approval

- You receive notification in Telegram
- Click "✅ Approve" or "❌ Reject"
- Bot stores approval status in Upstash Redis

### 6. App Downloads Update

- App polls for approval status from Upstash Redis
- If approved, downloads update file from Upstash Storage URL
- Verifies file hash
- Installs update (extracts and replaces modules)

## Security Features

1. **Witness Verification**: HMAC-SHA256 prevents tampering
2. **File Hash Verification**: SHA256 ensures file integrity
3. **Approval Required**: Manual approval prevents unauthorized updates
4. **Telegram Integration**: Second channel for security
5. **Time-Limited Approvals**: Witness expires after 1 hour

## Frontend Integration

Add `UpdateStatus` component to your app:

```tsx
import UpdateStatus from "./components/UpdateStatus";

// In your main component
<UpdateStatus />
```

## Tauri Commands

### `check_for_updates()`

Checks for available updates and sends approval request.

**Returns:**
```typescript
{
  status: "waiting_approval" | "up_to_date" | "error",
  update_available: boolean,
  version: string | null,
  changelog: string | null,
  error: string | null
}
```

### `check_update_approval(witness: string)`

Checks if a witness has been approved.

**Returns:** `boolean | null` (true = approved, false = rejected, null = pending)

### `install_update(witness: string)`

Downloads and installs an approved update.

**Returns:** Success message string

### `get_app_version()`

Gets current app version.

**Returns:** Version string (e.g., "0.1.0")

## Implementation Status

- ✅ Update checking
- ✅ Witness generation/verification
- ✅ Telegram approval system
- ✅ Secure download with hash verification
- ✅ Frontend UI component
- ⚠️ Update installation (extract and replace) - TODO
- ⚠️ Automatic update checking on app start - TODO
- ⚠️ Rollback mechanism - TODO

## Uploading Updates to Upstash Storage

To upload an update file to Upstash Storage, you can:

1. **Use Upstash Console**: Upload files through the Upstash web console
2. **Use Upstash Storage API**: Use the REST API to upload files programmatically
3. **Use a script**: Create a deployment script that uploads files and updates metadata

Example script structure:
```bash
#!/bin/bash
# 1. Upload ZIP to Upstash Storage
# 2. Get public URL
# 3. Calculate SHA256 hash
# 4. Generate witness
# 5. Store metadata in Upstash Redis
```

## Next Steps

1. Implement update installation (extract ZIP, replace modules)
2. Add automatic update check on app start
3. Add rollback mechanism
4. Create update upload script for Upstash Storage
5. Add update package creation script
