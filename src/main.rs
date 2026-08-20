use anyhow::Result;
use winlog::record::EventRecord;
use winlog::win_api::EventLogSubscription;

fn main() -> Result<()> {
    println!("Subscribing to live System events... Press Ctrl+C to stop.");
    let sub = EventLogSubscription::subscribe("System")?;

    for xml in sub.receiver() {
        if let Ok(record) = EventRecord::from_xml(&xml) {
            record.print_formatted();
        }
    }

    Ok(())
}
