# pretty-log

A high-performance streaming log beautifier for the terminal. Transform raw JSON logs into human-readable, colorized output with support for multi-line stack traces and customizable field mapping.

**[中文版本](README.zh-CN.md)** | **[English](README.md)**

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Features

✨ **Streaming JSON Parser** — Process logs line-by-line without buffering  
🎨 **ANSI Color Output** — Auto-detects TTY for intelligent colorization  
📋 **Multi-line Grouping** — Preserves stack traces and indents continuation lines  
⚙️ **Zero-Config Default** — Works out of the box with sensible field aliases  
🔧 **Flexible Configuration** — YAML config for custom field mappings and behaviors  
⚡ **Single Static Binary** — No runtime dependencies, ~5MB release build  

## Installation

### Build from Source

```bash
git clone https://github.com/jsooo/pretty-log.git
cd pretty-log
cargo build --release
./target/release/pretty --help
```

## Quick Start

```bash
# Basic usage
tail -f app.log | pretty

# Highlight errors
tail -f app.log | pretty -e

# Expand nested JSON
cat app.log | pretty -s

# Disable colors
cat app.log | pretty --no-color | grep "ERROR"
```

## Examples

**Input:**
```json
{"level":"info","msg":"server started","port":8080,"time":"2024-06-15T14:30:00Z"}
{"level":"error","msg":"crash","trace_id":"abc-123","time":"2024-06-15T14:30:01Z"}
goroutine 1 [running]:
main.handler(...)
```

**Output:**
```
14:30:00  INFO   server started  port=8080
14:30:01  ERROR  crash  trace=abc-123
  goroutine 1 [running]:
  main.handler(...)
```

## CLI Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--expand` | `-s` | Expand nested JSON field values |
| `--highlight-errors` | `-e` | Highlight error keywords (red) |
| `--config PATH` | | Load YAML configuration file |
| `--no-color` | | Disable ANSI color output |

## Configuration

### Default Locations

1. `--config <path>` (CLI)
2. `.pretty.yaml` (current directory)
3. `~/.config/pretty/config.yaml` (user home)
4. Built-in defaults

### Example Config

```yaml
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts, "@timestamp"]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id]
  caller:    [caller, file, source]

expand_nested: false
highlight_errors: false

multiline:
  enabled: true
  continuation_pattern: "^[^\{]"
```

## Built-in Field Aliases

- **level:** `level`, `lvl`, `severity`, `log_level`
- **timestamp:** `time`, `timestamp`, `ts`, `@timestamp`
- **message:** `msg`, `message`, `body`
- **trace_id:** `trace_id`, `traceId`, `traceid`, `request_id`, `x-trace-id`
- **caller:** `caller`, `file`, `source`

## Color Scheme

| Element | Color |
|---------|-------|
| ERROR | Red + Bold |
| WARN | Yellow + Bold |
| INFO | Green |
| DEBUG | Blue |
| TRACE | Dark Gray |
| Timestamp | Cyan |
| Message | White + Bold |
| Trace ID | Magenta |
| Continuation Lines | Dark Gray (indented) |

## Use Cases

### Development
```bash
cargo run -- app.log | pretty -e
```

### Production
```bash
tail -f /var/log/app.log | pretty
```

### Pipelines
```bash
# Filter errors only
cat app.log | pretty | grep "ERROR"

# Save formatted logs
tail -f app.log | pretty > formatted.log
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
pretty-log/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── config.rs        # YAML configuration loading
│   ├── reader.rs        # Streaming reader with multi-line support
│   ├── parser.rs        # JSON parsing
│   ├── classifier.rs    # Field semantic recognition
│   └── renderer.rs      # ANSI color rendering
├── tests/
│   └── integration.rs   # End-to-end tests
├── Cargo.toml
└── README.md
```

## Testing

```bash
# All tests
cargo test

# Specific test
cargo test basic_json_line_output

# With output
cargo test -- --nocapture
```

## Limitations

- JSON objects only (not arrays as top-level lines)
- Single-line output format
- No built-in filtering (use shell pipes)
- Invalid regex patterns fall back to defaults

## License

MIT License — See [LICENSE](LICENSE)

---

Made with ❤️ in Rust
