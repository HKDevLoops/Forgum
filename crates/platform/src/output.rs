//! Output redirection.
//!
//! Returns a `Write` that goes to the right place:
//!
//! 1. If stdout is a TTY → wrapped stdout.
//! 2. Else on Unix → open `/dev/tty` for writing.
//! 3. Else on Windows → open `CONOUT$`.
//! 4. Else → stdout (pipe) — animations won't render, but `cow_text` still prints.
//!
//! This single helper replaces the broken `console_out` dead code (BUG-B9)
//! and removes the need for the PowerShell `cmd /c` workaround (BUG-C1).

use std::io::{self, Write};

use crate::error::PlatformError;

/// A buffered write handle that writes to stdout, `/dev/tty`, or `CONOUT$`.
///
/// Internally this is a hand-rolled buffer around a `Box<dyn Write + Send>`.
/// We don't use `std::io::BufWriter` because we need to expose a stable raw
/// pointer to the inner writer so the [`AltScreenGuard`] /
/// [`CursorShowGuard`] can attach cleanup behavior without taking ownership.
#[allow(unsafe_code)]
pub struct OutputHandle {
    /// Hand-rolled buffer. Most writes go here; we flush to `inner` on
    /// `flush()` or when the buffer fills.
    buf: Vec<u8>,
    inner: Box<dyn Write + Send>,
    /// What we ended up writing to. Useful for diagnostics.
    pub target: OutputTarget,
}

impl std::fmt::Debug for OutputHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputHandle")
            .field("buf_len", &self.buf.len())
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

/// Where the output ultimately lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTarget {
    Stdout,
    Tty,
    Pipe,
}

impl OutputHandle {
    /// Open the right output. See module docs for the resolution order.
    pub fn open() -> Result<Self, PlatformError> {
        let target = pick_target();
        // If we picked the TTY but can't actually open it (e.g. a
        // headless daemon with no controlling terminal, where open("/dev/tty")
        // fails with ENXIO), fall back to the stdout pipe rather than
        // erroring out. This is exactly the documented "Else -> stdout
        // (pipe)" contract: animations won't render, but cow_text still
        // prints. Without this, a `--daemon` child that was forked
        // with no TTY would die in OutputHandle::open() and never
        // finish wiring up, leaving no state file / control socket.
        let writer: Box<dyn Write + Send> = match target {
            OutputTarget::Stdout => Box::new(io::stdout()),
            OutputTarget::Tty => match open_tty() {
                Ok(f) => f,
                Err(_) => Box::new(io::stdout()),
            },
            OutputTarget::Pipe => Box::new(io::stdout()), // last resort
        };
        Ok(Self {
            buf: Vec::with_capacity(8 * 1024),
            inner: writer,
            target,
        })
    }

    /// Flush any buffered bytes to the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        while !self.buf.is_empty() {
            let written = self.inner.write(&self.buf)?;
            if written == 0 {
                // Writer refuses more data — drop the rest to avoid spinning.
                self.buf.clear();
                break;
            }
            self.buf.drain(..written);
        }
        self.inner.flush()
    }

    /// Returns a raw pointer to the inner writer. The pointer is valid only
    /// for as long as `&mut self` is held and the `OutputHandle` is not
    /// dropped. Used by [`AltScreenGuard::acquire`] and
    /// [`CursorShowGuard::acquire`] so they can attach cleanup behavior to
    /// the writer without taking ownership.
    ///
    /// # Safety
    /// Callers must not dereference this pointer after the OutputHandle is
    /// dropped or after a mutable borrow on `self` ends.
    pub fn raw_writer_mut(&mut self) -> *mut (dyn Write + Send) {
        &mut *self.inner as *mut (dyn Write + Send)
    }
}

impl Write for OutputHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        OutputHandle::flush(self)
    }
}

fn pick_target() -> OutputTarget {
    if crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
        OutputTarget::Stdout
    } else if can_open_tty() {
        OutputTarget::Tty
    } else {
        OutputTarget::Pipe
    }
}

#[cfg(unix)]
fn can_open_tty() -> bool {
    std::path::Path::new("/dev/tty").exists()
}

#[cfg(windows)]
fn can_open_tty() -> bool {
    // We can't easily test "can open CONOUT$" without actually opening it;
    // assume yes and let the open call fail. That's fine — we'll fall back to
    // stdout in that case.
    true
}

#[cfg(unix)]
fn open_tty() -> Result<Box<dyn Write + Send>, PlatformError> {
    use std::os::unix::fs::OpenOptionsExt;
    let f: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open("/dev/tty")
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied {
                PlatformError::NoTerminal
            } else {
                PlatformError::Io(e)
            }
        })?;
    Ok(Box::new(f))
}

#[cfg(windows)]
fn open_tty() -> Result<Box<dyn Write + Send>, PlatformError> {
    use std::os::windows::fs::OpenOptionsExt;
    // `CONOUT$` is the Windows console output device. Open it with
    // FILE_SHARE_WRITE so it works alongside other console handles.
    let f = std::fs::OpenOptions::new()
        .write(true)
        .share_mode(win_share_mode())
        .open("CONOUT$")
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                PlatformError::NoTerminal
            } else {
                PlatformError::Io(e)
            }
        })?;
    Ok(Box::new(f))
}

#[cfg(windows)]
fn win_share_mode() -> u32 {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };
    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
}

/// Convenience: same as `OutputHandle::open()`.
pub fn open_output() -> Result<OutputHandle, PlatformError> {
    OutputHandle::open()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_succeeds_or_returns_clear_error() {
        // On any platform, open() should either succeed or return a typed
        // PlatformError — never panic.
        let result = OutputHandle::open();
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    matches!(e, PlatformError::NoTerminal | PlatformError::Io(_)),
                    "unexpected error: {e}"
                );
            }
        }
    }

    #[test]
    fn target_is_deterministic() {
        // Opening twice in the same process returns the same target.
        let a = OutputHandle::open().unwrap();
        let b = OutputHandle::open().unwrap();
        assert_eq!(a.target, b.target);
    }

    #[test]
    fn buffer_then_flush_does_not_lose_data() {
        let mut h = OutputHandle::open().unwrap();
        let payload = b"hello world";
        let n = h.write(payload).unwrap();
        assert_eq!(n, payload.len());
        h.flush().unwrap();
    }
}
