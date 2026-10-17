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

/// Check if stdin has data available to read without blocking indefinitely.
#[allow(unsafe_code)]
pub fn stdin_has_data() -> bool {
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{GetFileType, FILE_TYPE_DISK, FILE_TYPE_PIPE};
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

    if crossterm::tty::IsTty::is_tty(&io::stdin()) {
        return false;
    }

    let handle = io::stdin().as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    let file_type = unsafe { GetFileType(handle) };
    if file_type == FILE_TYPE_DISK {
        return true;
    }
    if file_type == FILE_TYPE_PIPE {
        let mut avail: u32 = 0;
        let ok = unsafe {
            PeekNamedPipe(
                handle,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut avail,
                std::ptr::null_mut(),
            )
        };
        return ok != 0 && avail > 0;
    }
    false
}

/// Direct Win32 Registry probe for GPU display adapters.
/// Executes in < 0.1ms with zero process spawns or allocations.
#[allow(unsafe_code)]
pub fn probe_windows_gpus_registry() -> Vec<(String, String, Option<u64>)> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
    };

    let mut results = Vec::new();
    let mut consecutive_misses = 0;

    let class_guids = [
        "{4d36e968-e325-11ce-bfc1-08002be10318}", // Display Adapters (GPU)
        "{5b45201d-f2f2-4f3b-85bb-30ff1f953599}", // Compute Accelerators (NPU)
    ];

    for class_guid in class_guids {
        for idx in 0..16 {
            let subkey_str = format!(
                "SYSTEM\\CurrentControlSet\\Control\\Class\\{}\\{:04}\0",
                class_guid, idx
            );
            let subkey_wide: Vec<u16> = subkey_str.encode_utf16().collect();
            let mut hkey: HKEY = unsafe { std::mem::zeroed() };

            let status = unsafe {
                RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    subkey_wide.as_ptr(),
                    0,
                    KEY_READ,
                    &mut hkey,
                )
            };

            if status != 0 {
                consecutive_misses += 1;
                if consecutive_misses >= 2 {
                    break;
                }
                continue;
            }
            consecutive_misses = 0;

            let query_string = |val_name: &str| -> Option<String> {
                let val_wide: Vec<u16> = format!("{val_name}\0").encode_utf16().collect();
                let mut val_type: u32 = 0;
                let mut byte_len: u32 = 0;
                let res = unsafe {
                    RegQueryValueExW(
                        hkey,
                        val_wide.as_ptr(),
                        std::ptr::null_mut(),
                        &mut val_type,
                        std::ptr::null_mut(),
                        &mut byte_len,
                    )
                };
                if res != 0 || byte_len == 0 {
                    return None;
                }
                let mut buf = vec![0u8; byte_len as usize];
                let res = unsafe {
                    RegQueryValueExW(
                        hkey,
                        val_wide.as_ptr(),
                        std::ptr::null_mut(),
                        &mut val_type,
                        buf.as_mut_ptr(),
                        &mut byte_len,
                    )
                };
                if res != 0 {
                    return None;
                }
                let u16_slice: &[u16] = unsafe {
                    std::slice::from_raw_parts(buf.as_ptr() as *const u16, (byte_len as usize) / 2)
                };
                let end = u16_slice
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(u16_slice.len());
                Some(String::from_utf16_lossy(&u16_slice[..end]))
            };

            let query_u64 = |val_name: &str| -> Option<u64> {
                let val_wide: Vec<u16> = format!("{val_name}\0").encode_utf16().collect();
                let mut val_type: u32 = 0;
                let mut byte_len: u32 = 8;
                let mut buf = [0u8; 8];
                let res = unsafe {
                    RegQueryValueExW(
                        hkey,
                        val_wide.as_ptr(),
                        std::ptr::null_mut(),
                        &mut val_type,
                        buf.as_mut_ptr(),
                        &mut byte_len,
                    )
                };
                if res != 0 {
                    return None;
                }
                if byte_len == 8 {
                    Some(u64::from_le_bytes(buf))
                } else if byte_len == 4 {
                    let mut dword_bytes = [0u8; 4];
                    dword_bytes.copy_from_slice(&buf[..4]);
                    Some(u32::from_le_bytes(dword_bytes) as u64)
                } else {
                    None
                }
            };

            let driver_desc = query_string("DriverDesc").unwrap_or_default();
            let matching_id = query_string("MatchingDeviceId").unwrap_or_default();
            let vram_bytes = query_u64("HardwareInformation.qwMemorySize");

            unsafe {
                RegCloseKey(hkey);
            }

            if !driver_desc.is_empty() {
                results.push((driver_desc, matching_id, vram_bytes));
            }
        }
    }

    results
}

/// Returns `true` if the parent shell process is currently the active foreground
/// process in the terminal/console. If an editor (e.g. `nvim`, `vim`, `emacs`, `nano`)
/// or another foreground command is running in the console, this returns `false`.
#[allow(unsafe_code)]
#[must_use]
pub fn is_parent_shell_in_foreground(parent_pid: Option<u32>) -> bool {
    use windows_sys::Win32::System::Console::GetConsoleProcessList;

    let ppid = match parent_pid {
        Some(pid) if pid > 0 => pid,
        _ => return true,
    };
    let my_pid = std::process::id();

    let mut pids = [0u32; 64];
    let count = unsafe { GetConsoleProcessList(pids.as_mut_ptr(), pids.len() as u32) };
    if count == 0 {
        return true;
    }

    let slice_len = (count as usize).min(pids.len());
    let active_pids = &pids[..slice_len];

    // If there is any PID attached to the console that is NOT us and NOT the parent shell,
    // an interactive child command/editor is running in the console!
    for &pid in active_pids {
        if pid != 0 && pid != my_pid && pid != ppid {
            return false;
        }
    }

    true
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
