# Pending Features - Implementation Status

This document tracks features that are planned but not yet fully implemented.

---

## 1. WMI Event Subscription Persistence ⚠️ **PENDING**

**Status:** Placeholder function exists, full implementation pending

**Location:** `stub/src/persistence.rs` - `persist_wmi_event()`

**Why Pending:**
- Requires complex COM (Component Object Model) implementation
- Uses `IWbemServices` interface
- Needs event filter and consumer creation
- Other persistence methods (Registry, Scheduled Task, Startup Folder) are fully functional

**Planned Implementation:**
```rust
// 1. Initialize COM
// 2. Connect to WMI namespace
// 3. Create event filter (e.g., process creation events)
// 4. Create event consumer (executes payload)
// 5. Bind filter to consumer
```

**Alternative:** Use existing persistence methods (Registry, Scheduled Task, Startup Folder)

---

## 2. UAC Bypass ✅ **COMPLETED**

**Status:** Fully implemented

**Location:** `stub/src/uac_bypass.rs`

**Implementation:**
- ✅ FodHelper.exe method
- ✅ ComputerDefaults method
- ✅ SilentCleanup method
- ✅ Automatic admin check (skips if already admin)
- ✅ Registry cleanup after execution

**Note:** UAC bypass is optional and only needed for privilege escalation. Fully functional and integrated into stub generation.

---

## 3. Process Doppelganging ⚠️ **PENDING**

**Status:** Placeholder module created, not yet implemented

**Location:** `stub/src/process_doppelganging.rs`

**Why Pending:**
- Requires Windows Transaction API (TxF - Transactional File System)
- Complex implementation
- Other injection methods (process hollowing, early bird) provide similar functionality
- TxF may not be available on all Windows versions

**Planned Implementation:**
```rust
// 1. Create transaction using CreateTransaction
// 2. Write payload file within transaction
// 3. Create process from transaction file
// 4. Rollback transaction (file never exists on disk)
// 5. Process continues running even though file doesn't exist
```

**Alternative:** Use `process_hollowing.rs` or `early_bird.rs` for similar stealth

---

## 4. Certificate Pinning Bypass ⚠️ **PENDING**

**Status:** Placeholder module created, not yet implemented

**Location:** `stub/src/cert_pinning_bypass.rs`

**Why Pending:**
- Only needed if payload communicates with C2 server
- C2 server must use certificate pinning
- Requires hooking SSL/TLS functions

**Planned Implementation:**
1. Hook OpenSSL functions:
   - `SSL_connect` - bypass certificate validation
   - `SSL_write` - intercept outgoing data
   - `SSL_read` - intercept incoming data

2. Hook Schannel (Windows native SSL):
   - `CertVerifyCertificateChainPolicy` - always return success

**When to Implement:** Only if payload has C2 communication with certificate pinning

---

## Summary

All pending features have:
- ✅ Placeholder files/modules created
- ✅ Documentation of planned implementation
- ✅ Clear notes on why they're pending
- ✅ Alternative solutions documented
- ✅ Marked with `#[allow(dead_code)]` to prevent warnings

**Note:** These features are optional or have alternatives. The crypter is fully functional without them.

---

## Recently Completed (Moved from Pending)

### ✅ UAC Bypass - COMPLETED
- **Location:** `stub/src/uac_bypass.rs`
- **Status:** Fully implemented with all three methods (FodHelper, ComputerDefaults, SilentCleanup)
- **Integration:** Added to stub generation, runs before payload execution
- **Features:**
  - Automatic admin check (skips if already admin)
  - Registry cleanup after execution
  - Three bypass methods with fallback

### ✅ WDAC Bypass (CVE-2025-26678) - COMPLETED
- **Location:** `stub/src/wdac_bypass.rs`
- **Status:** Fully implemented with vulnerability detection and bypass attempt
- **Integration:** Added to stub generation, runs early (before AMSI/ETW bypass)
- **CVE Reference:** [CVE-2025-26678](https://nvd.nist.gov/vuln/detail/CVE-2025-26678)
- **Features:**
  - Checks if WDAC is enabled
  - Detects vulnerability (CVE-2025-26678)
  - Attempts registry-based bypass
  - Graceful failure on patched systems
- **References:**
  - [NVD CVE-2025-26678](https://nvd.nist.gov/vuln/detail/CVE-2025-26678)
  - [Recorded Future CVE-2025-26678](https://www.recordedfuture.com/vulnerability-database/CVE-2025-26678)
- **Note:** Only works on unpatched systems. Microsoft has released patches.
