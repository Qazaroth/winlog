use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::ptr;
use windows::Win32::System::EventLog::{
    EVT_HANDLE, EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryFilePath,
    EvtQueryReverseDirection,
};
use windows::Win32::System::EventLog::{EVT_RENDER_FLAGS, EvtRender, EvtRenderEventXml};
use windows::core::PCWSTR;

/// Log severity levels mapped from Windows Event Log Level IDs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventLevel {
    Critical,    // Level 1
    Error,       // Level 2
    Warning,     // Level 3
    Information, // Level 4
    Verbose,     // Level 5
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

/// Represents a fully parsed, strongly-typed Windows Event Record
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event_id: u32,
    pub provider: String,
    pub channel: String,
    pub level: EventLevel,
    pub timestamp: Option<DateTime<Utc>>,
    pub computer: String,
    pub process_id: Option<u32>,
    pub thread_id: Option<u32>,
    /// Key-value event payload data or event parameters
    pub payload: Vec<(String, String)>,
    /// Raw XML payload retained for detailed view / debugging
    pub raw_xml: String,
}

impl EventRecord {
    /// Parses a raw XML string into a strongly-typed "EventRecord"
    pub fn from_xml(xml_str: &str) -> Result<Self> {
        let doc = roxmltree::Document::parse(xml_str)
            .map_err(|e| anyhow!("Failed to parse XML document: {}", e))?;
        let root = doc.root_element();
        let system_node = root
            .children()
            .find(|n| n.has_tag_name("System"))
            .ok_or_else(|| anyhow!("Missing <System> block in Event XML"))?;

        // 1. Event ID
        let event_id = system_node
            .children()
            .find(|n| n.has_tag_name("EventID"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        // 2. Provider name
        let provider = system_node
            .children()
            .find(|n| n.has_tag_name("Provider"))
            .and_then(|n| n.attribute("Name"))
            .unwrap_or("Unknown")
            .to_string();

        // 3. Channel
        let channel = system_node
            .children()
            .find(|n| n.has_tag_name("Channel"))
            .and_then(|n| n.text())
            .unwrap_or("Unknown")
            .to_string();

        // 4. Level
        let level_u8 = system_node
            .children()
            .find(|n| n.has_tag_name("Level"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(4); // default to info (4)
        let level = EventLevel::from(level_u8);

        // 5. Timestamp (SystemTime attribute in <TimeCreated>)
        let timestamp = system_node
            .children()
            .find(|n| n.has_tag_name("TimeCreated"))
            .and_then(|n| n.attribute("SystemTime"))
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // 6. Computer Name
        let computer = system_node
            .children()
            .find(|n| n.has_tag_name("Computer"))
            .and_then(|n| n.text())
            .unwrap_or("localhost")
            .to_string();

        // 7. Execution (ProcessID & ThreadID)
        let (process_id, thread_id) = system_node
            .children()
            .find(|n| n.has_tag_name("Execution"))
            .map(|n| {
                let pid = n.attribute("ProcessID").and_then(|s| s.parse::<u32>().ok());
                let tid = n.attribute("ThreadID").and_then(|s| s.parse::<u32>().ok());
                (pid, tid)
            })
            .unwrap_or((None, None));

        // 8. EventData Payload Parameters
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

    /// Helper to format the level as a clean uppercase string for display/filtering
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

/// Common live Windows Event Log channels
pub const LIVE_CHANNELS: &[&str] = &["System", "Security", "Application", "Setup"];

/// High-level wrapper around a Windows Event Log query handle.
pub struct EventLogQuery {
    handle: EVT_HANDLE,
    source_name: String,
}

impl EventLogQuery {
    /// Opens a query handle for an active live Windows channel (e.g., "System", "Security", "Application").
    pub fn open_live_channel(channel_name: &str) -> Result<Self> {
        // Convert Rust string slice (&str) into a null-terminated UTF-16 vector for Windows APIs
        let wide_channel: Vec<u16> = channel_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // EvtQueryChannelPath specifies we are querying an active live system channel
        let handle = unsafe {
            EvtQuery(
                None,
                PCWSTR(wide_channel.as_ptr()),
                PCWSTR(ptr::null()),
                EvtQueryChannelPath.0 | EvtQueryReverseDirection.0,
            )
            .map_err(|e| anyhow!("Failed to open live channel '{}': {}. (Note: 'Security' channel requires Administrator privileges)", channel_name, e))?
        };

        Ok(Self {
            handle,
            source_name: channel_name.to_string(),
        })
    }

    /// Opens query handle for an offline static ".evtx" file.
    pub fn open_evtx_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path_ref = file_path.as_ref();

        if !path_ref.exists() {
            return Err(anyhow!("The file path '{:?}' does not exist.", path_ref));
        }

        let canonical_path = path_ref.canonicalize()?;
        let path_str = canonical_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid UTF-8 in file path"))?;

        // Strip extended length path prefix (\\?\) if present, as winevt prefers standard wide paths
        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);

        let wide_path: Vec<u16> = clean_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // Pass EvtQueryFilePath flag to instruct winevt to parse an offline file
        let handle = unsafe {
            EvtQuery(
                None,
                PCWSTR(wide_path.as_ptr()),
                PCWSTR(ptr::null()),
                EvtQueryFilePath.0 | EvtQueryReverseDirection.0,
            )
            .map_err(|e| anyhow!("Failed to parse static .evtx file '{:?}': {}", path_ref, e))?
        };

        Ok(Self {
            handle,
            source_name: path_ref.display().to_string(),
        })
    }

    /// Returns name of the current open channel.
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    /// Fetches next batch of raw event handles
    pub fn next_events(&self, batch_size: u32) -> Result<Vec<EventHandle>> {
        let mut raw_events: Vec<isize> = vec![0; batch_size as usize];
        let mut returned: u32 = 0;

        let status = unsafe { EvtNext(self.handle, &mut raw_events, 1000, 0, &mut returned) };

        // If EvtNext returned an error or retrieved 0 events, return an empty vector
        if status.is_err() || returned == 0 {
            return Ok(Vec::new());
        }

        raw_events.truncate(returned as usize);

        Ok(raw_events
            .into_iter()
            .map(|h| EventHandle(EVT_HANDLE(h)))
            .collect())
    }
}

impl Drop for EventLogQuery {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = EvtClose(self.handle);
            }
        }
    }
}

pub struct EventHandle(pub EVT_HANDLE);

impl EventHandle {
    /// Renders raw event handle into an XML string using EvtRender
    pub fn to_xml(&self) -> Result<String> {
        let mut buffer_used: u32 = 0;
        let mut property_count: u32 = 0;

        // First call: pass 0/null to retrieve the required buffer size
        unsafe {
            let _ = EvtRender(
                None,
                self.0,
                EvtRenderEventXml.0,
                0,
                None,
                &mut buffer_used,
                &mut property_count,
            );
        }

        if buffer_used == 0 {
            return Err(anyhow!("Failed to determine buffer size for EvtRender"));
        }

        // Allocate a UTF-16 buffer of appropriate size
        let mut buffer: Vec<u16> = vec![0; (buffer_used / 2) as usize];

        // Second call: Populate buffer with actual XML content
        unsafe {
            EvtRender(
                None,
                self.0,
                EvtRenderEventXml.0,
                buffer_used,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut buffer_used,
                &mut property_count,
            )?;
        }

        // Trim null chars and parse UTF-16 slice into Rust String
        let xml_len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        String::from_utf16(&buffer[..xml_len])
            .map_err(|e| anyhow!("Failed to convert UTF-16 XML buffer: {}", e))
    }
}

impl Drop for EventHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = EvtClose(self.0);
            }
        }
    }
}

fn main() -> Result<()> {
    println!("Testing XML rendering & parsing...\n");

    let query = EventLogQuery::open_live_channel("System")?;
    let raw_events = query.next_events(3)?;

    for (idx, handle) in raw_events.iter().enumerate() {
        let xml = handle.to_xml()?;
        let record = EventRecord::from_xml(&xml)?;

        println!("--- Event #{} ---", idx + 1);
        println!("ID:        {}", record.event_id);
        println!("Provider:  {}", record.provider);
        println!("Level:     {}", record.level_str());
        println!("Time:      {:?}", record.timestamp);
        println!("Computer:  {}", record.computer);
        println!("Payloads:  {} parameter(s)", record.payload.len());
        println!();
    }

    Ok(())
}
