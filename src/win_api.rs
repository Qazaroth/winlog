use anyhow::{Result, anyhow};
use std::path::Path;
use std::ptr;
use windows::Win32::System::EventLog::{
    EVT_HANDLE, EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryFilePath,
    EvtQueryReverseDirection, EvtRender, EvtRenderEventXml,
};
use windows::core::PCWSTR;

pub const LIVE_CHANNELS: &[&str] = &["System", "Security", "Application", "Setup"];

/// High-level wrapper around a Windows Event Log query handle.
pub struct EventLogQuery {
    handle: EVT_HANDLE,
    source_name: String,
}

impl EventLogQuery {
    pub fn open_path_or_channel(input: &str) -> Result<Self> {
        let path = Path::new(input);
        if path.exists() && path.extension().and_then(|s| s.to_str()) == Some("evtx") {
            Self::open_evtx_file(path)
        } else {
            Self::open_live_channel(input)
        }
    }

    pub fn open_live_channel(channel_name: &str) -> Result<Self> {
        let wide_channel: Vec<u16> = channel_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            EvtQuery(
                None,
                PCWSTR(wide_channel.as_ptr()),
                PCWSTR(ptr::null()),
                EvtQueryChannelPath.0 | EvtQueryReverseDirection.0,
            )
            .map_err(|e| anyhow!("Failed to open live channel '{}': {}. (Note: 'Security' requires Administrator privileges)", channel_name, e))?
        };

        Ok(Self {
            handle,
            source_name: channel_name.to_string(),
        })
    }

    pub fn open_evtx_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path_ref = file_path.as_ref();
        if !path_ref.exists() {
            return Err(anyhow!("The file path '{:?}' does not exist.", path_ref));
        }

        let canonical_path = path_ref.canonicalize()?;
        let path_str = canonical_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid UTF-8 in file path"))?;

        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        let wide_path: Vec<u16> = clean_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

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

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn next_events(&self, batch_size: u32) -> Result<Vec<EventHandle>> {
        let mut raw_events: Vec<isize> = vec![0; batch_size as usize];
        let mut returned: u32 = 0;

        let status = unsafe { EvtNext(self.handle, &mut raw_events, 1000, 0, &mut returned) };

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
    pub fn to_xml(&self) -> Result<String> {
        let mut buffer_used: u32 = 0;
        let mut property_count: u32 = 0;

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

        let mut buffer: Vec<u16> = vec![0; (buffer_used / 2) as usize];

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
