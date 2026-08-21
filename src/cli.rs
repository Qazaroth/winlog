use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Text,
    Xml,
    Json,
    Ndjson,
    Csv,
}

/// A fast, modern CLI and TUI alternative to Windows Event Viewer.
#[derive(Parser, Debug)]
#[command(
    name = "winlog",
    author,
    version,
    about = "A fast, modern CLI alternative to Windows Event Viewer",
    long_about = "winlog allows you to query, stream, and export Windows Event Logs with zero UI lag."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Live channel name (e.g., System, Security, Application) or path to an .evtx file
    #[arg(short, long, default_value = "System")]
    pub channel: String,

    /// Path to a custom YAML presets configuration file
    #[arg(short = 'C', long, global = true)]
    pub config: Option<PathBuf>,

    /// Maximum number of events to fetch
    #[arg(short, long, default_value_t = 5)]
    pub limit: u32,

    /// Output format (text, xml)
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Shortcut for JSON array output mode
    #[arg(long)]
    pub json: bool,

    /// Shortcut for NDJSON (newline-delimited JSON) output mode
    #[arg(long)]
    pub ndjson: bool,

    /// Save output directly to a file (.json, .csv, .xml, .txt)
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,
}

impl Cli {
    /// Resolves explicit flags "--json" and "--ndjson" into "OutputFormat"
    pub fn resolved_format(&self) -> OutputFormat {
        if self.ndjson {
            OutputFormat::Ndjson
        } else if self.json {
            OutputFormat::Json
        } else if let Some(path) = &self.output {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext.to_lowercase().as_str() {
                    "csv" => OutputFormat::Csv,
                    "json" => OutputFormat::Json,
                    "ndjson" => OutputFormat::Ndjson,
                    "xml" => OutputFormat::Xml,
                    _ => self.format,
                }
            } else {
                self.format
            }
        } else {
            self.format
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch interactive Terminal User Interface (TUI)
    Tui {
        /// Target live event channel or .evtx log file
        #[arg(short, long, default_value = "System")]
        channel: String,

        /// Initial maximum events to pre-load
        #[arg(short, long, default_value_t = 500)]
        limit: u32,
    },
    /// Live stream events as they occur in real time
    Tail {
        /// Target live event channel (e.g., System, Security, Application)
        #[arg(short, long, default_value = "System")]
        channel: String,

        /// Output format (text, xml)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Shortcut for JSON array output mode
        #[arg(long)]
        json: bool,

        /// Shortcut for NDJSON output mode
        #[arg(long)]
        ndjson: bool,

        /// Save streamed logs to a file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

impl Commands {
    pub fn resolved_tail_format(&self) -> OutputFormat {
        match self {
            Commands::Tui { .. } => OutputFormat::Text,
            Commands::Tail {
                format,
                json,
                ndjson,
                output,
                ..
            } => {
                if *ndjson {
                    OutputFormat::Ndjson
                } else if *json {
                    OutputFormat::Json
                } else if let Some(path) = output {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        match ext.to_lowercase().as_str() {
                            "csv" => OutputFormat::Csv,
                            "json" => OutputFormat::Json,
                            "ndjson" => OutputFormat::Ndjson,
                            "xml" => OutputFormat::Xml,
                            _ => *format,
                        }
                    } else {
                        *format
                    }
                } else {
                    *format
                }
            }
        }
    }
}
