# Cross-Compilation Setup Guide

## Building Windows Version from macOS

### ✅ What CAN be cross-compiled:
- **Main Tauri app** - Can be built for Windows from macOS (experimental)

### ❌ What CANNOT be cross-compiled:
- **Stub project** - Requires Windows to compile (Windows-only APIs, PE executables)

---

## Setup Instructions for macOS → Windows

### 1. Install Rust Windows Target

```bash
# Add Windows MSVC target
rustup target add x86_64-pc-windows-msvc

# For ARM64 Windows (optional)
rustup target add aarch64-pc-windows-msvc
```

### 2. Install Cross-Compilation Tools

```bash
# Install cargo-xwin (manages Windows SDK)
cargo install cargo-xwin

# Install LLVM tools (for linking and resource compilation)
brew install llvm

# Install NSIS (for Windows installer)
brew install nsis
```

### 3. Configure Tauri for Cross-Compilation

Update `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis"],  // Use NSIS instead of "all" (MSI requires Windows)
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### 4. Build Commands

```bash
# Build for Windows from macOS
npm run tauri build -- --target x86_64-pc-windows-msvc

# Or using cargo-xwin explicitly
npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc
```

### 5. Output Location

Windows builds will be in:
```
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/
```

---

## Limitations

1. **MSI Installer**: Cannot be built from macOS (requires Windows)
2. **Code Signing**: Limited support for cross-compiled binaries
3. **Stub Compilation**: Cannot be done from macOS (requires Windows)
4. **Experimental**: Cross-compilation is experimental and may have issues

---

## Alternative: Use Windows VM or CI/CD

### Option 1: Windows VM (Recommended)
- Use Parallels, VMware, or VirtualBox
- Full Windows development environment
- Can build everything including stub

### Option 2: GitHub Actions / CI/CD
- Set up Windows runner
- Build on push/PR
- Most reliable for production builds

### Option 3: Dual Boot / Boot Camp
- Native Windows performance
- Best for development and testing

---

## For Stub Compilation

The stub project **must** be compiled on Windows because:
- Uses Windows-only APIs (`winapi` crate)
- Generates Windows PE executables
- Requires Windows linker and tools

**Workflow:**
1. Develop on macOS (main app)
2. Copy source to Windows VM/machine
3. Compile stub on Windows
4. Use stub in main app

---

## Recommended Setup

For best results:
- **Development**: macOS (main app development)
- **Stub Compilation**: Windows VM (when needed)
- **Final Builds**: Windows VM or CI/CD (for production)
