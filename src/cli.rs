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

    #[arg(short, long, default_value = "System")]
    pub channel: String,

    #[arg(short = 'C', long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(short, long, default_value_t = 5)]
    pub limit: u32,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub ndjson: bool,

    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,
}

impl Cli {
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
    Tui {
        #[arg(short, long, default_value = "System")]
        channel: String,

        #[arg(short, long, default_value_t = 500)]
        limit: u32,
    },
    Tail {
        #[arg(short, long, default_value = "System")]
        channel: String,

        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        ndjson: bool,

        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to a Sigma YAML rule file or folder containing rules to evaluate
        #[arg(short = 's', long)]
        sigma_rules: Option<PathBuf>,

        /// Executable script/command to run on rule match (receives rule title and event ID as env vars)
        #[arg(long)]
        hook: Option<String>,

        /// Enable desktop notification banners when a Sigma rule triggers
        #[arg(long)]
        notify: bool,
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
