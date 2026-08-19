use anyhow::{Result, anyhow};
use std::ptr;
use windows::Win32::System::EventLog::{
    EVT_HANDLE, EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryReverseDirection,
};
use windows::core::PCWSTR;

/// Common live Windows Event Log channels
pub const LIVE_CHANNELS: &[&str] = &["System", "Security", "Application", "Setup"];

/// High-level wrapper around a Windows Event Log query handle.
pub struct EventLogQuery {
    handle: EVT_HANDLE,
    channel_name: String,
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
            channel_name: channel_name.to_string(),
        })
    }

    /// Returns name of the current open channel.
    pub fn channel_name(&self) -> &str {
        &self.channel_name
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

    // Test reading across standard live channels
    for &channel in LIVE_CHANNELS {
        match EventLogQuery::open_live_channel(channel) {
            Ok(query) => {
                let events = query.next_events(5)?;
                println!(
                    "Successfully queried '{}' live channel! (Fetched {} events)",
                    query.channel_name(),
                    events.len()
                )
            }
            Err(err) => {
                println!("Could not read '{}': {}", channel, err);
            }
        }
    }

    Ok(())
}
