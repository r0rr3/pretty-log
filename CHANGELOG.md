# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-04-19

### Fixed

- **Table mode blank screen on oversize rows**: when a single log row's rendered
  height (wrapped message + detail pairs) exceeded the visible area, the render
  loop skipped every row and the screen went blank.  `row_height` is now capped
  at the visible height, long rows are truncated with a `… (N more lines)`
  indicator, and the renderer always shows at least one row as a safety net.
  `scroll_offset_for_bottom` also no longer scrolls past the selected row when
  that row alone is taller than the viewport.

## [0.0.8] - 2026-04-10

### Fixed

- **`tail -f` streaming** (`tail -f app.log | pretty`): the last log line was
  stuck in an internal buffer until the *next* line arrived, making it look
  like pretty had exited after processing existing content.  The normal-mode
  reader is now backed by a dedicated stdin thread + 50 ms timeout flush,
  matching the approach already used by table mode.  Affects both macOS and
  Windows.

## [0.0.2] - 2026-04-01

### Added

- **Homebrew support** for macOS and Linux
  - Easy installation: `brew install jsooo/tap/pretty-log`
  - Pre-compiled binaries for x86_64 and aarch64 (no compilation needed!)
- **GitHub Actions workflow** for automated cross-platform releases
  - Automatically builds binaries for 4 platforms on version tag:
    - macOS x86_64 and aarch64 (Apple Silicon)
    - Linux x86_64 and aarch64
  - Uploads binaries to GitHub Release
  - Calculates SHA256 checksums automatically
  - Updates Homebrew formula with correct checksums and URLs
- Comprehensive Homebrew documentation

### Changed

- Improved documentation with Homebrew installation instructions
- Homebrew formula now uses precompiled binaries (faster installation)
- Automated release process reduces manual work

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
