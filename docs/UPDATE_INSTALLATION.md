# Update Installation Implementation

## Overview

The update installation system handles:
1. **ZIP Extraction** - Extracts downloaded update ZIP files
2. **File Replacement** - Replaces crypter module source files
3. **Restart Mechanism** - Restarts the app to apply changes
4. **Pending Updates** - Handles updates that can't be applied immediately

## Architecture

### Immediate Installation Flow

```
1. Download Update ZIP
   ↓
2. Extract to Staging Directory
   ↓
3. Try to Replace Crypter Modules
   ├─ Success → Clean up → Restart
   └─ Failure (files locked) → Mark as Pending
```

### Pending Update Flow (On Startup)

```
1. App Starts
   ↓
2. Check for Pending Update
   ├─ Found → Extract ZIP → Replace Files → Clear Pending
   └─ Not Found → Continue Normal Startup
```

## Implementation Details

### ZIP Extraction

- Extracts to `{temp_dir}/x-crypter-updates/staging/`
- Preserves directory structure
- Handles nested directories

### File Replacement

Replaces these crypter module files:
- `mod.rs`
- `encryptor.rs`
- `pe_reader.rs`
- `stub_gen.rs`
- `mutator.rs`
- `key_manager.rs`
- `code_signing.rs`
- `resource_manipulation.rs`
- `string_obfuscator.rs`
- `obfuscator.rs`

**Backup System:**
- Creates backup in `src-tauri/src/crypter/backup/`
- Backs up before replacing
- Allows rollback if needed

### Project Root Detection

The system uses multiple strategies to find the project root:

1. **Current Directory**: Looks for `Cargo.toml` and `src-tauri/`
2. **Executable Path**: Traverses up from executable location
3. **Parent Directories**: Checks parent directories recursively

### Restart Mechanism

**Windows:**
```rust
cmd /C start "" {exe_path}
```

**Unix-like (macOS/Linux):**
```rust
spawn({exe_path})
```

Both approaches:
- Start new instance
- Wait 500ms
- Exit current instance

## Update Package Structure

Your update ZIP should have this structure:

```
update_0.2.0.zip
└── src-tauri/
    └── src/
        └── crypter/
            ├── mod.rs
            ├── encryptor.rs
            ├── pe_reader.rs
            ├── stub_gen.rs
            ├── mutator.rs
            ├── key_manager.rs
            ├── code_signing.rs
            ├── resource_manipulation.rs
            ├── string_obfuscator.rs
            └── obfuscator.rs
```

## Usage

### Creating Update Package

```bash
# From project root
cd src-tauri/src/crypter
zip -r ../../../update_0.2.0.zip .
cd ../../..
# Or create with proper structure:
mkdir -p update_package/src-tauri/src/crypter
cp src-tauri/src/crypter/*.rs update_package/src-tauri/src/crypter/
cd update_package
zip -r ../update_0.2.0.zip .
```

### Update Server Requirements

Your update server's `/api/updates/latest` endpoint should return:

```json
{
  "version": "0.2.0",
  "file_hash": "sha256-hash-of-zip-file",
  "file_url": "https://your-server.com/updates/update_0.2.0.zip",
  "timestamp": 1234567890,
  "witness": "hmac-sha256-witness",
  "changelog": "Optional changelog",
  "size": 1024000
}
```

## Error Handling

### Files Locked

If files can't be replaced (locked by running process):
- Update is marked as pending
- ZIP file is kept
- Update applied on next app start

### Update File Missing

If update ZIP is missing on startup:
- Pending update is cleared
- App continues normally

### Backup and Rollback

- All files are backed up before replacement
- Backups stored in `src-tauri/src/crypter/backup/`
- Manual rollback: Copy files from backup directory

## Security

1. **Witness Verification**: Required before installation
2. **File Hash Verification**: SHA256 verified after download
3. **Approval Required**: Manual approval via Telegram
4. **Backup System**: Allows rollback if update fails

## Limitations

1. **Source File Updates Only**: Updates source files, requires rebuild for changes to take effect
2. **Runtime Updates**: New stub generation code is used immediately after restart
3. **No Binary Updates**: Doesn't replace compiled binaries (would require full app update)

## Future Enhancements

- [ ] Automatic rebuild after file replacement
- [ ] Rollback mechanism UI
- [ ] Update verification before replacement
- [ ] Incremental updates (only changed files)
- [ ] Update history tracking
