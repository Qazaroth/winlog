#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HLOCAL;
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::{
    FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
    FormatMessageW,
};
#[cfg(target_os = "windows")]
use windows::core::PWSTR;

/// Resolves an error code (HRESULT, NTSTATUS, or Win32 Error) to a human-readable description.
pub fn parse_win32_error(code: u32) -> Option<String> {
    // 1. Convert HRESULT facility codes (e.g., 0x80070005 -> Win32 Error 5)
    let win32_code = if (code & 0xFFFF0000) == 0x80070000 {
        code & 0xFFFF
    } else {
        code
    };

    #[cfg(target_os = "windows")]
    {
        let mut buffer: *mut u16 = std::ptr::null_mut();

        unsafe {
            let flags = FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS;

            let res = FormatMessageW(
                flags,
                None,
                win32_code,
                0,
                PWSTR(&mut buffer as *mut _ as *mut u16),
                0,
                None,
            );

            if res > 0 && !buffer.is_null() {
                let slice = std::slice::from_raw_parts(buffer, res as usize);
                let message = String::from_utf16_lossy(slice)
                    .trim()
                    .replace("\r\n", " ")
                    .replace('\n', " ");

                // Free allocated memory from FormatMessageW
                let _ = windows::Win32::Foundation::LocalFree(Some(HLOCAL(buffer as *mut _)));

                if !message.is_empty() {
                    return Some(message);
                }
            }
        }
    }

    // 2. Fallback static map for standard codes (and non-Windows targets)
    match code {
        0x00000005 | 0x80070005 | 0xC0000022 => Some("Access is denied.".to_string()),
        0x00000002 | 0x80070002 | 0xC000000F => {
            Some("The system cannot find the file specified.".to_string())
        }
        0x00000003 | 0x80070003 => Some("The system cannot find the path specified.".to_string()),
        0x00000032 | 0x80070032 => Some("The network request is not supported.".to_string()),
        0x00000057 | 0x80070057 => Some("The parameter is incorrect.".to_string()),
        0x8007000E | 0xC0000017 => {
            Some("Not enough storage is available to process this command.".to_string())
        }
        0xC000006D => Some("Logon failure: unknown user name or bad password.".to_string()),
        0xC000006E => Some("Logon failure: user account restriction.".to_string()),
        0xC0000072 => Some("Logon failure: account currently disabled.".to_string()),
        0xC0000133 => Some("Clocks between DC and target machine are out of sync.".to_string()),
        _ => None,
    }
}
