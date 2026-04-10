//! pretty — a streaming log beautifier for JSON logs
//!
//! This CLI tool reads JSON logs from stdin and outputs a formatted, colorized version.
//! It supports multi-line grouping for stack traces and customizable field recognition.
//!
//! Pipeline:
//! stdin → reader thread → channel (50ms timeout) → assemble LogicalLine → render → stdout

mod config;
mod reader;
mod parser;
mod classifier;
mod renderer;
mod table;

use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use clap::Parser as ClapParser;
use config::load_config;
use reader::LogicalLine;
use parser::{parse_line, ParseResult};
use classifier::classify;
use renderer::{render, render_raw};

#[derive(ClapParser, Debug)]
#[command(name = "pretty", about = "Streaming log beautifier", long_about = "pretty reads from stdin and outputs beautified JSON logs.\n\nExamples:\n  tail -f app.log | pretty\n  cat app.log | pretty -e\n  pretty < app.log")]
struct Args {
    /// Expand nested JSON field values
    #[arg(short = 's', long = "expand")]
    expand: bool,

    /// Highlight error keywords in message field
    #[arg(short = 'e', long = "highlight-errors")]
    highlight_errors: bool,

    /// Path to config file
    #[arg(long = "config", value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    /// Disable ANSI color output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Enable interactive table view
    #[arg(short = 't', long = "table")]
    table: bool,

    /// Show extras fields in expanded row detail (table mode only)
    #[arg(short = 'x', long = "extras")]
    extras: bool,

    /// Note: This tool is designed for piping. Use 'cat file.log | pretty' instead
    #[arg(value_name = "FILE", hide = true)]
    _input: Option<String>,
}

enum RawLine {
    Text(String),
    Eof,
}

fn emit_line(
    logical: LogicalLine,
    out: &mut impl Write,
    no_color: bool,
    config: &config::Config,
) {
    let result = parse_line(&logical.main, logical.continuations);
    let rendered = match result {
        ParseResult::Json(parsed) => {
            let classified = classify(parsed, config);
            render(&classified, config, no_color)
        }
        ParseResult::Raw { line, continuation_lines } => {
            render_raw(&line, &continuation_lines, no_color)
        }
    };
    writeln!(out, "{}", rendered).ok();
    out.flush().ok();
}

fn main() {
    let args = Args::parse();

    // Helpful hint if user tries to pass a file argument
    if args._input.is_some() {
        eprintln!("Error: pretty is designed for piping, not file arguments.");
        eprintln!("Usage: cat file.log | pretty");
        eprintln!("       tail -f app.log | pretty");
        eprintln!("       pretty < file.log");
        eprintln!("\nUse 'pretty --help' for more options.");
        std::process::exit(1);
    }

    let mut config = load_config(args.config.as_deref());

    // CLI flags override config file
    if args.expand {
        config.expand_nested = true;
    }
    if args.highlight_errors {
        config.highlight_errors = true;
    }

    // Disable color when not a TTY or --no-color is set
    let no_color = args.no_color || !atty::is(atty::Stream::Stdout);

    if args.table {
        let show_extras = args.extras || config.table.show_extras_in_detail;
        if let Err(e) = table::run_table_mode(&config, show_extras) {
            eprintln!("pretty: table mode error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // ── Streaming mode ──────────────────────────────────────────────────────
    //
    // A background thread reads raw lines from stdin (blocking).  The main
    // thread receives them via a channel and assembles LogicalLines (with
    // optional multiline continuation grouping).
    //
    // Key fix for `tail -f`:  recv_timeout(50ms) flushes any pending logical
    // line when no new data arrives for 50 ms.  Without this, the most-recent
    // log entry would sit in the pending buffer indefinitely, because the next
    // line never arrives until the file is written to again.

    let (raw_tx, raw_rx) = mpsc::channel::<RawLine>();

    let stdin = io::stdin();
    thread::spawn(move || {
        use io::BufRead;
        let mut locked = stdin.lock();
        let mut buf = String::new();
        loop {
            buf.clear();
            match locked.read_line(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = raw_tx.send(RawLine::Eof);
                    break;
                }
                Ok(_) => {
                    let line = buf
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if raw_tx.send(RawLine::Text(line)).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    // After 50 ms of silence, flush the pending entry so `tail -f` users
    // always see the latest log line promptly.
    let flush_timeout = Duration::from_millis(50);
    let is_cont = reader::make_continuation_checker(&config.multiline);
    let mut pending: Option<LogicalLine> = None;

    loop {
        match raw_rx.recv_timeout(flush_timeout) {
            Ok(RawLine::Text(line)) => {
                if is_cont(&line) {
                    match &mut pending {
                        Some(p) => p.continuations.push(line),
                        None => {
                            pending = Some(LogicalLine { main: line, continuations: vec![] });
                        }
                    }
                } else {
                    let prev = pending.replace(LogicalLine {
                        main: line,
                        continuations: vec![],
                    });
                    if let Some(p) = prev {
                        emit_line(p, &mut out, no_color, &config);
                    }
                }
            }
            Ok(RawLine::Eof) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                // stdin closed (EOF or pipe broken) — flush and exit
                if let Some(p) = pending.take() {
                    emit_line(p, &mut out, no_color, &config);
                }
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No new data for 50 ms — flush so `tail -f` output is
                // immediately visible without waiting for the next log line.
                if let Some(p) = pending.take() {
                    emit_line(p, &mut out, no_color, &config);
                }
            }
        }
    }
}
