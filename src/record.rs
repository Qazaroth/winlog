use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use colored::*;
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventLevel {
    Critical,
    Error,
    Warning,
    Information,
    Verbose,
    Unknown(u8),
}

impl From<u8> for EventLevel {
    fn from(level: u8) -> Self {
        match level {
            1 => EventLevel::Critical,
            2 => EventLevel::Error,
            3 => EventLevel::Warning,
            4 => EventLevel::Information,
            5 => EventLevel::Verbose,
            other => EventLevel::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: u32,
    pub provider: String,
    pub channel: String,
    pub level: EventLevel,
    pub timestamp: Option<DateTime<Utc>>,
    pub computer: String,
    pub process_id: Option<u32>,
    pub thread_id: Option<u32>,
    pub payload: Vec<(String, String)>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub raw_xml: String,
}

impl EventRecord {
    /// Serializes the record into CSV format.
    pub fn write_csv_row<W: Write>(&self, wtr: &mut Writer<W>) -> anyhow::Result<()> {
        let timestamp_str = self.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default();
        let payload_str = self
            .payload
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        wtr.write_record(&[
            self.event_id.to_string(),
            format!("{:?}", self.level),
            self.provider.clone(),
            self.channel.clone(),
            timestamp_str,
            self.computer.clone(),
            self.process_id.map(|p| p.to_string()).unwrap_or_default(),
            self.thread_id.map(|t| t.to_string()).unwrap_or_default(),
            payload_str,
        ])?;

        wtr.flush()?;

        Ok(())
    }

    /// Writes CSV header line
    pub fn write_csv_header<W: Write>(wtr: &mut Writer<W>) -> anyhow::Result<()> {
        wtr.write_record(&[
            "EventID",
            "Level",
            "Provider",
            "Channel",
            "Timestamp",
            "Computer",
            "ProcessID",
            "ThreadID",
            "Payload",
        ])?;
        wtr.flush()?;
        Ok(())
    }

    pub fn colored_level_str(&self) -> ColoredString {
        match self.level {
            EventLevel::Critical => "CRITICAL".red().bold().reversed(),
            EventLevel::Error => "ERROR".red().bold(),
            EventLevel::Warning => "WARN".yellow().bold(),
            EventLevel::Information => "INFO".green().bold(),
            EventLevel::Verbose => "VERBOSE".blue(),
            EventLevel::Unknown(_) => "UNKNOWN".normal(),
        }
    }

    pub fn print_formatted(&self) {
        let time_str = self
            .timestamp
            .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "[{}] [{}] [{}] ID:{}: {}",
            time_str.dimmed(),
            self.colored_level_str(),
            self.provider.cyan(),
            self.event_id.to_string().bold(),
            self.channel.dimmed()
        );

        for (k, v) in &self.payload {
            if !v.trim().is_empty() {
                println!("    {} {}: {}", "↳".dimmed(), k.bold(), v);
            }
        }
    }

    pub fn from_xml(xml_str: &str) -> Result<Self> {
        let doc = roxmltree::Document::parse(xml_str)
            .map_err(|e| anyhow!("Failed to parse XML document: {}", e))?;
        let root = doc.root_element();
        let system_node = root
            .children()
            .find(|n| n.has_tag_name("System"))
            .ok_or_else(|| anyhow!("Missing <System> block in Event XML"))?;

        let event_id = system_node
            .children()
            .find(|n| n.has_tag_name("EventID"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let provider = system_node
            .children()
            .find(|n| n.has_tag_name("Provider"))
            .and_then(|n| n.attribute("Name"))
            .unwrap_or("Unknown")
            .to_string();

        let channel = system_node
            .children()
            .find(|n| n.has_tag_name("Channel"))
            .and_then(|n| n.text())
            .unwrap_or("Unknown")
            .to_string();

        let level_u8 = system_node
            .children()
            .find(|n| n.has_tag_name("Level"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(4);
        let level = EventLevel::from(level_u8);

        let timestamp = system_node
            .children()
            .find(|n| n.has_tag_name("TimeCreated"))
            .and_then(|n| n.attribute("SystemTime"))
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let computer = system_node
            .children()
            .find(|n| n.has_tag_name("Computer"))
            .and_then(|n| n.text())
            .unwrap_or("localhost")
            .to_string();

        let (process_id, thread_id) = system_node
            .children()
            .find(|n| n.has_tag_name("Execution"))
            .map(|n| {
                let pid = n.attribute("ProcessID").and_then(|s| s.parse::<u32>().ok());
                let tid = n.attribute("ThreadID").and_then(|s| s.parse::<u32>().ok());
                (pid, tid)
            })
            .unwrap_or((None, None));

        let mut payload = Vec::new();
        if let Some(event_data_node) = root.children().find(|n| n.has_tag_name("EventData")) {
            for data_node in event_data_node
                .children()
                .filter(|n| n.has_tag_name("Data"))
            {
                let key = data_node.attribute("Name").unwrap_or("Data").to_string();
                let val = data_node.text().unwrap_or("").to_string();
                payload.push((key, val));
            }
        }

        Ok(EventRecord {
            event_id,
            provider,
            channel,
            level,
            timestamp,
            computer,
            process_id,
            thread_id,
            payload,
            raw_xml: xml_str.to_string(),
        })
    }

    pub fn level_str(&self) -> &str {
        match self.level {
            EventLevel::Critical => "CRITICAL",
            EventLevel::Error => "ERROR",
            EventLevel::Warning => "WARNING",
            EventLevel::Information => "INFO",
            EventLevel::Verbose => "VERBOSE",
            EventLevel::Unknown(_) => "UNKNOWN",
        }
    }
}
