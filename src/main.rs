//! pretty — a streaming log beautifier for JSON logs
//!
//! This CLI tool reads JSON logs from stdin and outputs a formatted, colorized version.
//! It supports multi-line grouping for stack traces and customizable field recognition.
//!
//! Pipeline:
//! stdin → LineReader → parse_line → classify → render → stdout

mod config;
mod reader;
mod parser;
mod classifier;
mod renderer;
mod table;

use std::io::{self, Write};
use clap::Parser as ClapParser;
use config::load_config;
use reader::LineReader;
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

    let stdin = io::stdin();
    let reader = LineReader::new(stdin.lock(), &config.multiline);
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for logical_line in reader {
        let result = parse_line(&logical_line.main, logical_line.continuations);
        let rendered = match result {
            ParseResult::Json(parsed) => {
                let classified = classify(parsed, &config);
                render(&classified, &config, no_color)
            }
            ParseResult::Raw { line, continuation_lines } => {
                render_raw(&line, &continuation_lines, no_color)
            }
        };
        writeln!(out, "{}", rendered).ok();
        // Flush after every line so tail -f output appears immediately
        out.flush().ok();
    }
}
