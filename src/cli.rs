use clap::{Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Text,
    Xml,
    Json,
    Ndjson,
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
}

impl Cli {
    /// Resolves explicit flags "--json" and "--ndjson" into "OutputFormat"
    pub fn resolved_format(&self) -> OutputFormat {
        if self.ndjson {
            OutputFormat::Ndjson
        } else if self.json {
            OutputFormat::Json
        } else {
            self.format
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
    },
}

impl Commands {
    pub fn resolved_tail_format(&self) -> OutputFormat {
        match self {
            Commands::Tail {
                format,
                json,
                ndjson,
                ..
            } => {
                if *ndjson {
                    OutputFormat::Ndjson
                } else if *json {
                    OutputFormat::Json
                } else {
                    *format
                }
            }
        }
    }
}
