# 🚀 GitHub Setup - Complete Guide

## Quick Links

- **Quick Start (5 min)**: See `docs/GITHUB_QUICK_START.md`
- **Detailed Setup**: See `docs/GITHUB_SETUP.md`
- **How It Works**: See `docs/GITHUB_ACTIONS_GUIDE.md`

---

## 📝 What You Need to Do on GitHub

### 1. Create Repository

1. Go to https://github.com/new
2. Repository name: `X-Crypter` (or your preferred name)
3. Description: "Advanced EXE Crypter"
4. **Visibility**: Public (for free Actions) or Private
5. **DO NOT** check "Initialize with README" (you already have code)
6. Click **"Create repository"**

---

### 2. Push Your Code

```bash
# In your project directory:
git init
git add .
git commit -m "Initial commit"

# Add GitHub remote (replace YOUR_USERNAME):
git remote add origin https://github.com/YOUR_USERNAME/X-Crypter.git

# Push to GitHub:
git branch -M main
git push -u origin main
```

---

### 3. Enable GitHub Actions

**On GitHub Website:**

1. Go to your repository: `https://github.com/YOUR_USERNAME/X-Crypter`
2. Click **Settings** tab (top menu, next to "Code")
3. Click **Actions** in left sidebar
4. Under **"Actions permissions"**:
   - Select: **"Allow all actions and reusable workflows"**
   - Click **Save**

**That's it! Actions are now enabled.**

---

### 4. Verify Workflow Files

**Check on GitHub:**

1. In your repository, click on `.github` folder
2. Click on `workflows` folder
3. You should see:
   - ✅ `ci.yml`
   - ✅ `release.yml`

**If files are missing:**
```bash
# Check locally:
ls -la .github/workflows/

# If missing, they weren't committed. Add them:
git add .github/
git commit -m "Add GitHub Actions workflows"
git push origin main
```

---

### 5. Trigger First Build

**Option A: Push a Change**
```bash
echo "\n## CI/CD\nGitHub Actions enabled!" >> README.md
git add README.md
git commit -m "Trigger CI"
git push origin main
```

**Option B: Create Pull Request**
```bash
git checkout -b test-ci
echo "Test" >> test.txt
git add test.txt
git commit -m "Test CI"
git push -u origin test-ci
```

Then on GitHub:
- Click **"Compare & pull request"**
- Click **"Create pull request"**

---

### 6. Watch It Work!

1. Go to **Actions** tab (top menu)
2. You'll see workflows running
3. Click on a workflow to see progress
4. Wait 5-10 minutes (first build is slower)
5. Green ✅ = Success!

---

## 🎯 What Happens Automatically

### On Every Push/PR:
- ✅ Lint & format check
- ✅ Run tests (Ubuntu, macOS, Windows)
- ✅ Build Tauri app (all platforms)
- ✅ Build stub (Windows)
- ✅ Security audit

### On Version Tags (`v1.0.0`):
- ✅ Build release binaries
- ✅ Create GitHub release
- ✅ Upload installers

---

## 📊 Understanding GitHub UI

### Repository Tabs:
- **Code**: Your source code
- **Issues**: Bug reports, feature requests
- **Pull requests**: Code reviews
- **Actions**: ⭐ **CI/CD workflows** (this is what you'll use!)
- **Projects**: Project management
- **Wiki**: Documentation
- **Security**: Security alerts
- **Insights**: Statistics
- **Settings**: Repository settings

### Actions Tab:
```
Actions
├── All workflows (list of workflow files)
│   ├── CI
│   └── Release
│
└── Workflow runs (execution history)
    ├── "Trigger CI" - main - ✅ (2m ago)
    ├── "Initial commit" - main - ✅ (5m ago)
    └── ...
```

### Workflow Run Page:
- **Jobs**: List of all jobs (lint, test, build, etc.)
- **Logs**: Click any job to see detailed output
- **Artifacts**: Built files you can download
- **Annotations**: Warnings and errors

---

## 🔧 GitHub Settings Explained

### Actions Settings
**Location:** Settings → Actions → General

**What to set:**
- **Workflow permissions**: "Read and write permissions" ✅
- **Allow GitHub Actions to create and approve pull requests**: ✅
- **Allow all actions and reusable workflows**: ✅

**Why:** Allows workflows to create releases, comment on PRs, etc.

### Branch Protection (Recommended)
**Location:** Settings → Branches

**For `main` branch:**
- ✅ Require a pull request before merging
- ✅ Require status checks to pass
- ✅ Require branches to be up to date

**Why:** Prevents broken code from being merged to main.

### Secrets (Optional)
**Location:** Settings → Secrets and variables → Actions

**When needed:**
- Code signing certificates
- Private dependency tokens
- API keys

**For this project:** You probably don't need any secrets initially.

---

## 🚀 Creating Releases

### Method 1: Git Tags (Recommended)

```bash
# 1. Update version in src-tauri/Cargo.toml
# 2. Commit the change
git add src-tauri/Cargo.toml
git commit -m "Bump version to 1.0.0"

# 3. Create and push tag
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

**What happens:**
- Release workflow triggers automatically
- Builds for all platforms
- Creates GitHub release
- Takes ~15-20 minutes

### Method 2: GitHub UI

1. Go to **Releases** → **"Create a new release"**
2. **Choose a tag**: `v1.0.0` (creates if doesn't exist)
3. **Release title**: `X-Crypter v1.0.0`
4. **Description**: Add release notes
5. Click **"Publish release"**

---

## 📦 Downloading Builds

### From Workflow Runs:
1. Go to **Actions** tab
2. Click on a workflow run
3. Scroll down to **Artifacts**
4. Click artifact name to download

### From Releases:
1. Go to **Releases** tab
2. Click on a release
3. Download installers from **Assets** section

---

## ✅ Success Checklist

After setup, you should see:

- [ ] **Actions tab** appears in repository
- [ ] **Workflows run** automatically on push
- [ ] **Jobs complete** with green checkmarks ✅
- [ ] **Artifacts** available for download
- [ ] **Releases** created on tags

---

## 🆘 Need Help?

### Workflow Not Running?
1. Check `.github/workflows/` files exist
2. Check Actions are enabled (Settings → Actions)
3. Check you're pushing to `main` branch

### Build Failing?
1. Click failed workflow
2. Click failed job
3. Read error messages in logs
4. Fix issue and push again

### Can't Find Actions Tab?
- Repository might be private with Actions disabled
- You might not have admin access
- Check Settings → Actions → General

---

## 📚 Documentation Files

- **`docs/GITHUB_QUICK_START.md`** - 5-minute quick start
- **`docs/GITHUB_SETUP.md`** - Detailed setup instructions
- **`docs/GITHUB_ACTIONS_GUIDE.md`** - How it works and best practices

---

## 🎉 You're Ready!

Once set up:
- ✅ Develop on macOS
- ✅ Push to GitHub
- ✅ CI tests and builds automatically
- ✅ Create releases with tags
- ✅ No Windows VM needed!

**Happy coding! 🚀**
