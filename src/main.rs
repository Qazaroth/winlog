use anyhow::Result;
use clap::Parser;
use winlog::cli::{Cli, OutputFormat};
use winlog::record::EventRecord;
use winlog::win_api::EventLogQuery;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let query = EventLogQuery::open_path_or_channel(&cli.channel)?;
    let raw_events = query.next_events(cli.limit)?;

    for handle in raw_events {
        let xml = handle.to_xml()?;

        match cli.format {
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
