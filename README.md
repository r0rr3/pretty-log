# pretty-log

A fast, streaming log prettifier for JSON logs. Pipe from `tail -f` for live colored output, or use `-t` for a full-terminal interactive table.

**[中文版本](README.zh-CN.md)** | **[English](README.md)**

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## What you get

- Streaming JSON parsing — reads from stdin line by line, no buffering delay
- ANSI colors — auto-detects terminal and colorizes by log level
- Multi-line support — groups stack traces with their parent log entry
- Sensible defaults — works immediately with common field names (`level`, `time`, `msg`, `trace_id`)
- Interactive table mode — full-terminal TUI with search, scrolling, and detail panel
- YAML config — customize field names and behaviors if your logs use different keys
- Single static binary — ~5 MB, no runtime dependencies

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
tail -f app.log | pretty              # live stream with colors
cat app.log | pretty                  # pipe a file
tail -f app.log | pretty -t           # interactive table mode
tail -f app.log | pretty -t -x        # table mode + extras in detail panel
cat app.log | pretty --no-color | grep ERROR   # pipe-friendly output
```

## What does it look like?

Input:

```json
{"level":"info","msg":"server started","port":8080,"time":"2024-06-15T14:30:00Z"}
{"level":"error","msg":"crash","trace_id":"abc-123","time":"2024-06-15T14:30:01Z"}
goroutine 1 [running]:
main.handler(...)
```

Output:

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
| `-e`, `--highlight-errors` | Highlight error keywords in message |
| `-t`, `--table` | Enable interactive table view |
| `-x`, `--extras` | Show extra fields in detail panel (table mode only) |
| `--config <path>` | Path to config file |
| `--no-color` | Disable ANSI color output |

## Table Mode

Activated with `-t`. Displays logs in a full-terminal interactive table with wrapping messages, a detail panel, and live search.

**Key bindings:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor up / down |
| Mouse wheel | Scroll |
| `g` / `Home` | Jump to first row |
| `G` / `End` | Jump to latest row |
| `Space` | Pause / resume live tail |
| `/` | Open search |
| `n` / `N` | Next / previous search match |
| `Esc` | Clear search |
| `q` | Quit |

**Search** uses KMP for fast case-insensitive matching across message and all extra fields. Matches are highlighted inline.

When scrolling up during a live stream, new logs continue buffering. The status bar shows `↓ N new`. Press `G` or `End` to jump back to the latest.

## Configuration

Config file locations (in priority order):

1. `--config <path>`
2. `.pretty.yaml` in current directory
3. `~/.config/pretty/config.yaml`

```yaml
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts, "@timestamp"]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id, x-trace-id]
  caller:    [caller, file, source]

expand_nested: false
highlight_errors: false

multiline:
  enabled: true
  continuation_pattern: "^[^{]"

table:
  columns: [time, level, message]
  show_extras_in_detail: false
```

## Default field names

Recognized out of the box:

- **level** — `level`, `lvl`, `severity`, `log_level`
- **timestamp** — `time`, `timestamp`, `ts`, `@timestamp`
- **message** — `msg`, `message`, `body`
- **trace_id** — `trace_id`, `traceId`, `traceid`, `request_id`, `x-trace-id`
- **caller** — `caller`, `file`, `source`

## Colors

| Level | Color |
|-------|-------|
| ERROR | Red |
| WARN  | Yellow |
| INFO  | Green |
| DEBUG | Blue |
| TRACE | Dark gray |

## How it works

```
stdin
  └─ reader thread (blocking read_line)
       └─ channel (50ms timeout flush)
            └─ multiline assembler
                 └─ parser → classifier → renderer → stdout
```

The 50ms timeout ensures the last log line of each burst is always displayed promptly, which is what makes `tail -f` work correctly.

## Project structure

```
pretty-log/
├── src/
│   ├── main.rs          entry point, CLI flags, streaming loop
│   ├── config.rs        load and merge YAML config
│   ├── reader.rs        multiline grouping, continuation checker
│   ├── parser.rs        detect JSON, extract fields
│   ├── classifier.rs    map fields to semantic roles
│   ├── renderer.rs      format and colorize output
│   └── table.rs         interactive TUI table mode (-t)
├── tests/
│   └── integration.rs   end-to-end pipeline tests
├── Cargo.toml
└── README.md
```

## Build and test

```bash
cargo build
cargo build --release
cargo test
```

## Known limits

- JSON objects only — arrays at the top level are passed through as raw text
- No built-in filtering — pipe to `grep` or `jq` instead
- Regex in config that fail to compile fall back to the default pattern

## License

MIT

---

Made with ❤️ in Rust
