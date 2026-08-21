use anyhow::Result;
use clap::Parser;
use colored::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winlog::cli::{Cli, Commands, OutputFormat};
use winlog::record::EventRecord;
use winlog::win_api::{EventLogQuery, EventLogSubscription};

fn print_record(record: &EventRecord, format: OutputFormat, is_first: &mut bool) {
    match format {
        OutputFormat::Text => record.print_formatted(),
        OutputFormat::Xml => unreachable!("XML rendered separately."),
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(record) {
                if !*is_first {
                    println!(",");
                }
                print!("{}", json);
                *is_first = false;
            }
        }
        OutputFormat::Ndjson => {
            if let Ok(json) = serde_json::to_string(record) {
                println!("{}", json);
            }
        }
    }
}

fn run_tail_stream(channel: &str, format: OutputFormat) -> Result<()> {
    if format == OutputFormat::Text {
        println!(
            "{} Streaming events from channel '{}'. Press {} to stop.\n",
            "●".green().bold(),
            channel.cyan(),
            "Ctrl+C".bold()
        );
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let sub = EventLogSubscription::subscribe(channel)?;
    let receiver = sub.receiver();

    let mut is_first = true;
    if format == OutputFormat::Json {
        println!("[");
    }

    while running.load(Ordering::SeqCst) {
        if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
            if format == OutputFormat::Xml {
                println!("{}\n---", xml);
            } else if let Ok(record) = EventRecord::from_xml(&xml) {
                print_record(&record, format, &mut is_first);
            }
        }
    }

    if format == OutputFormat::Json {
        println!("\n]");
    } else if format == OutputFormat::Text {
        println!("\n{}", "Stopped streaming.".yellow());
    }

    Ok(())
}

fn run_static_query(input: &str, limit: u32, format: OutputFormat) -> Result<()> {
    let query = EventLogQuery::open_path_or_channel(input)?;
    let raw_events = query.next_events(limit)?;

    let mut is_first = true;
    if format == OutputFormat::Json {
        println!("[");
    }

    for handle in raw_events {
        let xml = handle.to_xml()?;

        if format == OutputFormat::Xml {
            println!("{}\n---", xml);
        } else if let Ok(record) = EventRecord::from_xml(&xml) {
            print_record(&record, format, &mut is_first);
        }
    }

    if format == OutputFormat::Json {
        println!("\n]");
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd @ Commands::Tail { channel, .. }) => {
            run_tail_stream(channel, cmd.resolved_tail_format())?;
        }
        None => {
            run_static_query(&cli.channel, cli.limit, cli.resolved_format())?;
        }
    }

    Ok(())
}
