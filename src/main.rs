use anyhow::Result;
use clap::Parser;
use colored::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winlog::cli::{Cli, Commands, OutputFormat};
use winlog::record::EventRecord;
use winlog::win_api::{EventLogQuery, EventLogSubscription};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Tail { channel, format }) => {
            run_tail_stream(channel, *format)?;
        }
        None => {
            run_static_query(&cli.channel, cli.limit, cli.format)?;
        }
    }

    Ok(())
}

fn run_tail_stream(channel: &str, format: OutputFormat) -> Result<()> {
    println!(
        "{} Streaming events from channel '{}'. Press {} to stop.\n",
        "●".green().bold(),
        channel.cyan(),
        "Ctrl+C".bold()
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up graceful shutdown on Ctrl+C
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let sub = EventLogSubscription::subscribe(channel)?;
    let receiver = sub.receiver();

    while running.load(Ordering::SeqCst) {
        // Timeout check every 200ms to allow smooth response to Ctrl+C
        if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
            match format {
                OutputFormat::Xml => {
                    println!("{}\n---", xml);
                }
                OutputFormat::Text => {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        record.print_formatted();
                    }
                }
            }
        }
    }

    println!("\n{}", "Stopped streaming.".yellow());
    Ok(())
}

fn run_static_query(input: &str, limit: u32, format: OutputFormat) -> Result<()> {
    let query = EventLogQuery::open_path_or_channel(input)?;
    let raw_events = query.next_events(limit)?;

    for handle in raw_events {
        let xml = handle.to_xml()?;

        match format {
            OutputFormat::Xml => println!("{}\n---", xml),
            OutputFormat::Text => {
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    record.print_formatted();
                }
            }
        }
    }

    Ok(())
}
