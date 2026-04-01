# Setting Up Homebrew Tap for pretty-log

This document describes how to set up and maintain the Homebrew tap for pretty-log.

## Prerequisites

1. A GitHub account
2. Homebrew installed locally (for testing)
3. Git configured

## Step 1: Create the Homebrew Tap Repository

Create a new repository named `homebrew-tap` (or similar) on GitHub.

```bash
# Example repository structure
homebrew-tap/
├── Formula/
│   └── pretty-log.rb
├── README.md
└── .github/
    └── workflows/
        └── tests.yml
```

## Step 2: Add the Formula

Copy the formula from this repository to your tap:

```bash
git clone https://github.com/YOUR-USERNAME/homebrew-tap.git
cd homebrew-tap
mkdir -p Formula
cp ../pretty-log/Formula/pretty-log.rb Formula/
```

## Step 3: Update SHA256 Hash

For each release, you need to calculate the SHA256 of the release tarball:

```bash
# Download and hash the release
curl -sL https://github.com/jsooo/pretty-log/archive/refs/tags/v0.0.2.tar.gz | shasum -a 256
# Output: abc123def456... (copy this)
```

Update the `sha256` in `Formula/pretty-log.rb` with the actual hash.

## Step 4: Test the Formula Locally

```bash
# Test installation
brew install --build-from-source ./Formula/pretty-log.rb

# Verify it works
pretty --help
echo '{"level":"info","msg":"test"}' | pretty

# Uninstall after testing
brew uninstall pretty-log
```

## Step 5: Publish the Tap

```bash
cd homebrew-tap
git add Formula/pretty-log.rb
git commit -m "Add pretty-log formula v0.0.2"
git tag v0.0.2
git push origin main --tags
```

## Step 6: Users Install the Tap

Users can now install pretty-log via your tap:

```bash
# Add tap
brew tap YOUR-USERNAME/tap https://github.com/YOUR-USERNAME/homebrew-tap

# Install
brew install pretty-log

# Or in one command
brew install YOUR-USERNAME/tap/pretty-log
```

## Automated Updates with GitHub Actions

Create `.github/workflows/tests.yml` to auto-test formulas:

```yaml
name: Homebrew Formula Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: Homebrew/actions/setup-homebrew@master
      - run: |
          brew test-bot \
            --only-formulae \
            --skip-dependents \
            Formula/pretty-log.rb
```

## Updating for New Releases

When releasing a new version of pretty-log:

1. Calculate new SHA256:
   ```bash
   curl -sL https://github.com/jsooo/pretty-log/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```

2. Update Formula/pretty-log.rb:
   - Update version in URL
   - Update sha256

3. Commit and push:
   ```bash
   git add Formula/pretty-log.rb
   git commit -m "Update pretty-log to vX.Y.Z"
   git push
   ```

## Formula Best Practices

- Keep formula simple and minimal
- Use `std_cargo_args` for cargo builds
- Include meaningful tests
- Document dependencies clearly
- Follow [Homebrew Contributing Guidelines](https://docs.brew.sh/Formula-Cookbook)

## Troubleshooting

### Formula fails to build

Check the build logs:
```bash
brew install --build-from-source --verbose ./Formula/pretty-log.rb
```

### SHA256 mismatch

Recalculate the hash:
```bash
curl -sL https://github.com/jsooo/pretty-log/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
```

### Test fails

Update the test block in the formula if the binary behavior changes.

## Official Homebrew Inclusion (Optional)

For a much wider audience, you can submit to the official Homebrew Core:

1. Create a PR against [homebrew-core](https://github.com/Homebrew/homebrew-core)
2. Follow their guidelines for new formula
3. Once merged, anyone can `brew install pretty-log` without adding a tap

This is more involved but recommended for mature projects.

## Additional Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap Documentation](https://docs.brew.sh/Taps)
- [Creating a Homebrew Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
