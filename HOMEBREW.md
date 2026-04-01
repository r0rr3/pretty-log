# Homebrew Support

pretty-log is available via Homebrew for macOS and Linux users. Installation is fast since we provide precompiled binaries for all platforms.

## Installation via Homebrew

```bash
brew install jsooo/tap/pretty-log
```

Or add the tap first:

```bash
brew tap jsooo/tap
brew install pretty-log
```

## Supported Platforms

- **macOS** (x86_64 and Apple Silicon/aarch64) — precompiled binary
- **Linux** (x86_64 and aarch64) — precompiled binary
- **Homebrew on Linux** (using Linuxbrew) — precompiled binary

Installation is **fast** — downloads precompiled binary (~5MB), no compilation needed!

## Building from Source with Homebrew

If you want to build the latest development version:

```bash
brew install --HEAD jsooo/tap/pretty-log
```

## Verification

After installation, verify it works:

```bash
pretty --version
echo '{"level":"info","msg":"hello"}' | pretty
```

## Homebrew Tap Repository

The Homebrew formula is maintained in the `jsooo/tap` Homebrew tap:
- Repository: https://github.com/jsooo/homebrew-tap
- Formula: [Formula/pretty-log.rb](https://github.com/jsooo/homebrew-tap/blob/main/Formula/pretty-log.rb)

### Setting Up Your Own Tap

If you want to fork this and create your own tap:

1. Create a new repository named `homebrew-<tapname>`
2. Add formulas to `Formula/` directory
3. Users can then install with: `brew install <username>/<tapname>/<formula>`

### Creating/Updating a Formula

The formula file is automatically generated during the release process. It specifies:

- The release tarball URL
- SHA256 checksum of the release
- Build dependencies (Rust)
- Installation steps
- Test cases

See [Formula/pretty-log.rb](Formula/pretty-log.rb) for the current formula.

## Release Process

### Automated (GitHub Actions)

Releases are fully automated via GitHub Actions. When you push a version tag:

1. **Create release tag:**
   ```bash
   git tag v0.0.2
   git push origin v0.0.2
   ```

2. **GitHub Actions automatically:**
   - Builds binaries for 4 platforms (macOS x86_64/aarch64, Linux x86_64/aarch64)
   - Uploads all binaries to GitHub Release
   - Calculates SHA256 checksums
   - Updates the Homebrew formula with correct URLs and checksums
   - Commits the updated formula back to the repository

3. **Users can install:**
   ```bash
   brew upgrade pretty-log
   # or
   brew install pretty-log
   ```

### Manual Process (if needed)

If you need to update the formula manually:

1. Get the SHA256 of each release binary:
   ```bash
   curl -sL https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-apple-darwin.tar.gz | shasum -a 256
   ```

2. Update `Formula/pretty-log.rb` with the checksums

3. Push to the tap repository

## Troubleshooting

### Formula not found

```bash
brew tap jsooo/tap
brew install pretty-log
```

### Build fails

Try building from source:

```bash
brew install --build-from-source jsooo/tap/pretty-log
```

### Check formula info

```bash
brew info jsooo/tap/pretty-log
```

### View formula source

```bash
brew edit jsooo/tap/pretty-log
```

## macOS Code Signing

For macOS releases, binaries can optionally be signed. To add code signing:

1. Generate the signed binary in the release
2. Update the SHA256 in the formula to match the signed binary

## License

The formula follows the same MIT license as the pretty-log project.
