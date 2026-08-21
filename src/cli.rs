use std::process::Command;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Text,
    Xml,
}

/// A fast, modern CLI and TUI alternative to Windows Event Viewer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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
    },
}
