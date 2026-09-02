use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use bg_engine::weidu::log::parse_active_entries;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "chriz-bg-author")]
#[command(about = "Read-only recipe authoring helpers")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Capture active WeiDU.log entries as stable TSV evidence on stdout.
    CaptureLog {
        /// Human-readable provenance label repeated on every emitted row.
        #[arg(long)]
        source_label: String,
        /// Existing WeiDU.log to read. This file is never modified.
        #[arg(long)]
        log: PathBuf,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::CaptureLog { source_label, log } => {
            let input = fs::read_to_string(log)?;
            let entries = parse_active_entries(&input)?;

            // Build the complete output before touching stdout. A malformed active row therefore
            // cannot leave a plausible-looking partial capture behind.
            let mut output = String::from(
                "position\tsource_label\tsource_line\ttp2\tlanguage\tcomponent\tcomponent_name\n",
            );
            for (position, entry) in entries.iter().enumerate() {
                writeln!(
                    output,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    position + 1,
                    escape_tsv(&source_label),
                    entry.line_number,
                    escape_tsv(&entry.tp2),
                    entry.language,
                    entry.component,
                    escape_tsv(entry.annotation.as_deref().unwrap_or("")),
                )?;
            }
            print!("{output}");
        }
    }
    Ok(())
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
