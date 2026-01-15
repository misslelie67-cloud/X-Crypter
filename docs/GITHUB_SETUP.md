# GitHub Setup Guide - Step by Step

This guide walks you through setting up GitHub Actions for your X-Crypter project.

---

## 📋 Prerequisites

1. **GitHub Account** - Free account works fine
2. **Git Repository** - Your code pushed to GitHub
3. **Basic Git Knowledge** - How to push, tag, etc.

---

## 🚀 Step-by-Step Setup

### Step 1: Push Your Code to GitHub

If you haven't already:

```bash
# Initialize git (if not already done)
git init

# Add all files
git add .

# Create initial commit
git commit -m "Initial commit"

# Add GitHub remote (replace with your repo URL)
git remote add origin https://github.com/YOUR_USERNAME/X-Crypter.git

# Push to GitHub
git push -u origin main
```

**Or** if you already have a GitHub repo:
```bash
git remote add origin https://github.com/YOUR_USERNAME/X-Crypter.git
git push -u origin main
```

---

### Step 2: Verify Workflow Files Are Pushed

Make sure these files are in your repository:

```
.github/
  └── workflows/
      ├── ci.yml          # Continuous Integration
      └── release.yml     # Release builds

rust-toolchain.toml       # Rust version pinning
```

Check on GitHub:
1. Go to your repository
2. Click on `.github/workflows/` folder
3. Verify both `ci.yml` and `release.yml` are there

---

### Step 3: Enable GitHub Actions

GitHub Actions is enabled by default, but verify:

1. Go to your repository on GitHub
2. Click **Settings** tab (top menu)
3. Click **Actions** in left sidebar
4. Under **Actions permissions**:
   - Select **"Allow all actions and reusable workflows"**
   - Click **Save**

---

### Step 4: Configure Secrets (Optional but Recommended)

If you need to sign code or use private dependencies:

1. Go to repository **Settings**
2. Click **Secrets and variables** → **Actions**
3. Click **New repository secret**

**Common secrets you might need:**

#### For Code Signing (Windows):
- `WINDOWS_CERTIFICATE_PFX` - Your code signing certificate (base64 encoded)
- `WINDOWS_CERTIFICATE_PASSWORD` - Certificate password

#### For Private Dependencies:
- `CARGO_REGISTRY_TOKEN` - Cargo registry token (if using private crates)

#### For Releases:
- `GITHUB_TOKEN` - Automatically provided, no action needed

**Note:** For most projects, you don't need any secrets - GitHub provides `GITHUB_TOKEN` automatically.

---

### Step 5: Test the CI Workflow

#### Option A: Push a Change
```bash
# Make a small change
echo "# Test" >> README.md

# Commit and push
git add README.md
git commit -m "Test CI workflow"
git push origin main
```

#### Option B: Create a Pull Request
1. Create a new branch:
   ```bash
   git checkout -b test-ci
   git push -u origin test-ci
   ```
2. On GitHub, create a Pull Request
3. CI will run automatically

#### Check Results:
1. Go to your repository on GitHub
2. Click **Actions** tab (top menu)
3. You should see workflows running
4. Click on a workflow run to see details

---

### Step 6: Verify Workflow Runs Successfully

In the **Actions** tab:

1. **Green checkmark** ✅ = Success
2. **Red X** ❌ = Failure (click to see errors)
3. **Yellow circle** ⏳ = Running

**What to check:**
- ✅ Lint job passes
- ✅ Test jobs pass (Ubuntu, macOS, Windows)
- ✅ Build jobs complete
- ✅ Stub builds on Windows

---

### Step 7: Create Your First Release

Once CI is working, create a release:

#### Method 1: Using Git Tags (Recommended)
```bash
# Update version in Cargo.toml first
# Then create and push tag:
git tag v1.0.0
git push origin v1.0.0
```

#### Method 2: Using GitHub UI
1. Go to **Releases** → **Draft a new release**
2. Choose tag: `v1.0.0` (create if doesn't exist)
3. Release title: `X-Crypter v1.0.0`
4. Click **Publish release**
5. GitHub Actions will build and attach installers automatically

**What happens:**
- Release workflow triggers automatically
- Builds for all platforms (Windows, macOS, Linux)
- Creates GitHub release with installers
- Uploads artifacts

---

## 🔧 GitHub Repository Settings

### Recommended Settings:

1. **Branch Protection** (Optional but recommended):
   - Go to **Settings** → **Branches**
   - Add rule for `main` branch
   - Require status checks to pass
   - Require branches to be up to date

2. **Actions Permissions**:
   - **Settings** → **Actions** → **General**
   - **Workflow permissions**: "Read and write permissions"
   - **Allow GitHub Actions to create and approve pull requests**: ✅

3. **Notifications** (Optional):
   - **Settings** → **Notifications**
   - Enable email notifications for workflow failures

---

## 📊 Understanding GitHub Actions UI

### Actions Tab Overview:

```
Actions Tab
├── All workflows
│   ├── CI (runs on push/PR)
│   └── Release (runs on tags)
│
└── Workflow runs
    ├── Status (✅/❌/⏳)
    ├── Commit message
    ├── Branch/tag
    └── Duration
```

### Workflow Run Details:

Click on a workflow run to see:
- **Jobs**: List of all jobs (lint, test, build, etc.)
- **Logs**: Detailed build logs
- **Artifacts**: Built binaries (if any)
- **Annotations**: Warnings and errors

---

## 🐛 Troubleshooting

### Workflow Not Running?

1. **Check file location:**
   - Must be in `.github/workflows/`
   - File must be `.yml` or `.yaml`
   - Check file is committed and pushed

2. **Check Actions are enabled:**
   - Settings → Actions → General
   - Ensure "Allow all actions" is selected

3. **Check branch:**
   - Workflows run on `main`/`master` by default
   - Or on PRs targeting these branches

### Build Failures?

1. **Check logs:**
   - Click failed job
   - Scroll through logs
   - Look for error messages

2. **Common issues:**
   - Missing dependencies → Check `Cargo.toml`
   - Rust version mismatch → Check `rust-toolchain.toml`
   - Missing environment variables → Check `.env` requirements

3. **Test locally first:**
   ```bash
   # Run what CI runs
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features
   cargo test
   ```

### Stub Build Failing?

- Stub only builds on Windows runner
- Check Windows-specific dependencies
- Verify `stub/Cargo.toml` is correct

---

## 📝 Workflow File Locations

Make sure these files exist in your repo:

```
X-Crypter/
├── .github/
│   └── workflows/
│       ├── ci.yml          ← CI workflow
│       └── release.yml     ← Release workflow
├── rust-toolchain.toml     ← Rust version
├── src-tauri/
│   └── Cargo.toml
├── stub/
│   └── Cargo.toml
└── package.json
```

---

## 🎯 Quick Start Checklist

- [ ] Code pushed to GitHub
- [ ] `.github/workflows/` folder exists with both workflow files
- [ ] `rust-toolchain.toml` exists
- [ ] GitHub Actions enabled in Settings
- [ ] Push a commit to trigger CI
- [ ] Verify CI runs successfully
- [ ] Create a test release tag

---

## 🔗 Useful GitHub Links

Once set up, you'll frequently use:

- **Actions**: `https://github.com/YOUR_USERNAME/X-Crypter/actions`
- **Releases**: `https://github.com/YOUR_USERNAME/X-Crypter/releases`
- **Settings**: `https://github.com/YOUR_USERNAME/X-Crypter/settings`

---

## 💡 Pro Tips

1. **Monitor First Few Builds:**
   - Watch the Actions tab during first builds
   - Check for any warnings or errors
   - Adjust workflows if needed

2. **Use Branch Protection:**
   - Require CI to pass before merging
   - Prevents broken code from reaching main

3. **Check Build Times:**
   - First build: ~10-20 minutes (no cache)
   - Subsequent builds: ~5-10 minutes (with cache)

4. **Artifacts:**
   - Built binaries are available as artifacts
   - Download from workflow run page
   - Kept for 90 days (configurable)

5. **Notifications:**
   - Get email when builds fail
   - Stay informed about CI status

---

## 🎉 You're Done!

Once set up, your workflow will be:
1. Write code locally
2. Push to GitHub
3. CI automatically tests and builds
4. Create release tag when ready
5. GitHub automatically builds and releases

No more manual builds or Windows VMs needed! 🚀
