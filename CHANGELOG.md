# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2] - 2026-04-01

### Added

- Homebrew support for macOS and Linux
  - Easy installation via `brew install jsooo/tap/pretty-log`
  - Pre-built binaries for x86_64 and aarch64
- Comprehensive installation documentation

### Changed

- Improved documentation with Homebrew installation instructions

## [0.0.1] - 2026-04-01

### Added

- Initial release of pretty-log
- Streaming JSON log parser with no buffering
- ANSI color support with TTY detection
  - ERROR → red + bold
  - WARN → yellow + bold
  - INFO → green
  - DEBUG → blue
  - TRACE → dark gray
- Multi-line support for stack traces and continuation lines
- Semantic field recognition with configurable aliases
  - Default aliases for: level, timestamp, message, trace_id, caller
  - Easy customization via YAML config
- Comprehensive configuration system
  - `.pretty.yaml` in current directory
  - `~/.config/pretty/config.yaml` for user defaults
  - Command-line overrides via `--config`
- Command-line flags
  - `--expand` / `-s`: Pretty-print nested JSON values
  - `--highlight-errors` / `-e`: Red highlight for error keywords
  - `--no-color`: Disable ANSI colors
  - `--config PATH`: Specify custom config file
- Cross-platform support
  - Windows (MSVC and GNU toolchains)
  - macOS (x86_64 and aarch64)
  - Linux (x86_64 and aarch64)
- Comprehensive documentation
  - English README with quick start
  - Chinese README (中文版本)
  - Cross-platform build guide (BUILDING.md)
  - Full API documentation with doc comments
- Test suite with 39 passing tests
  - Unit tests for all modules
  - Integration tests for full pipeline
- Single static binary (~5MB) with no external dependencies

### Technical Details

- Written in Rust for performance and safety
- Streaming pipeline architecture:
  - LineReader → Parser → Classifier → Renderer
- Efficient multi-line grouping with regex patterns
- Zero-copy JSON field parsing and extraction
- TTY detection for smart color output

### Known Limitations

- JSON objects only (not arrays at the top level)
- Single-line output (no wrapping)
- No built-in filtering (use `grep` instead)
- Regex patterns that fail to compile fall back to default behavior

---

## Release Process

### Creating a New Release

1. Create a release branch from main:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b release/v0.1.0
   ```

2. Update version number in `Cargo.toml`:
   ```toml
   [package]
   version = "0.1.0"
   ```

3. Update `CHANGELOG.md` with new version section

4. Commit and push:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to 0.1.0"
   git push -u origin release/v0.1.0
   ```

5. Create pull request on GitHub

6. After merge to main, create version tag:
   ```bash
   git checkout main
   git pull origin main
   git tag v0.1.0
   git push origin v0.1.0
   ```

7. Create GitHub release with release notes

### Version Numbering

This project uses [Semantic Versioning](https://semver.org/):

- MAJOR version (breaking changes)
- MINOR version (new features, backwards compatible)
- PATCH version (bug fixes, no API changes)

Example: v1.2.3
- 1 = major
- 2 = minor
- 3 = patch
