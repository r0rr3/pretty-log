# pretty-log

A fast log prettifier for JSON logs. Piped from `tail -f`, it gives you colored output, multi-line support for stack traces, and sane field recognition out of the box.

**[中文版本](README.zh-CN.md)** | **[English](README.md)**

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## What you get

- Streaming JSON parsing — reads from stdin, no buffering
- ANSI colors — detects if you're in a real terminal and colors accordingly
- Multi-line support — knows when a line is a stack trace and indents it
- Sensible defaults — works immediately with common field names (level, time, msg, trace_id)
- YAML config — customize field names and behaviors if your logs use something different
- No dependencies — single static ~5MB binary, runs anywhere

## Install

### Homebrew (macOS and Linux)

```bash
brew install jsooo/tap/pretty-log
```

See [HOMEBREW.md](HOMEBREW.md) for more details.

### From source

```bash
git clone https://github.com/jsooo/pretty-log.git
cd pretty-log
cargo build --release
./target/release/pretty --help
```

## Usage

```bash
tail -f app.log | pretty           # stream with colors
cat app.log | pretty               # pipe a file
tail -f app.log | pretty -t        # interactive table mode
tail -f app.log | pretty -t -x     # table mode + show extras in detail panel
```

## What does it do?

Here's a real example. Feed it JSON:

```json
{"level":"info","msg":"server started","port":8080,"time":"2024-06-15T14:30:00Z"}
{"level":"error","msg":"crash","trace_id":"abc-123","time":"2024-06-15T14:30:01Z"}
goroutine 1 [running]:
main.handler(...)
```

You get:

```
14:30:00  INFO   server started  port=8080
14:30:01  ERROR  crash  trace=abc-123
  goroutine 1 [running]:
  main.handler(...)
```

## Options

| Flag | Description |
|------|-------------|
| `-s`, `--expand` | Expand nested JSON field values |
| `-t`, `--table` | Enable interactive table view |
| `-x`, `--extras` | Show extras fields in expanded row detail (table mode only) |
| `--config <path>` | Path to config file |
| `--no-color` | Disable ANSI color output |

## Table Mode

Activated with `-t`. Displays logs in a full-terminal interactive table.

**Key bindings:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| Mouse wheel | Scroll |
| `Enter` | Expand/collapse row detail |
| `End` / `G` | Jump to latest row |
| `Space` | Pause/resume auto-scroll |
| `q` / `Ctrl-C` | Quit |

When scrolling up during a live stream, new logs continue buffering. The status bar shows `↓ N new`. Press `End` to resume.

## Configuration

Config file locations (in priority order):
1. `--config <path>`
2. `.pretty.yaml` in current directory
3. `~/.config/pretty/config.yaml`

```yaml
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id]
  caller:    [caller, file, source]

expand_nested: false
highlight_errors: false

multiline:
  enabled: true
  continuation_pattern: "^[^{]"

# Table mode settings
table:
  # Column display order — omit any column to hide it
  columns: [time, level, message, service, label, trace_id]
  # Show extra fields (not in a dedicated column) in the expanded row detail panel
  # Can also be enabled per-run with -x / --extras
  show_extras_in_detail: false
```

## Default field names

Recognized out of the box:

- level: level, lvl, severity, log_level
- timestamp: time, timestamp, ts, @timestamp
- message: msg, message, body
- trace_id: trace_id, traceId, traceid, request_id, x-trace-id
- caller: caller, file, source

## Colors

- ERROR → red + bold
- WARN → yellow + bold
- INFO → green
- DEBUG → blue
- TRACE → dark gray
- Other fields → standard colors

## Build and test

```bash
cargo build
cargo build --release
cargo test
```

## How it works

```
pretty-log/
├── src/
│   ├── main.rs          entry point, CLI flags
│   ├── config.rs        load and merge YAML config
│   ├── reader.rs        group lines with stack traces
│   ├── parser.rs        detect JSON, extract fields
│   ├── classifier.rs    figure out what each field means
│   └── renderer.rs      format and colorize
├── tests/integration.rs full pipeline tests
├── Cargo.toml
└── README.md
```

## Testing

```bash
cargo test
cargo test basic_json_line_output
cargo test -- --nocapture
```

## Known limits

- JSON objects only (not arrays at the top level)
- Single-line output (no wrapping)
- No built-in filtering (pipe to grep instead)
- Regex in config that don't compile just use the fallback

## License

MIT

---

Made with ❤️ in Rust
