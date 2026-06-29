//! Cross-platform IPC socket for daemon control.
//!
//! - Unix: Unix domain socket (`SOCK_STREAM`)
//! - Windows: Named pipe (`\\.\pipe\forgum-{session}`)

use std::io;
use std::path::Path;

#[cfg(unix)]
use std::io::{BufRead, Write};

use crate::error::PlatformError;

/// A bound and listening IPC socket.
#[allow(missing_debug_implementations)]
pub struct DaemonSocket {
    inner: SocketInner,
}

enum SocketInner {
    #[cfg(unix)]
    Unix {
        listener: std::os::unix::net::UnixListener,
        _path: std::path::PathBuf,
    },
    #[cfg(windows)]
    Windows {
        handle: windows_sys::Win32::Foundation::HANDLE,
    },
}

impl DaemonSocket {
    /// Bind a new socket at the given path.
    ///
    /// On Unix, removes any stale socket file before binding to avoid
    /// `EADDRINUSE`. Sets the listener to non-blocking mode so
    /// [`accept`](Self::accept) never blocks the caller.
    ///
    /// On Windows, creates a named pipe server instance and waits for a
    /// client connection. The `path` parameter is used only to derive the
    /// pipe name; the file system path itself is not created.
    pub fn bind(path: &Path) -> Result<Self, PlatformError> {
        #[cfg(unix)]
        {
            // Remove stale socket file from a previous unclean shutdown.
            let _ = std::fs::remove_file(path);

            let listener = std::os::unix::net::UnixListener::bind(path)
                .map_err(PlatformError::Io)?;
            listener
                .set_nonblocking(true)
                .map_err(PlatformError::Io)?;

            Ok(Self {
                inner: SocketInner::Unix {
                    listener,
                    _path: path.to_path_buf(),
                },
            })
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            let pipe_name: Vec<u16> = std::ffi::OsStr::new(&path_to_pipe_name(path))
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // SAFETY: CreateNamedPipeW creates a named pipe server instance.
            // Parameters are validated:
            // - pipe_name is a null-terminated wide string
            // - PIPE_ACCESS_DUPLEX allows bidirectional communication
            // - PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT for
            //   stream-oriented, blocking byte-mode I/O
            // - 1 max instance, 4096 byte buffers, infinite timeout
            // - No security attributes (default)
            #[allow(unsafe_code)]
            let handle = unsafe {
                windows_sys::Win32::System::Pipes::CreateNamedPipeW(
                    pipe_name.as_ptr(),
                    windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX,
                    windows_sys::Win32::System::Pipes::PIPE_TYPE_BYTE
                        | windows_sys::Win32::System::Pipes::PIPE_READMODE_BYTE
                        | windows_sys::Win32::System::Pipes::PIPE_WAIT,
                    1,          // max instances
                    4096,       // out buffer size
                    4096,       // in buffer size
                    0,          // default timeout
                    std::ptr::null(),
                )
            };

            if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(PlatformError::Io(io::Error::last_os_error()));
            }

            // SAFETY: ConnectNamedPipe blocks until a client connects.
            // The handle is valid and was just created by CreateNamedPipeW.
            // We pass null for OVERLAPPED since this is a blocking pipe.
            #[allow(unsafe_code)]
            let ok = unsafe {
                windows_sys::Win32::System::Pipes::ConnectNamedPipe(
                    handle,
                    std::ptr::null_mut(),
                )
            };

            if ok == 0 {
                let err = io::Error::last_os_error();
                // SAFETY: handle is valid; we must clean up on failure.
                #[allow(unsafe_code)]
                unsafe {
                    windows_sys::Win32::Foundation::CloseHandle(handle);
                }
                return Err(PlatformError::Io(err));
            }

            Ok(Self {
                inner: SocketInner::Windows { handle },
            })
        }
    }

    /// Accept a new connection.
    ///
    /// On Unix this is non-blocking (returns `Ok(None)` immediately if no
    /// client is waiting). On Windows the pipe was already connected during
    /// [`bind`](Self::bind), so the server handle is returned directly.
    pub fn accept(&self) -> Result<Option<SocketConnection>, PlatformError> {
        match &self.inner {
            #[cfg(unix)]
            SocketInner::Unix { listener, .. } => match listener.accept() {
                Ok((stream, _addr)) => Ok(Some(SocketConnection::Unix(stream))),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
                Err(e) => Err(PlatformError::Io(e)),
            },
            #[cfg(windows)]
            SocketInner::Windows { handle, .. } => {
                Ok(Some(SocketConnection::Windows(*handle)))
            }
        }
    }

    /// Remove the socket file (Unix) or do nothing (Windows).
    ///
    /// On Unix this removes the `.sock` file from the filesystem. On Windows,
    /// named pipes are kernel objects that are cleaned up when all handles are
    /// closed, so this is a no-op.
    pub fn cleanup(&self) {
        #[cfg(unix)]
        if let SocketInner::Unix { _path, .. } = &self.inner {
            let _ = std::fs::remove_file(_path);
        }
    }
}

/// A single accepted IPC connection.
#[allow(missing_debug_implementations)]
pub enum SocketConnection {
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixStream),
    #[cfg(windows)]
    Windows(windows_sys::Win32::Foundation::HANDLE),
}

impl SocketConnection {
    /// Read a newline-delimited command from the client.
    ///
    /// Returns `Ok(Some(line))` on success, `Ok(None)` on EOF.
    pub fn read_line(&mut self) -> Result<Option<String>, PlatformError> {
        match self {
            #[cfg(unix)]
            Self::Unix(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => Ok(None),
                    Ok(_) => {
                        // Trim the trailing newline.
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        Ok(Some(line))
                    }
                    Err(e) => Err(PlatformError::Io(e)),
                }
            }
            #[cfg(windows)]
            Self::Windows(handle) => {
                let mut line = Vec::new();
                let mut buf = [0u8; 1];

                loop {
                    let mut bytes_read = 0u32;
                    // SAFETY: handle is valid (server pipe handle), buf is a
                    // valid 1-byte buffer, and bytes_read is a valid pointer
                    // for the output count. We pass null for OVERLAPPED
                    // since this is a blocking pipe.
                    #[allow(unsafe_code)]
                    let ok = unsafe {
                        windows_sys::Win32::Storage::FileSystem::ReadFile(
                            *handle,
                            buf.as_mut_ptr(),
                            1,
                            &mut bytes_read,
                            std::ptr::null_mut(),
                        )
                    };

                    if ok == 0 || bytes_read == 0 {
                        if line.is_empty() {
                            return Ok(None);
                        }
                        break;
                    }

                    if buf[0] == b'\n' {
                        break;
                    }
                    line.push(buf[0]);
                }

                // Strip trailing \r if present.
                if line.last() == Some(&b'\r') {
                    line.pop();
                }

                String::from_utf8(line)
                    .map(Some)
                    .map_err(|e| PlatformError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))
            }
        }
    }

    /// Write a response to the client.
    pub fn write_response(&mut self, data: &str) -> Result<(), PlatformError> {
        match self {
            #[cfg(unix)]
            Self::Unix(stream) => {
                stream.write_all(data.as_bytes()).map_err(PlatformError::Io)?;
                stream.flush().map_err(PlatformError::Io)?;
                Ok(())
            }
            #[cfg(windows)]
            Self::Windows(handle) => {
                let bytes = data.as_bytes();
                let mut bytes_written = 0u32;
                // SAFETY: handle is valid (server pipe handle), bytes is a
                // valid buffer of known length, and bytes_written is a valid
                // output parameter. We pass null for OVERLAPPED since this
                // is a blocking pipe.
                #[allow(unsafe_code)]
                let ok = unsafe {
                    windows_sys::Win32::Storage::FileSystem::WriteFile(
                        *handle,
                        bytes.as_ptr(),
                        bytes.len() as u32,
                        &mut bytes_written,
                        std::ptr::null_mut(),
                    )
                };

                if ok == 0 {
                    return Err(PlatformError::Io(io::Error::last_os_error()));
                }
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
impl Drop for SocketConnection {
    fn drop(&mut self) {
        let Self::Windows(handle) = self;
        // SAFETY: The handle is a valid server pipe handle or INVALID.
        // CloseHandle on INVALID_HANDLE_VALUE is a no-op per MSDN.
        #[allow(unsafe_code)]
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(*handle);
        }
    }
}

/// Convert a filesystem path to a Windows named pipe path.
///
/// Given `/tmp/Forgum/ctrl-session.sock` or
/// `C:\Users\...\Forgum\ctrl-session.pipe`, returns
/// `\\.\pipe\ctrl-session.sock` (or `.pipe`).
#[cfg(windows)]
fn path_to_pipe_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("forgum-ipc");
    format!("\\\\.\\pipe\\{}", file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_pipe_name_extracts_filename() {
        let path = Path::new(r"C:\Users\test\AppData\Local\Forgum\ctrl-session.pipe");
        assert_eq!(path_to_pipe_name(path), r"\\.\pipe\ctrl-session.pipe");
    }

    #[test]
    fn path_to_pipe_name_unix_style_path() {
        let path = Path::new("/tmp/Forgum/ctrl-abc.sock");
        assert_eq!(path_to_pipe_name(path), r"\\.\pipe\ctrl-abc.sock");
    }

    #[test]
    #[cfg(unix)]
    fn unix_bind_accept_cleanup_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let sock_path = tmp.path().join("test.sock");

        let socket = DaemonSocket::bind(&sock_path).unwrap();
        assert!(sock_path.exists());

        // Non-blocking accept with no client -> None.
        let conn = socket.accept().unwrap();
        assert!(conn.is_none());

        socket.cleanup();
        assert!(!sock_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn unix_stale_socket_removed_on_bind() {
        let tmp = tempfile::tempdir().unwrap();
        let sock_path = tmp.path().join("stale.sock");

        // Create a stale socket file.
        std::fs::write(&sock_path, b"stale").unwrap();
        assert!(sock_path.exists());

        // bind() should remove the stale file and succeed.
        let socket = DaemonSocket::bind(&sock_path).unwrap();
        assert!(sock_path.exists());
        socket.cleanup();
    }
}
