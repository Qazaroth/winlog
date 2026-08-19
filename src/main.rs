use anyhow::Result;
use std::ptr;
use windows::Win32::System::EventLog::{
    EVT_HANDLE, EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryReverseDirection,
};
use windows::core::PCWSTR;

/// High-level wrapper around a Windows Event Log query handle.
pub struct EventLogQuery {
    handle: EVT_HANDLE,
}

impl EventLogQuery {
    /// Opens a query handle for a given channel (eg "System") or .evtx file path.
    pub fn open_channel(channel_name: &str) -> Result<Self> {
        // Convert Rust string slice (&str) into a null-terminated UTF-16 vector for Windows APIs
        let wide_channel: Vec<u16> = channel_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            EvtQuery(
                None,
                PCWSTR(wide_channel.as_ptr()),
                PCWSTR(ptr::null()), // XPath query string (NULL = fetch all)
                EvtQueryChannelPath.0 | EvtQueryReverseDirection.0, // Read newest events first
            )?
        };

        Ok(Self { handle })
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
    println!("Testing EvtQuery and EvtNext wrapper...");

    let query = EventLogQuery::open_channel("System")?;
    println!("Successfully queried 'System' channel!");

    // Added '?' here to unwrap Result<Vec<EventHandle>> into Vec<EventHandle>
    let events = query.next_events(5)?;
    println!("Retrieved {} raw event handles.", events.len());

    Ok(())
}
