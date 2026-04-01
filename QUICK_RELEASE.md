# Quick Release Guide

Fast reference for releasing a new version of pretty-log.

## One-Command Release (After Merge to Main)

```bash
# 1. Go to main and update
git checkout main
git pull origin main

# 2. Create and push version tag
git tag v0.X.Y
git push origin refs/tags/v0.X.Y
```

**That's it!** GitHub Actions handles the rest:
- ✅ Builds 4 platform binaries
- ✅ Uploads to Release
- ✅ Updates Homebrew formula
- ✅ Calculates SHA256 hashes

## Full Release Workflow

### Step 1: Create Release Branch (from main)

```bash
git checkout main
git pull origin main
git checkout -b release/v0.X.Y
```

### Step 2: Update Version Files

**Cargo.toml:**
```toml
[package]
version = "0.X.Y"
```

**CHANGELOG.md:**
```markdown
## [0.X.Y] - YYYY-MM-DD

### Added
- New feature

### Fixed
- Bug fix
```

### Step 3: Commit and Push

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.X.Y"
git push -u origin release/v0.X.Y
```

### Step 4: Create Pull Request

On GitHub, click "Create Pull Request" for `release/v0.X.Y` → `main`.

### Step 5: Merge to Main

After approval, merge the PR.

### Step 6: Tag and Release

```bash
git checkout main
git pull origin main

# Create and push tag
git tag v0.X.Y
git push origin refs/tags/v0.X.Y

# Or explicitly push tag:
git push origin v0.X.Y
```

### Step 7: Wait for GitHub Actions

Check the Actions tab to see:
- Build progress (takes ~5-10 minutes)
- Binary uploads
- Formula update

### Step 8: Verify Release

1. Go to Releases page
2. Check all 4 binaries present:
   - `pretty-log-x86_64-apple-darwin.tar.gz`
   - `pretty-log-aarch64-apple-darwin.tar.gz`
   - `pretty-log-x86_64-unknown-linux-gnu.tar.gz`
   - `pretty-log-aarch64-unknown-linux-gnu.tar.gz`
3. Check `Formula/pretty-log.rb` has SHA256 hashes updated

### Step 9: Test Homebrew Installation (Optional)

```bash
# On macOS or Linux:
brew tap jsooo/tap https://github.com/r0rr3/pretty-log.git
brew install pretty-log --HEAD

# Verify
pretty --version
```

## What Gets Automated

When you push the version tag, GitHub Actions automatically:

1. **Build Matrix**
   - 4 parallel builds (macOS x86_64, macOS ARM64, Linux x86_64, Linux ARM64)
   - Each: compile + strip + tar.gz

2. **Create Release**
   - Automatically creates GitHub Release

3. **Upload Binaries**
   - All 4 compiled binaries → Release

4. **Calculate Hashes**
   - SHA256 for each binary

5. **Update Formula**
   - Generates `Formula/pretty-log.rb` with:
     - Correct download URLs
     - SHA256 checksums
   - Commits back to repository

## Versioning Scheme

`vX.Y.Z` where:
- **X** = major (breaking changes)
- **Y** = minor (new features)
- **Z** = patch (bug fixes)

Examples:
- `v0.0.1` → `v0.0.2` (patch: bug fix)
- `v0.0.2` → `v0.1.0` (minor: new feature)
- `v0.1.0` → `v1.0.0` (major: breaking change)

## Checklist

- [ ] Update `Cargo.toml` version
- [ ] Update `CHANGELOG.md`
- [ ] Create release branch: `release/v0.X.Y`
- [ ] Push to GitHub
- [ ] Create and merge PR to main
- [ ] Tag main: `git tag v0.X.Y`
- [ ] Push tag: `git push origin v0.X.Y`
- [ ] Wait for GitHub Actions (check Actions tab)
- [ ] Verify Release page has all 4 binaries
- [ ] Test Homebrew install (optional)

## Troubleshooting

### GitHub Actions failed to build

1. Check Actions tab for error logs
2. Likely issues:
   - Rust toolchain unavailable (retry, usually temporary)
   - Compilation error (fix code, retag with new SHA: `git tag -f v0.X.Y && git push -f origin v0.X.Y`)

### Missing binaries in Release

1. Check Actions tab — workflow may still be running
2. If completed but binaries missing, check "upload-release-asset" step for errors

### Formula update didn't happen

1. Workflow may still be running
2. If done, manually create PR in homebrew-tap repo with updated checksums

### Users report install failing

```bash
# Debug locally:
brew install --verbose jsooo/tap/pretty-log

# Check formula:
brew cat jsooo/tap/pretty-log

# List available versions:
brew search pretty-log
```

## See Also

- [VERSIONING.md](VERSIONING.md) — Branching strategy
- [CI_CD.md](CI_CD.md) — Detailed workflow docs
- [HOMEBREW.md](HOMEBREW.md) — Homebrew details
- [.github/workflows/release.yml](.github/workflows/release.yml) — Workflow source
