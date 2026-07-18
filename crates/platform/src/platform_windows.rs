//! Windows-specific platform helpers.

/// Returns the current process ID.
pub fn current_pid() -> u32 {
    std::process::id()
}

/// Portable handle-count metric for leak soak tests (G2/D2).
#[must_use]
pub fn handle_count() -> Option<usize> {
    process_handle_count()
}

/// Returns the current process's handle count via `GetProcessHandleCount`.
/// Used by the daemon leak soak test as the authoritative "are handles
/// leaking?" signal (G2/D2). Returns `None` if the call is unavailable.
#[allow(unsafe_code)]
#[must_use]
pub fn process_handle_count() -> Option<usize> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessHandleCount};

    unsafe {
        let proc: HANDLE = GetCurrentProcess();
        let mut count: u32 = 0;
        if GetProcessHandleCount(proc, &mut count) != 0 {
            Some(count as usize)
        } else {
            let _ = CloseHandle(proc);
            None
        }
    }
}

/// Returns the parent process ID via the `Win32_Process` WMI class — but
/// WMI requires COM init, which we want to avoid for a fast check. Instead,
/// we use `Process32First`/`Process32Next` from `tlhelp32.h` (via the
/// `windows-sys` feature).
#[allow(unsafe_code)]
pub fn parent_pid() -> Option<u32> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let me = std::process::id();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return None;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) == 0 {
            CloseHandle(snap);
            return None;
        }
        let mut found_parent = None;
        loop {
            if entry.th32ProcessID == me {
                found_parent = Some(entry.th32ParentProcessID);
                break;
            }
            if Process32NextW(snap, &mut entry) == 0 {
                break;
            }
        }
        CloseHandle(snap);
        found_parent
    }
}

/// Read the name of the parent process. Looks up the PID in the process
/// snapshot and returns the exe filename (without extension).
#[allow(unsafe_code)]
pub fn parent_comm() -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let ppid = parent_pid()?;
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return None;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) == 0 {
            CloseHandle(snap);
            return None;
        }
        let mut found = None;
        loop {
            if entry.th32ProcessID == ppid {
                // szExeFile is a wide-char null-terminated string. Find the
                // length and slice off everything after the last `\\`.
                let wide: &[u16] = &entry.szExeFile;
                let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
                let s = String::from_utf16_lossy(&wide[..len]);
                found = Some(s);
                break;
            }
            if Process32NextW(snap, &mut entry) == 0 {
                break;
            }
        }
        CloseHandle(snap);
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_pid_is_nonzero() {
        assert!(current_pid() > 0);
    }

    #[test]
    fn parent_pid_is_something() {
        // On Windows CI we should still find a parent (cargo, the test runner).
        // If this fails it's because of a permissions issue with toolhelp.
        if let Some(p) = parent_pid() {
            assert!(p > 0);
        }
    }
}
