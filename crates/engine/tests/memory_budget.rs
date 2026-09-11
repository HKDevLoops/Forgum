//! Verification of the strict <100MB RAM mandate under heavy parallel execution.
//!
//! Rule I: The entire Forgum execution across parallel render loops and CLI operations
//! must strictly consume under 100MB of resident RAM.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use forgum_engine::dna::BaseAnim;
use forgum_engine::framebuffer::FrameBuffer;
use forgum_engine::scenery::{render_scenery_full, EnvironmentStyle, MountainStyle, RoadStyle};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

#[test]
fn test_parallel_rendering_memory_strictly_under_100mb() {
    let num_threads = 8;
    let frames_per_thread = 200;
    let running = Arc::new(AtomicBool::new(true));
    let peak_rss_atomic = Arc::new(std::sync::atomic::AtomicU64::new(0));

    // Spawn concurrent peak memory monitor sampling every 5ms
    let monitor_running = Arc::clone(&running);
    let peak_rss_writer = Arc::clone(&peak_rss_atomic);
    let monitor_handle = thread::spawn(move || {
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
        );
        while monitor_running.load(Ordering::Relaxed) {
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            if let Some(proc) = sys.process(pid) {
                let mem = proc.memory();
                peak_rss_writer.fetch_max(mem, Ordering::Relaxed);
            }
            thread::sleep(std::time::Duration::from_millis(5));
        }
    });

    let mut handles = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        let running_clone = Arc::clone(&running);
        let handle = thread::spawn(move || {
            let mut fb = FrameBuffer::new(120, 40);
            let styles = [
                (
                    MountainStyle::Peaks,
                    RoadStyle::Magma,
                    EnvironmentStyle::Inferno,
                ),
                (
                    MountainStyle::Hills,
                    RoadStyle::Dirt,
                    EnvironmentStyle::Pasture,
                ),
                (
                    MountainStyle::Volcano,
                    RoadStyle::Tracks,
                    EnvironmentStyle::Jurassic,
                ),
                (
                    MountainStyle::Iceberg,
                    RoadStyle::Ice,
                    EnvironmentStyle::Arctic,
                ),
            ];

            for f in 0..frames_per_thread {
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }
                let (mtn, road, env) = styles[(thread_id + f) % styles.len()];
                let time = (f as f32) * 0.05;
                fb.clear();
                render_scenery_full(
                    &mut fb,
                    mtn,
                    road,
                    env,
                    35,
                    time,
                    Some("dragon"),
                    Some(12),
                    Some(BaseAnim::Walk),
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread joined cleanly");
    }

    // Stop peak memory monitor
    running.store(false, Ordering::Relaxed);
    monitor_handle
        .join()
        .expect("Monitor thread joined cleanly");

    let peak_bytes = peak_rss_atomic.load(Ordering::Relaxed);
    let peak_mb = peak_bytes as f64 / (1024.0 * 1024.0);
    println!(
        "Peak concurrent parallel rendering memory: {:.2} MB",
        peak_mb
    );

    let max_allowed_bytes = 100 * 1024 * 1024; // 100MB strict ceiling

    // If monitor caught samples, verify peak usage; otherwise verify post-run memory
    if peak_bytes > 0 {
        assert!(
            peak_bytes < max_allowed_bytes,
            "Memory ceiling violated: peak concurrent usage was {:.2} MB, exceeding the 100MB limit",
            peak_mb
        );
    } else {
        // Fallback post-join check
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
        );
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        if let Some(process) = sys.process(pid) {
            let mem_bytes = process.memory();
            let mem_mb = mem_bytes as f64 / (1024.0 * 1024.0);
            println!("Post-run process memory: {:.2} MB", mem_mb);
            assert!(
                mem_bytes < max_allowed_bytes,
                "Memory limit violated: process used {:.2} MB, exceeding the 100MB ceiling",
                mem_mb
            );
        }
    }
}
