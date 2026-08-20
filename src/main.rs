use anyhow::{Result, anyhow};
use std::path::Path;
use std::ptr;
use windows::Win32::System::EventLog::{
    EVT_HANDLE, EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryFilePath,
    EvtQueryReverseDirection,
};
use windows::core::PCWSTR;

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
    println!("Testing live channels & static .evtx file reader...\n");

    // 1. Test active channel
    let live_query = EventLogQuery::open_live_channel("System")?;
    let live_events = live_query.next_events(5)?;
    println!(
        "Live Channel ['{}']: Fetched {} events",
        live_query.source_name(),
        live_events.len()
    );

    // 2. Test reading a default system .evtx file directly from disk
    let system_evtx_path = r"C:\Windows\System32\winevt\Logs\System.evtx";

    match EventLogQuery::open_evtx_file(system_evtx_path) {
        Ok(file_query) => {
            let file_events = file_query.next_events(5)?;
            println!(
                "Static .evtx File ['{}']: Fetched {} events",
                file_query.source_name(),
                file_events.len()
            );
        }
        Err(err) => {
            println!("Could not read .evtx file: {}", err);
        }
    }

    Ok(())
}
