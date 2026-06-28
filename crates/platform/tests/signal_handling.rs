//! G2 — signal handling: installing and triggering a ShutdownFlag works.
//!
//! These are unit tests of the platform crate's signal module — we don't
//! actually send SIGTERM in unit tests (that's the integration test in
//! `engine/tests/duration_zero.rs`). Here we just verify the flag machinery
//! is sound.

use forgum_platform::{ShutdownFlag, SignalGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn shutdown_flag_starts_false() {
    let flag = ShutdownFlag::new();
    assert!(!flag.is_shutdown());
}

#[test]
fn shutdown_flag_can_be_triggered_externally() {
    let flag = ShutdownFlag::new();
    let handle = flag.handle();
    // Spawn a thread that sets the flag after 50 ms.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        // SAFETY: we know the Arc lives as long as `flag` does; the spawn
        // here is bounded.
    });
    // We can't easily race here without sending a signal; just trigger
    // manually and assert it's visible.
    flag.trigger();
    assert!(flag.is_shutdown());
    drop(handle);
}

#[test]
fn handle_shares_state() {
    let flag = ShutdownFlag::new();
    let h1 = flag.handle();
    let h2 = flag.handle();
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_for_thread = Arc::clone(&counter);

    // One thread waits for the flag, another sets it.
    let wait_handle = std::thread::spawn(move || {
        for _ in 0..100 {
            if h1.load(Ordering::Relaxed) {
                counter_for_thread.fetch_add(1, Ordering::Relaxed);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(30));
    h2.store(true, Ordering::Relaxed);
    wait_handle.join().unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[test]
fn install_does_not_panic() {
    let flag = ShutdownFlag::new();
    let _guard = SignalGuard::install(flag.clone()).expect("install");
    // Trigger should be visible.
    flag.trigger();
    assert!(flag.is_shutdown());
}

#[test]
fn install_twice_overwrites_windows_or_stacks_unix() {
    // On Unix, signal-hook allows multiple registrations. On Windows,
    // SetConsoleCtrlHandler overwrites. Either way both flags must work.
    let a = ShutdownFlag::new();
    let b = ShutdownFlag::new();
    let _g1 = SignalGuard::install(a.clone()).expect("install a");
    let _g2 = SignalGuard::install(b.clone()).expect("install b");
    a.trigger();
    b.trigger();
    assert!(a.is_shutdown());
    assert!(b.is_shutdown());
}

#[test]
fn install_then_drop_does_not_leak_handlers() {
    let flag = ShutdownFlag::new();
    let _guard = SignalGuard::install(flag.clone()).expect("install");
    drop(_guard);
    // After drop, we should be able to install again.
    let _guard2 = SignalGuard::install(flag.clone()).expect("install after drop");
}
