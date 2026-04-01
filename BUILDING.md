# Building pretty-log

Guide to building pretty-log on different platforms.

## Requirements

- Rust 1.70+ — [Install rustup](https://rustup.rs/)
- Standard build tools for your platform

## Supported Platforms

| Platform | Arch | Status | Notes |
|----------|------|--------|-------|
| Windows | x86_64 | ✅ | MSVC or GNU toolchain |
| macOS | x86_64 | ✅ | ARM64 support planned |
| Linux | x86_64 | ✅ | glibc 2.17+ |
| Linux | aarch64 | ✅ | ARM64 support |

## Quick Start

```bash
# Clone the repository
git clone https://github.com/jsooo/pretty-log.git
cd pretty-log

# Build (debug)
cargo build

# Build optimized release binary
cargo build --release

# Run tests
cargo test

# Install to ~/.cargo/bin
cargo install --path .
```

## Platform-Specific Instructions

### Windows

#### Using MSVC (Recommended)

```bash
# Install Visual Studio Build Tools if needed
# https://visualstudio.microsoft.com/downloads/

# Build
cargo build --release

# Binary location: target/release/pretty.exe
```

#### Using GNU Toolchain (MinGW)

```bash
# Install MinGW-w64
# https://www.mingw-w64.org/

# Configure Rust
rustup target add x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

# Build
cargo build --release

# Binary location: target/release/pretty.exe
```

### macOS

```bash
# Xcode Command Line Tools required
xcode-select --install

# Build for native architecture
cargo build --release

# Binary location: target/release/pretty

# (Optional) Build for Apple Silicon (M1/M2)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install build-essential

# Build
cargo build --release

# Binary location: target/release/pretty

# (Optional) Build for ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# Cross-compile with cross tool
# https://github.com/cross-rs/cross
cargo install cross
cross build --release --target aarch64-unknown-linux-musl
```

## Release Builds

For production use, always build in release mode:

```bash
cargo build --release
```

This enables optimizations and produces a ~5MB binary.

## Testing

Run the full test suite:

```bash
cargo test
```

Run specific tests:

```bash
cargo test parser
cargo test --test integration
```

Run with output:

```bash
cargo test -- --nocapture
```

## Generating Documentation

Generate and view Rust documentation:

```bash
cargo doc --open
```

## Troubleshooting

### Linker errors on Windows

If you see "dlltool.exe not found":

```bash
# Add MinGW to PATH or configure in .cargo/config.toml
[target.x86_64-pc-windows-gnu]
linker = "C:/mingw64/mingw64/bin/gcc.exe"
```

### Build failures with dependencies

Clear build cache and retry:

```bash
cargo clean
cargo build --release
```

### Performance issues during build

If the build is slow:

```bash
# Use parallel compilation (if available)
cargo build -j 4 --release

# Use faster linker on Linux
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release
```

## Distribution

### Creating a Release

```bash
# Update version in Cargo.toml
# Create a release branch
git checkout -b release/v0.2.0

# Build all platforms
cargo build --release

# Create archives
tar czf pretty-v0.2.0-x86_64-unknown-linux-gnu.tar.gz target/release/pretty
zip pretty-v0.2.0-x86_64-pc-windows-msvc.zip target/release/pretty.exe
tar czf pretty-v0.2.0-aarch64-apple-darwin.tar.gz target/release/pretty

# Create git tag
git tag v0.2.0
git push --tags
```

## GitHub Actions CI/CD

Pre-built binaries are available for release tags from GitHub Actions. See the [Releases](https://github.com/jsooo/pretty-log/releases) page.

## Development

For development builds with debug symbols:

```bash
cargo build
cargo run -- --help
```

## Environment Variables

- `RUST_LOG` — Control logging level (if logging is implemented)
- `RUSTFLAGS` — Pass flags to the Rust compiler
- `CARGO_BUILD_JOBS` — Number of parallel build jobs
