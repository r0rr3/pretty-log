# Homebrew Support

pretty-log is available via Homebrew for macOS and Linux users.

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

- **macOS** (x86_64 and Apple Silicon/aarch64)
- **Linux** (x86_64 and aarch64)
- **Homebrew on Linux** (using Linuxbrew)

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

When releasing a new version:

1. Create a release on GitHub with the version tag (e.g., `v0.0.2`)
2. Get the SHA256 of the release tarball:
   ```bash
   curl -sL https://github.com/jsooo/pretty-log/archive/refs/tags/v0.0.2.tar.gz | shasum -a 256
   ```
3. Update the formula in the homebrew-tap repository:
   - Update version in URL
   - Update SHA256 checksum
4. Commit and push to the tap repository
5. Users can then `brew upgrade pretty-log` or `brew install` the new version

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
