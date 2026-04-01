# CI/CD Pipeline — Automated Releases

pretty-log uses GitHub Actions for automated building and releasing of cross-platform binaries.

## Overview

When you push a version tag (e.g., `v0.0.2`), GitHub Actions automatically:

1. ✅ Builds binaries for 4 platforms
2. ✅ Uploads to GitHub Release
3. ✅ Calculates SHA256 checksums
4. ✅ Updates Homebrew formula

**Result:** Users get fast installation via `brew install jsooo/tap/pretty-log` (no compilation)

## Workflow: `.github/workflows/release.yml`

### Trigger

```yaml
on:
  push:
    tags:
      - 'v*'
```

Runs automatically when you push a version tag.

### Build Matrix

Compiles for 4 platforms in parallel:

| Platform | Target | OS |
|----------|--------|-----|
| macOS x86_64 | `x86_64-apple-darwin` | `macos-latest` |
| macOS ARM64 | `aarch64-apple-darwin` | `macos-latest` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `ubuntu-latest` |

### Build Details

- Uses `dtolnay/rust-toolchain` for Rust
- Uses `cross` tool for ARM64 Linux (cross-compilation)
- Optimized builds (`--release`)
- Creates `.tar.gz` artifacts for each platform

### Artifact Upload

All binaries uploaded to GitHub Release with names:

```
pretty-log-x86_64-apple-darwin.tar.gz
pretty-log-aarch64-apple-darwin.tar.gz
pretty-log-x86_64-unknown-linux-gnu.tar.gz
pretty-log-aarch64-unknown-linux-gnu.tar.gz
```

### Homebrew Formula Update

After all builds complete:

1. Downloads all artifacts
2. Calculates SHA256 for each
3. Updates `Formula/pretty-log.rb` with:
   - Correct download URLs
   - SHA256 checksums for each platform
4. Commits and pushes updated formula

## How to Release

### 1. Prepare Release Branch

```bash
git checkout main
git pull origin main
git checkout -b release/v0.X.Y
```

### 2. Update Version and Changelog

```bash
# In Cargo.toml
version = "0.X.Y"

# In CHANGELOG.md
## [0.X.Y] - YYYY-MM-DD
### Added
- Feature description

### Fixed
- Bug fix description
```

### 3. Commit and Push

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.X.Y"
git push -u origin release/v0.X.Y
```

### 4. Create Pull Request

On GitHub, create PR from `release/v0.X.Y` to `main`.

### 5. Merge to Main

After approval, merge the PR (use "Create a merge commit" to preserve history).

### 6. Create Version Tag

```bash
git checkout main
git pull origin main
git tag v0.X.Y
git push origin refs/tags/v0.X.Y
```

**GitHub Actions will automatically:**
- 🔨 Build 4 binaries
- 📦 Upload to Release
- 🔐 Calculate checksums
- 📝 Update Homebrew formula

### 7. Verify Release

Wait for GitHub Actions workflow to complete:

1. Go to Actions tab
2. See "Release" workflow status
3. Check Release page for binaries
4. Verify `Formula/pretty-log.rb` was updated with SHA256 hashes

### 8. Announce Release (Optional)

Create GitHub Release notes with highlights:
- New features
- Bug fixes
- Known issues

## Manual Testing

To test the Homebrew installation locally before a public release:

```bash
# Install the specific formula file
brew install --build-from-source ./Formula/pretty-log.rb

# Test
pretty --version
echo '{"level":"info","msg":"test"}' | pretty

# Clean up
brew uninstall pretty-log
```

## Troubleshooting

### Build fails for a specific platform

Check the GitHub Actions workflow logs:
1. Go to Actions tab
2. Click the failed workflow
3. Expand the build step to see error

Common issues:
- Missing dependencies (e.g., `cross` for ARM64 Linux)
- Rust toolchain issues (usually temporary, retry)
- Source code compilation error (fix and re-tag)

### Formula update didn't work

Check the workflow logs for the "update-homebrew-formula" step.

If manual update needed:
```bash
# Calculate SHA256 for a binary
curl -sL https://github.com/r0rr3/pretty-log/releases/download/vX.Y.Z/pretty-log-x86_64-apple-darwin.tar.gz | shasum -a 256

# Update Formula/pretty-log.rb manually
git add Formula/pretty-log.rb
git commit -m "chore: update Homebrew formula SHA256"
git push
```

### Users report installation issues

1. Verify the formula has correct URLs and SHA256 hashes
2. Test locally with `brew install --verbose jsooo/tap/pretty-log`
3. Check GitHub Release page has all 4 binaries

## Future Improvements

Potential enhancements to the CI/CD pipeline:

- [ ] Automatically create GitHub Release draft with changelog
- [ ] Sign binaries with GPG for additional security
- [ ] Generate macOS .dmg package
- [ ] Publish to official Homebrew Core (wider audience)
- [ ] Build Windows binaries (.exe)
- [ ] Auto-increment version patch in workflow
- [ ] Publish to other package managers (apt, pacman, etc.)

## Workflow YAML

See `.github/workflows/release.yml` for complete workflow definition.

Key sections:
- **create-release**: Creates GitHub Release
- **build-and-upload**: Builds 4 binaries, uploads to Release
- **update-homebrew-formula**: Generates and commits updated formula
