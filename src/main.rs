use anyhow::Result;
use clap::Parser;
use colored::*;
use csv::WriterBuilder;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winlog::cli::{Cli, Commands, OutputFormat};
use winlog::record::EventRecord;
use winlog::win_api::{EventLogQuery, EventLogSubscription};

fn create_writer(output_path: Option<&PathBuf>) -> Result<Box<dyn Write>> {
    match output_path {
        Some(path) => {
            let file = File::create(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

fn run_static_query(
    input: &str,
    limit: u32,
    format: OutputFormat,
    output_path: Option<&PathBuf>,
) -> Result<()> {
    let query = EventLogQuery::open_path_or_channel(input)?;
    let raw_events = query.next_events(limit)?;

    let mut writer = create_writer(output_path)?;

    match format {
        OutputFormat::Json => {
            let mut records = Vec::new();
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    records.push(record);
                }
            }
            serde_json::to_writer_pretty(&mut writer, &records)?;
            writeln!(writer)?;
        }
        OutputFormat::Ndjson => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    serde_json::to_writer(&mut writer, &record)?;
                    writeln!(writer)?;
                }
            }
        }
        OutputFormat::Csv => {
            let mut wtr = WriterBuilder::new().from_writer(writer);
            EventRecord::write_csv_header(&mut wtr)?;
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    record.write_csv_row(&mut wtr)?;
                }
            }
        }
        OutputFormat::Xml => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                writeln!(writer, "{}\n---", xml)?;
            }
        }
        OutputFormat::Text => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    record.print_formatted();
                }
            }
        }
    }

    if let Some(path) = output_path {
        println!(
            "{} Exported events to {}",
            "✔".green().bold(),
            path.display().to_string().cyan()
        );
    }

    Ok(())
}

fn run_tail_stream(
    channel: &str,
    format: OutputFormat,
    output_path: Option<&PathBuf>,
) -> Result<()> {
    if output_path.is_none() && format == OutputFormat::Text {
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
    let writer = create_writer(output_path)?;

    match format {
        OutputFormat::Csv => {
            let mut wtr = WriterBuilder::new().from_writer(writer);
            EventRecord::write_csv_header(&mut wtr)?;

            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        record.write_csv_row(&mut wtr)?;
                    }
                }
            }
        }
        OutputFormat::Ndjson | OutputFormat::Json => {
            let mut writer = writer;
            let mut first = true;
            writeln!(writer, "[")?;

            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        if !first {
                            writeln!(writer, ",")?;
                        }
                        serde_json::to_writer(&mut writer, &record)?;
                        writer.flush()?;
                        first = false;
                    }
                }
            }

            writeln!(writer, "\n]")?;
            writer.flush()?;
        }
        OutputFormat::Xml => {
            let mut writer = writer;
            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    writeln!(writer, "{}\n---", xml)?;
                    writer.flush()?;
                }
            }
        }
        OutputFormat::Text => {
            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        record.print_formatted();
                    }
                }
            }
        }
    }

    if let Some(path) = output_path {
        println!(
            "\n{} Stopped tailing. Saved logs to {}",
            "✔".green().bold(),
            path.display().to_string().cyan()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(
            cmd @ Commands::Tail {
                channel, output, ..
            },
        ) => {
            run_tail_stream(channel, cmd.resolved_tail_format(), output.as_ref())?;
        }
        None => {
            run_static_query(
                &cli.channel,
                cli.limit,
                cli.resolved_format(),
                cli.output.as_ref(),
            )?;
        }
    }

    Ok(())
}
