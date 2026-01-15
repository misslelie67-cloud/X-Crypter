# ✅ GitHub Setup Checklist

Follow these steps in order to set up GitHub Actions for your project.

---

## 📋 Pre-Setup Checklist

Before starting, make sure you have:

- [ ] GitHub account (free account works)
- [ ] Code committed locally with git
- [ ] `.github/workflows/` folder exists with workflow files
- [ ] `rust-toolchain.toml` file exists

---

## 🚀 Setup Steps

### Step 1: Create GitHub Repository

- [ ] Go to https://github.com/new
- [ ] Repository name: `X-Crypter`
- [ ] Description: (optional)
- [ ] Visibility: Public or Private
- [ ] **DO NOT** check "Initialize with README"
- [ ] Click **"Create repository"**

**✅ Done when:** You see an empty repository page

---

### Step 2: Push Code to GitHub

**In your terminal:**

```bash
# Check if git is initialized
git status

# If not initialized:
git init
git add .
git commit -m "Initial commit with GitHub Actions"

# Add GitHub remote (replace YOUR_USERNAME):
git remote add origin https://github.com/misslelie67-cloud/X-Crypter.git

# Push code
git branch -M main
git push -u origin main
```

- [ ] Code pushed successfully
- [ ] Can see files on GitHub

**✅ Done when:** You can see your code files on GitHub

---

### Step 3: Verify Workflow Files

**On GitHub:**

1. Click on `.github` folder
2. Click on `workflows` folder
3. Verify you see:
   - [ ] `ci.yml`
   - [ ] `release.yml`

**If files are missing:**

```bash
# Check locally
ls -la .github/workflows/

# Add and push if missing
git add .github/
git commit -m "Add GitHub Actions workflows"
git push origin main
```

**✅ Done when:** Both workflow files are visible on GitHub

---

### Step 4: Enable GitHub Actions

**On GitHub:**

1. Go to repository **Settings** (top menu)
2. Click **Actions** (left sidebar)
3. Under **"Actions permissions"**:
   - [ ] Select **"Allow all actions and reusable workflows"**
   - [ ] Click **Save**

**✅ Done when:** Settings saved successfully

---

### Step 5: Trigger First Build

**Option A: Push a Change**

```bash
echo "\n## CI/CD\nGitHub Actions enabled!" >> README.md
git add README.md
git commit -m "Trigger CI workflow"
git push origin main
```

- [ ] Change pushed
- [ ] Went to Actions tab
- [ ] See workflow running

**Option B: Create Pull Request**

```bash
git checkout -b test-ci
echo "Test" >> test.txt
git add test.txt
git commit -m "Test CI"
git push -u origin test-ci
```

Then on GitHub:
- [ ] Click **"Compare & pull request"**
- [ ] Click **"Create pull request"**
- [ ] See CI running on PR

**✅ Done when:** Workflow appears in Actions tab

---

### Step 6: Monitor First Build

**On GitHub Actions tab:**

- [ ] Workflow shows as running (yellow circle ⏳)
- [ ] Can see job list (lint, test, build, etc.)
- [ ] Jobs are completing
- [ ] All jobs show green checkmark ✅

**Expected time:** 5-15 minutes for first build

**✅ Done when:** All jobs pass (green checkmarks)

---

### Step 7: Configure Settings (Optional but Recommended)

**Branch Protection:**

1. Go to **Settings** → **Branches**
2. Click **"Add rule"** for `main` branch
3. Check:
   - [ ] Require a pull request before merging
   - [ ] Require status checks to pass
   - [ ] Require branches to be up to date
4. Click **"Create"**

**✅ Done when:** Branch protection rule created

---

## 🎉 Setup Complete!

Once all steps are done:

- ✅ CI runs on every push
- ✅ Tests run automatically
- ✅ Builds happen automatically
- ✅ Ready to create releases

---

## 🚀 Next Steps

### Create Your First Release:

```bash
# Update version
# Then:
git tag v1.0.0
git push origin v1.0.0
```

**Or use GitHub UI:**
1. Go to **Releases** → **"Create a new release"**
2. Tag: `v1.0.0`
3. Title: `X-Crypter v1.0.0`
4. Click **"Publish release"**

---

## 📖 Need More Help?

- **Quick Start**: `docs/GITHUB_QUICK_START.md`
- **Detailed Guide**: `docs/GITHUB_SETUP.md`
- **How It Works**: `docs/GITHUB_ACTIONS_GUIDE.md`

---

## 🐛 Troubleshooting

### Workflow Not Running?
- ✅ Check `.github/workflows/` files exist on GitHub
- ✅ Check Actions enabled (Settings → Actions)
- ✅ Check you pushed to `main` branch

### Build Failing?
- ✅ Click failed job to see logs
- ✅ Check error messages
- ✅ Fix and push again

### Can't Find Actions Tab?
- ✅ Check repository Settings → Actions
- ✅ Verify you have admin access
- ✅ Check if Actions are enabled for your account

---

**You're all set! 🎉**
