# Versioning and Release Strategy

## Semantic Versioning

pretty-log follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version (X.0.0) — breaking changes to CLI or output format
- **MINOR** version (0.X.0) — new features, backwards compatible
- **PATCH** version (0.0.X) — bug fixes, internal improvements

Examples:
- v0.0.1 → v0.0.2 (patch: bug fix)
- v0.0.2 → v0.1.0 (minor: new feature)
- v0.1.0 → v1.0.0 (major: breaking change)

## Git Branching Strategy

### Main branches

- **main** — production-ready code, stable releases only
- **develop** — integration branch for next release (optional, use as needed)

### Supporting branches

- **release/vX.Y.Z** — prepare a specific release
  - Branch from: `main`
  - Tag and merge back: `main`, optionally `develop`
  - Naming: `release/v0.1.0`, `release/v1.2.3`

- **feature/*** — new features
  - Branch from: `main` or `develop`
  - Merge back via: pull request
  - Naming: `feature/streaming-filters`, `feature/config-validation`

- **hotfix/vX.Y.Z** — urgent production fixes
  - Branch from: `main` (latest tag)
  - Merge back: `main` and `develop` (if exists)
  - Naming: `hotfix/v0.0.2`

## Release Process

### 1. Prepare Release Branch

```bash
# Start from main
git checkout main
git pull origin main

# Create release branch
git checkout -b release/v0.1.0
```

### 2. Update Version

**In `Cargo.toml`:**
```toml
[package]
name = "pretty-log"
version = "0.1.0"  # ← Update this
```

**In `CHANGELOG.md`:**
- Add `## [0.1.0] - YYYY-MM-DD` section at top
- Document all changes since last version
- Keep `[Unreleased]` section for next development

### 3. Commit and Push

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.1.0"
git push -u origin release/v0.1.0
```

### 4. Create Pull Request

On GitHub, create a PR from `release/v0.1.0` → `main`:
- Title: "Release v0.1.0"
- Description: Link to CHANGELOG section or summarize changes
- Request review if applicable

### 5. Merge to Main

After approval, merge the release PR (preferably with "Create a merge commit" to preserve history).

### 6. Create Git Tag

```bash
# Update main with the merge
git checkout main
git pull origin main

# Tag the release commit
git tag v0.1.0

# Push tag
git push origin refs/tags/v0.1.0
```

### 7. Create GitHub Release

On GitHub:
1. Go to [Releases](https://github.com/r0rr3/pretty-log/releases)
2. Click "New release"
3. Tag version: `v0.1.0`
4. Title: `v0.1.0`
5. Description: Copy from CHANGELOG.md
6. Attach binary artifacts (optional)
7. Publish

### 8. Update Develop Branch (if applicable)

```bash
git checkout develop
git pull origin develop
git merge main --no-ff -m "Merge release v0.1.0 into develop"
git push origin develop
```

## Quick Reference: Current Release

**Current Version:** v0.0.1  
**Release Date:** 2026-04-01  
**Branch:** v0.0.1 (release branch)  
**Tag:** v0.0.1

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Hotfix Process (Urgent Fixes)

If a critical bug is found in a released version:

```bash
# Start from the release tag
git checkout v0.1.0
git checkout -b hotfix/v0.1.1

# Make fix, test, commit
git add .
git commit -m "fix: critical issue description"

# Create PR to main
git push -u origin hotfix/v0.1.1
# → Create PR on GitHub

# After merge
git checkout main
git pull
git tag v0.1.1
git push origin refs/tags/v0.1.1
```

## Version Bump Checklist

Before releasing, verify:

- [ ] All tests pass: `cargo test`
- [ ] All changes documented in CHANGELOG.md
- [ ] Version number updated in Cargo.toml
- [ ] No uncommitted changes
- [ ] Release branch created and pushed
- [ ] PR created and reviewed
- [ ] PR merged to main
- [ ] Tag created and pushed
- [ ] GitHub release created (optional but recommended)

## Automation Notes

Currently, releases are manual. Future improvements could include:
- GitHub Actions to automatically build and publish binaries
- Automated changelog generation from commits
- Pre-release validation and testing
- Docker image publishing
