# GitHub Actions Quick Start Guide

## 🎯 5-Minute Setup

### Step 1: Push Code to GitHub

```bash
# If you haven't created a GitHub repo yet:
# 1. Go to github.com and create a new repository
# 2. Don't initialize with README (you already have code)

# Then in your terminal:
git init
git add .
git commit -m "Initial commit with GitHub Actions"
git branch -M main
git remote add origin https://github.com/YOUR_USERNAME/X-Crypter.git
git push -u origin main
```

**Replace `YOUR_USERNAME` with your GitHub username!**

---

### Step 2: Verify Files Are on GitHub

1. Go to: `https://github.com/YOUR_USERNAME/X-Crypter`
2. Click on `.github` folder
3. Click on `workflows` folder
4. You should see:
   - ✅ `ci.yml`
   - ✅ `release.yml`

If you don't see these files, they weren't pushed. Check your `.gitignore`!

---

### Step 3: Enable GitHub Actions

1. Go to your repository on GitHub
2. Click **Settings** tab (top right, next to "Code")
3. In left sidebar, click **Actions**
4. Under **Actions permissions**:
   - Select: **"Allow all actions and reusable workflows"**
   - Click **Save** button

**That's it! Actions are now enabled.**

---

### Step 4: Trigger Your First Build

#### Option A: Push a Small Change
```bash
# Make a small change
echo "\n## CI/CD\nGitHub Actions enabled!" >> README.md

# Commit and push
git add README.md
git commit -m "Trigger CI workflow"
git push origin main
```

#### Option B: Create a Pull Request
```bash
# Create a new branch
git checkout -b test-ci

# Make a change
echo "Test" >> test.txt
git add test.txt
git commit -m "Test CI"
git push -u origin test-ci
```

Then on GitHub:
1. You'll see a banner: "test-ci had recent pushes"
2. Click **"Compare & pull request"**
3. Click **"Create pull request"**
4. CI will run automatically!

---

### Step 5: Watch It Work!

1. Go to **Actions** tab (top menu of your repo)
2. You should see workflows running:
   - **CI** workflow (yellow circle = running)
   - Click on it to see progress
3. Wait 5-10 minutes for first build
4. Green checkmark ✅ = Success!

---

## 📸 What You'll See

### Actions Tab:
```
Actions
├── All workflows
│   ├── CI ✅ (or ⏳ or ❌)
│   └── Release (only runs on tags)
│
└── Recent workflow runs
    ├── "Trigger CI workflow" - main - ✅
    └── "Initial commit" - main - ✅
```

### Workflow Run Page:
```
CI / Trigger CI workflow
├── Jobs (5)
│   ├── Lint & Format ✅ (30s)
│   ├── Test Rust (Ubuntu) ✅ (2m)
│   ├── Test Rust (macOS) ✅ (3m)
│   ├── Test Rust (Windows) ✅ (4m)
│   ├── Build Tauri App (Ubuntu) ✅ (5m)
│   ├── Build Tauri App (macOS) ✅ (6m)
│   ├── Build Tauri App (Windows) ✅ (7m)
│   ├── Build Stub (Windows) ✅ (2m)
│   └── Security Audit ✅ (1m)
```

---

## 🚀 Creating Your First Release

### Method 1: Using Git (Recommended)

```bash
# Update version in src-tauri/Cargo.toml first
# Then:
git add src-tauri/Cargo.toml
git commit -m "Bump version to 1.0.0"
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

### Method 2: Using GitHub UI

1. Go to **Releases** (right sidebar, or `https://github.com/YOUR_USERNAME/X-Crypter/releases`)
2. Click **"Create a new release"**
3. **Choose a tag**: Type `v1.0.0` (will create tag automatically)
4. **Release title**: `X-Crypter v1.0.0`
5. **Describe this release**: Add release notes
6. Click **"Publish release"**
7. **Release workflow will start automatically!**

**What happens:**
- Release workflow triggers
- Builds for Windows, macOS, Linux
- Creates release with installers
- Takes ~15-20 minutes

---

## 🔍 Checking Workflow Status

### On Repository Homepage:
- Look for **yellow/yellow circle** next to latest commit = running
- Look for **green checkmark** ✅ = passed
- Look for **red X** ❌ = failed

### In Actions Tab:
- **All workflows** = List of all workflow files
- **Workflow runs** = History of all runs
- Click any run to see details

### In Workflow Run:
- **Jobs** = List of all jobs (lint, test, build, etc.)
- **Logs** = Detailed output (click any job to see)
- **Artifacts** = Built files (downloadable)

---

## ⚙️ GitHub Settings You Should Check

### 1. Actions Settings
**Path:** Settings → Actions → General

**Recommended:**
- ✅ **Workflow permissions**: "Read and write permissions"
- ✅ **Allow GitHub Actions to create and approve pull requests**
- ✅ **Allow all actions and reusable workflows**

### 2. Branch Protection (Optional)
**Path:** Settings → Branches

**Recommended for `main` branch:**
- ✅ Require a pull request before merging
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- Select: **CI / lint**, **CI / test**, **CI / build-tauri**

This prevents broken code from being merged!

### 3. Notifications (Optional)
**Path:** Settings → Notifications

**Recommended:**
- ✅ Email notifications for workflow failures
- ✅ Email notifications for workflow runs

---

## 🐛 Common Issues & Fixes

### Issue: "Workflow not running"

**Check:**
1. ✅ Files are in `.github/workflows/` folder
2. ✅ Files are committed and pushed
3. ✅ Actions are enabled (Settings → Actions)
4. ✅ You're pushing to `main` branch (or PR targeting `main`)

**Fix:**
```bash
# Verify files exist locally
ls -la .github/workflows/

# Verify they're tracked by git
git ls-files .github/workflows/

# If missing, add them:
git add .github/workflows/
git commit -m "Add GitHub Actions workflows"
git push origin main
```

### Issue: "Build failing"

**Check logs:**
1. Go to Actions tab
2. Click failed workflow
3. Click failed job
4. Scroll through logs
5. Look for error messages (usually in red)

**Common fixes:**
- Missing dependencies → Check `Cargo.toml`
- Rust version → Check `rust-toolchain.toml`
- Environment variables → Check if `.env` is needed

### Issue: "Can't find Actions tab"

**Possible reasons:**
1. Repository is private and Actions aren't enabled
2. You don't have admin access
3. Organization has Actions disabled

**Fix:**
- Ask repository admin to enable Actions
- Or make repository public (if appropriate)

---

## 📋 Pre-Flight Checklist

Before your first push, verify:

- [ ] `.github/workflows/ci.yml` exists
- [ ] `.github/workflows/release.yml` exists
- [ ] `rust-toolchain.toml` exists
- [ ] Code is committed locally
- [ ] GitHub repository is created
- [ ] Remote is added: `git remote -v`

---

## 🎓 Learning Resources

- **GitHub Actions Docs**: https://docs.github.com/en/actions
- **Rust CI Examples**: https://github.com/actions-rs
- **Tauri CI Examples**: https://github.com/tauri-apps/tauri/tree/dev/.github/workflows

---

## ✅ Success Indicators

You'll know it's working when:

1. ✅ **Actions tab appears** after first push
2. ✅ **Workflow runs** show up automatically
3. ✅ **Jobs complete** with green checkmarks
4. ✅ **Artifacts** are available for download
5. ✅ **Releases** are created automatically on tags

---

## 🎉 You're All Set!

Once workflows are running:
- ✅ Every push triggers CI
- ✅ Every tag triggers release
- ✅ No manual builds needed
- ✅ Multi-platform builds automatic

**Happy coding! 🚀**
