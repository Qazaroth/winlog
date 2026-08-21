use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub fn get_event_db() -> HashMap<u32, EventMetadata> {
    let mut db = HashMap::new();

    let events = vec![
        // --- Security / Authentication (46xx & 47xx) ---
        (
            4624,
            "Successful Logon",
            "Security",
            "An account was successfully logged on.",
        ),
        (
            4625,
            "Failed Logon",
            "Security",
            "An account failed to log on.",
        ),
        (
            4634,
            "Account Logoff",
            "Security",
            "An account was logged off.",
        ),
        (
            4647,
            "User Initiated Logoff",
            "Security",
            "User initiated logoff.",
        ),
        (
            4648,
            "Logon via Explicit Credentials",
            "Security",
            "A logon was attempted using explicit credentials (runas).",
        ),
        (
            4672,
            "Special Privileges Assigned",
            "Security",
            "Special privileges assigned to new logon (e.g., Administrator).",
        ),
        (
            4720,
            "User Account Created",
            "Security",
            "A user account was created.",
        ),
        (
            4722,
            "User Account Enabled",
            "Security",
            "A user account was enabled.",
        ),
        (
            4723,
            "Password Reset Attempt",
            "Security",
            "An attempt was made to change an account's password.",
        ),
        (
            4724,
            "Password Reset Success",
            "Security",
            "An account password was reset.",
        ),
        (
            4725,
            "User Account Disabled",
            "Security",
            "A user account was disabled.",
        ),
        (
            4726,
            "User Account Deleted",
            "Security",
            "A user account was deleted.",
        ),
        (
            4732,
            "Member Added to Local Group",
            "Security",
            "A member was added to a security-enabled local group.",
        ),
        (
            4733,
            "Member Removed from Local Group",
            "Security",
            "A member was removed from a security-enabled local group.",
        ),
        (
            4738,
            "User Account Modified",
            "Security",
            "A user account was modified.",
        ),
        (
            4740,
            "User Account Locked Out",
            "Security",
            "A user account was locked out.",
        ),
        (
            4768,
            "Kerberos TGT Requested",
            "Security",
            "A Kerberos authentication ticket (TGT) was requested.",
        ),
        (
            4769,
            "Kerberos Service Ticket Requested",
            "Security",
            "A Kerberos service ticket was requested.",
        ),
        (
            4771,
            "Kerberos Pre-Authentication Failed",
            "Security",
            "Kerberos pre-authentication failed.",
        ),
        (
            4776,
            "NTLM Authentication Attempt",
            "Security",
            "The domain controller attempted to validate credentials for an account.",
        ),
        // --- Process & Audit Tracking ---
        (
            4688,
            "New Process Created",
            "Process Tracking",
            "A new process has been created.",
        ),
        (
            4689,
            "Process Terminated",
            "Process Tracking",
            "A process has exited.",
        ),
        (
            4697,
            "Service Installed in System",
            "System",
            "A service was installed in the system.",
        ),
        (
            4703,
            "Token Rights Adjusted",
            "Security",
            "A token right was adjusted.",
        ),
        (
            4719,
            "System Audit Policy Changed",
            "Audit Policy",
            "System audit policy was changed.",
        ),
        (
            1102,
            "Audit Log Cleared",
            "Security",
            "The audit log was cleared.",
        ),
        // --- System & Kernel (1 - 100) ---
        (
            1,
            "System Time Changed",
            "System",
            "The system time was changed.",
        ),
        (
            6,
            "Kernel Driver Loaded",
            "System",
            "A kernel mode driver was loaded.",
        ),
        (
            12,
            "OS Startup",
            "System",
            "The operating system started up.",
        ),
        (
            13,
            "OS Shutdown",
            "System",
            "The operating system shut down.",
        ),
        (
            41,
            "Kernel Power Critical Failure",
            "Kernel Power",
            "System rebooted without cleanly shutting down first.",
        ),
        (
            42,
            "System Entering Sleep",
            "Kernel Power",
            "The system is entering sleep mode.",
        ),
        (
            1074,
            "System Shutdown/Restart Initiated",
            "User32",
            "A process or user initiated a system shutdown or restart.",
        ),
        (
            6005,
            "Event Log Service Started",
            "Event Log",
            "The Event Log service was started.",
        ),
        (
            6006,
            "Event Log Service Stopped",
            "Event Log",
            "The Event Log service was stopped.",
        ),
        (
            6008,
            "Unexpected System Shutdown",
            "Event Log",
            "The previous system shutdown was unexpected.",
        ),
        (
            7030,
            "Service Marked Interactive",
            "Service Control Manager",
            "A service was configured as interactive.",
        ),
        (
            7034,
            "Service Terminated Unexpectedly",
            "Service Control Manager",
            "A service terminated unexpectedly.",
        ),
        (
            7036,
            "Service State Changed",
            "Service Control Manager",
            "A service entered a running or stopped state.",
        ),
        (
            7040,
            "Service Start Type Changed",
            "Service Control Manager",
            "A service start type was changed.",
        ),
        (
            7045,
            "New Service Installed",
            "Service Control Manager",
            "A new service was installed on the system.",
        ),
        // --- Sysmon (System Monitor) ---
        (
            1001,
            "Sysmon: Process Create",
            "Sysmon",
            "Process creation event.",
        ),
        (
            1002,
            "Sysmon: Process Change Time",
            "Sysmon",
            "A process changed a file creation time.",
        ),
        (
            1003,
            "Sysmon: Network Connection",
            "Sysmon",
            "Network connection detected.",
        ),
        (
            1005,
            "Sysmon: Driver Loaded",
            "Sysmon",
            "Driver loaded by process.",
        ),
        (
            1007,
            "Sysmon: Image Loaded",
            "Sysmon",
            "Image/DLL loaded in process.",
        ),
        (
            1008,
            "Sysmon: CreateRemoteThread",
            "Sysmon",
            "Process created a thread in another process.",
        ),
        (
            1010,
            "Sysmon: ProcessAccess",
            "Sysmon",
            "Process opened another process.",
        ),
        (
            1011,
            "Sysmon: FileCreate",
            "Sysmon",
            "File was created or overwritten.",
        ),
        (
            1012,
            "Sysmon: RegistryEvent (Value Set)",
            "Sysmon",
            "Registry object value set.",
        ),
        (
            1013,
            "Sysmon: RegistryEvent (Key/Value Rename)",
            "Sysmon",
            "Registry key or value renamed.",
        ),
        (
            1015,
            "Sysmon: FileCreateStreamHash",
            "Sysmon",
            "Named file stream created.",
        ),
        (
            1017,
            "Sysmon: PipeEvent (Created)",
            "Sysmon",
            "Named pipe created.",
        ),
        (
            1018,
            "Sysmon: PipeEvent (Connected)",
            "Sysmon",
            "Named pipe connected.",
        ),
        (
            1022,
            "Sysmon: DNS Query",
            "Sysmon",
            "DNS query was executed.",
        ),
        // --- Windows Defender & Security Center ---
        (
            1000,
            "Application Error / Crash",
            "Application",
            "Faulting application name, version, or module crash.",
        ),
        (
            1001,
            "Windows Error Reporting",
            "Application",
            "Fault bucket or error report submitted.",
        ),
        (
            1116,
            "Defender Malware Detected",
            "Windows Defender",
            "Malware or unwanted software was detected.",
        ),
        (
            1117,
            "Defender Action Taken",
            "Windows Defender",
            "Action taken against detected malware.",
        ),
        (
            1118,
            "Defender Remediation Failed",
            "Windows Defender",
            "Failed to take action against malware.",
        ),
        (
            5000,
            "Defender Real-Time Protection Disabled",
            "Windows Defender",
            "Real-time protection was disabled.",
        ),
        (
            5001,
            "Defender Real-Time Protection Enabled",
            "Windows Defender",
            "Real-time protection was enabled.",
        ),
        // --- Active Directory & Domain Services ---
        (
            5136,
            "Directory Service Object Modified",
            "Directory Services",
            "A directory service object was modified.",
        ),
        (
            5137,
            "Directory Service Object Created",
            "Directory Services",
            "A directory service object was created.",
        ),
        (
            5138,
            "Directory Service Object Undeleted",
            "Directory Services",
            "A directory service object was undeleted.",
        ),
        (
            5139,
            "Directory Service Object Moved",
            "Directory Services",
            "A directory service object was moved.",
        ),
        (
            5140,
            "Network Share Accessed",
            "File Share",
            "A network share object was accessed.",
        ),
        (
            5142,
            "Network Share Added",
            "File Share",
            "A network share object was added.",
        ),
        (
            5145,
            "Network Share Detailed Check",
            "File Share",
            "Network share object check for access rights.",
        ),
        (
            5156,
            "Windows Filtering Platform Allowed Connection",
            "WFP",
            "WFP permitted a connection.",
        ),
        (
            5157,
            "Windows Filtering Platform Blocked Connection",
            "WFP",
            "WFP blocked a connection.",
        ),
        // --- Terminal Services / Remote Desktop (RDP) ---
        (
            21,
            "RDP Session Logon Successful",
            "TerminalServices",
            "Remote Desktop Services: Session logon succeeded.",
        ),
        (
            22,
            "RDP Shell Start Notification",
            "TerminalServices",
            "Remote Desktop Services: Shell start notification received.",
        ),
        (
            24,
            "RDP Session Disconnected",
            "TerminalServices",
            "Remote Desktop Services: Session has been disconnected.",
        ),
        (
            25,
            "RDP Session Reconnected",
            "TerminalServices",
            "Remote Desktop Services: Session reconnected.",
        ),
        (
            1149,
            "RDP User Authentication Succeeded",
            "TerminalServices",
            "User authentication succeeded for RDP.",
        ),
        // --- PowerShell & Script Execution ---
        (
            4103,
            "PowerShell Module Logging",
            "PowerShell",
            "PowerShell module engine activity logged.",
        ),
        (
            4104,
            "PowerShell Script Block Logging",
            "PowerShell",
            "PowerShell script block execution text captured.",
        ),
        (
            800,
            "PowerShell Pipeline Execution",
            "PowerShell",
            "Pipeline execution detail.",
        ),
        // --- AppLocker & Software Restriction ---
        (
            8002,
            "AppLocker Executable Allowed",
            "AppLocker",
            "AppLocker policy allowed binary execution.",
        ),
        (
            8004,
            "AppLocker Executable Blocked",
            "AppLocker",
            "AppLocker policy blocked binary execution.",
        ),
        (
            8007,
            "AppLocker Script Blocked",
            "AppLocker",
            "AppLocker policy blocked script execution.",
        ),
        // --- Windows Update & Installer ---
        (
            19,
            "Windows Update Success",
            "WindowsUpdateClient",
            "Installation Successful: Windows successfully installed update.",
        ),
        (
            20,
            "Windows Update Failure",
            "WindowsUpdateClient",
            "Installation Failure: Windows failed to install update.",
        ),
        (
            1033,
            "MsiInstaller Provider Installed",
            "MsiInstaller",
            "Windows Installer installed an application.",
        ),
        (
            1034,
            "MsiInstaller Provider Removed",
            "MsiInstaller",
            "Windows Installer removed an application.",
        ),
        // --- Disk & Storage ---
        (7, "Disk Block Error", "Disk", "Bad block on disk device."),
        (
            11,
            "Disk Controller Error",
            "Disk",
            "The driver detected a controller error on device.",
        ),
        (
            51,
            "Disk Paging Error",
            "Disk",
            "An error was detected on device during a paging operation.",
        ),
        (
            157,
            "Disk Surprisingly Removed",
            "Disk",
            "Disk has been surprisingly removed.",
        ),
        // --- Task Scheduler ---
        (
            100,
            "Task Started",
            "TaskScheduler",
            "Task Scheduler started an instance of a task.",
        ),
        (
            102,
            "Task Completed",
            "TaskScheduler",
            "Task Scheduler successfully finished an instance.",
        ),
        (
            106,
            "Task Registered",
            "TaskScheduler",
            "User registered a Scheduled Task.",
        ),
        (
            140,
            "Task Updated",
            "TaskScheduler",
            "User updated a Scheduled Task.",
        ),
        (
            141,
            "Task Deleted",
            "TaskScheduler",
            "User deleted a Scheduled Task.",
        ),
    ];

    for (id, title, category, description) in events {
        db.insert(
            id,
            EventMetadata {
                title,
                category,
                description,
            },
        );
    }

    db
}
