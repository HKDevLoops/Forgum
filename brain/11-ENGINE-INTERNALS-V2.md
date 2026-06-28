# Forgum — Engine Internals v2 (Multi-threaded, DCL, No-Leak, Hybrid Rendering)

> **The deep dive.** This document specifies the engine internals that make Forgum a *professional, efficient, leak-free, multi-threaded* program with *hardware + software rendering*. It is the implementation companion to `10-FINE-TUNED-MASTER-PLAN.md` Phase 1 + 4. Every struct, every channel, every singleton is named. Every claim is grounded in the research in `worklog.md` (Task IDs `RESEARCH-MT`, `RESEARCH-XPLAT`, `RESEARCH-ANIM`).
>
> **Audience:** the Rust engineer implementing `crates/engine/src/`. Read once before touching `main.rs`.

---

## 1. The 3-thread model

Forgum runs three OS threads. No async runtime (tokio/async-std) is needed — the workload is CPU-bound simulation + blocking I/O, not thousands of concurrent connections. A `tokio` dep would add 2 MB to the binary and a runtime tax for zero benefit.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         forgum-engine process                           │
│                                                                         │
│  ┌─────────────────┐         ┌─────────────────┐                        │
│  │  CONTROL THREAD │         │   SIM THREAD    │                        │
│  │  (main thread)  │         │  (spawned)       │                        │
│  │                 │         │                  │                        │
│  │ • signal-hook   │ Control │ • owns back:     │                        │
│  │   SIGWINCH      │ Msg     │   Vec<Cell>      │                        │
│  │ • SIGTERM/INT/  │ ──────► │ • owns Particle  │                        │
│  │   HUP/TSTP/CONT │ (cross- │   Pool (slotmap) │                        │
│  │ • control sock  │  beam   │ • owns AnimState │                        │
│  │   accept loop   │  unbounded)│  (DNA + Verlet)│                       │
│  │ • NO tty reads  │         │ • rayon::par_    │                        │
│  │ • pushes cmds   │         │   chunks_mut for │                        │
│  │                 │ ◄────── │   particles      │                        │
│  │                 │ Frame   │ • fixed-timestep │                        │
│  │                 │ Request │   accumulator    │                        │
│  │                 │ (sync)  │ • ships Arc<Frame>│                       │
│  └─────────────────┘         │   on frame bound │                        │
│                              └────────┬─────────┘                        │
│                                       │ Arc<Frame>                       │
│                                       │ (crossbeam bounded(2))           │
│                                       ▼                                  │
│                              ┌─────────────────┐                        │
│                              │  RENDER THREAD  │                        │
│                              │  (spawned)       │                        │
│                              │                  │                        │
│                              │ • owns front:    │                        │
│                              │   Vec<Cell>      │                        │
│                              │ • owns Renderer  │                        │
│                              │   (trait object) │                        │
│                              │ • compute_damage │                        │
│                              │ • coalesced runs │                        │
│                              │ • BeginSyncUpd / │                        │
│                              │   EndSyncUpd     │                        │
│                              │ • BufWriter<     │                        │
│                              │   Mutex<Stdout>> │                        │
│                              └─────────────────┘                        │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.1 Why three threads (not two, not async)

- **Control thread is the only one that touches signals and the control socket.** Signals cannot be handled safely inside a thread doing `Vec` mutation (no allocation, no locking in a handler). `signal-hook`'s self-pipe trick moves the work out of the handler context; the control thread `poll`s the self-pipe + the socket and pushes `ControlMsg`s onto a channel. This thread NEVER reads from `/dev/tty` (the BUG-B1 fix).
- **Sim thread owns the mutable simulation state** (`back` buffer, particles, `AnimState`). It runs the fixed-timestep accumulator and parallelizes particle integration with `rayon`. It does NOT touch stdout.
- **Render thread owns the terminal output** (`front` buffer, `Renderer` trait, `BufWriter`). It does NOT mutate simulation state. It receives `Arc<Frame>` snapshots and computes damage + emits ANSI.

Two threads would conflate sim + render (the current bug: `scheduler.wait_if_needed()` blocks the sim thread that also writes stdout). Async would add a runtime for no gain (no I/O multiplexing needed — one stdout, one socket, one signal source).

### 1.2 The channel topology

```rust
use crossbeam_channel::{bounded, unbounded};

// Control → Sim: unbounded (commands are rare, never block the control thread)
let (control_tx, control_rx) = unbounded::<ControlMsg>();

// Sim → Render: bounded(2) (backpressure — sim can't outrun render by more than 1 frame)
let (frame_tx, frame_rx) = bounded::<Arc<Frame>>(2);

// Render → Sim: sync request for next frame (keeps sim from racing ahead)
let (request_tx, request_rx) = bounded::<()>(1);
```

`bounded(2)` for the frame channel is the key insight: it gives natural backpressure. If the render thread is stalled (slow terminal, SSH lag), the sim thread blocks on `frame_tx.send()` after 2 frames — it doesn't pile up memory with unrendered frames. The `request_tx`/`request_rx` pair makes this explicit: the render thread requests the next frame only after it has flushed the previous one.

### 1.3 Thread spawn with `std::thread::scope`

Scoped threads (stable since Rust 1.63) let the sim and render threads borrow `&mut` stack-local data in `main` without `Arc<Mutex<>>` overhead. The borrow checker proves all spawned threads join before the scope returns.

```rust
fn run_engine(config: Config) -> io::Result<()> {
    let mut sim_state = SimState::new(config)?;
    let mut render_state = RenderState::new()?;

    std::thread::scope(|s| {
        let control_handle = s.spawn(|| control_thread(control_rx_clone, shutdown_flag.clone()));
        let sim_handle = s.spawn(|| sim_thread(sim_state, control_rx, frame_tx, request_rx, shutdown_flag.clone()));
        let render_handle = s.spawn(|| render_thread(render_state, frame_rx, request_tx, shutdown_flag.clone()));

        // Wait for shutdown signal, then join all
        shutdown_flag.wait();
        control_handle.join().expect("control thread panicked");
        sim_handle.join().expect("sim thread panicked");
        render_handle.join().expect("render thread panicked");
    });
    Ok(())
}
```

No `Arc<Mutex<>>` for the simulation state — it's owned exclusively by the sim thread. No `unsafe`. No leaks.

---

## 2. DCL singletons (Double-Checked Locking, the modern Rust way)

### 2.1 The singleton hierarchy

Forgum uses four singleton tiers, each with the correct primitive:

| Tier | Use case | Primitive | Why |
|------|----------|-----------|-----|
| **Static, set-once, no reset** | `CONFIG`, compiled-in cow data | `LazyLock<T>` (1.80) | Ergonomic `Deref`; init runs once on first access |
| **Runtime-probed, set-once** | `TERM_CAPS` (terminal capability probe) | `OnceLock<T>` (1.70) | Init depends on runtime env; `get_or_init` is DCL-safe |
| **Resettable** | `PARTICLE_POOL` (reset on resize) | `LazyLock<Mutex<Option<T>>>` | `OnceLock` can't be reset; `Mutex<Option<T>>` can `take()` + re-init |
| **Atomic counters** | `FRAME_COUNT`, `SPAWN_COUNT` | `AtomicU64` | No lock; `Relaxed`/`AcqRel` ordering |

### 2.2 The DCL pattern, explicitly

`OnceLock::get_or_init` encodes double-checked locking safely:

```rust
// The conceptual pattern OnceLock implements internally:
fn get_or_init(&self, init: impl FnOnce() -> T) -> &T {
    // FAST PATH: single Acquire load (no lock)
    if let Some(val) = self.inner.load(Acquire) {
        return val;
    }
    // SLOW PATH: take the lock
    let guard = self.mutex.lock();
    // DOUBLE CHECK: another thread may have initialized while we waited
    if let Some(val) = self.inner.load(Relaxed) {
        return val;
    }
    let val = init();
    self.inner.store(val, Release);  // publish with Release
    val
}
```

The Acquire/Release pair forms the happens-before relationship: a reader's Acquire load synchronizes with the writer's Release store, guaranteeing the initialized `T` is visible. `OnceLock` is `unsafe`-free and audited. **Never hand-roll DCL** — `OnceLock` is the correct, reviewed implementation.

### 2.3 Concrete Forgum singletons

```rust
use std::sync::{OnceLock, LazyLock, Mutex, atomic::{AtomicU64, Ordering}};

// 1. Config: loaded once at startup, never changes.
static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::load_or_default()
});

// 2. Terminal capabilities: probed lazily on first render.
static TERM_CAPS: OnceLock<TermCaps> = OnceLock::new();
// Usage: TERM_CAPS.get_or_init(|| probe_terminal_capabilities());

// 3. Resettable particle pool (cleared on resize).
static PARTICLE_POOL: LazyLock<Mutex<Option<ParticlePool>>> =
    LazyLock::new(|| Mutex::new(None));
// Reset on resize:
*PARTICLE_POOL.lock().unwrap() = Some(ParticlePool::new(capacity));
// Take + drop (e.g. on shutdown):
let _ = PARTICLE_POOL.lock().unwrap().take();

// 4. Atomic counters (no lock, no DCL needed).
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);
static SPAWN_COUNT: AtomicU64 = AtomicU64::new(0);
// Usage: FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
```

### 2.4 What we removed

- **`static mut`** — Edition 2024 forbids it; we adopt the rule now. `rg 'static mut' crates/engine/src/` must return 0 hits (CI gate).
- **`lazy_static!`** — officially deprecated (clippy #12895). Migrate to `LazyLock`.
- **`once_cell::sync::Lazy`** — keep only if MSRV < 1.80; otherwise `LazyLock`.
- **`Box::leak`** in init paths — replaces with `LazyLock`/`Arc`. `rg 'Box::leak|mem::forget' crates/engine/src/` must return 0 hits (CI gate, with reviewed exceptions).

### 2.5 Why `OnceLock` doesn't poison (and `LazyLock` does)

If the init closure panics:
- `OnceLock` stays uninitialized; a later caller can retry `get_or_init` with a different closure. This is the right behavior for `TERM_CAPS` — a failed probe shouldn't kill the process.
- `LazyLock` poisons; future accesses panic. This is acceptable for `CONFIG` — if config load panics, the process should die loudly.

Choose `OnceLock` for runtime-probed singletons (resilient); `LazyLock` for static singletons (fail-fast).

---

## 3. Memory-leak prevention (the zero-alloc hot loop)

### 3.1 The memory budget

| Region | Size | Lifetime |
|--------|------|----------|
| `FrameBuffer` back + front (80×24) | 2 × 1920 × 16B = 60 KB | process |
| `ParticlePool` (10k particles, SoA) | 10k × 32B = 320 KB | process (reset on resize) |
| `bumpalo::Bump` per-frame arena | 64 KB initial, grows as needed | per-frame (`reset()` at top) |
| `BufWriter<Mutex<Stdout>>` | 8 KB | process |
| Cow text + speech bubble | ~2 KB per cow | per-render |
| `Arc<Frame>` in flight | 60 KB × 2 (bounded channel) | transient |

**Total steady-state RSS: < 2 MB.** No growth over time.

### 3.2 The per-frame arena (`bumpalo`)

Every allocation that exists only for one frame (ANSI escape buffer, damage list, run-coalesce strings, particle spawn requests) lives in a `bumpalo::Bump` owned by the sim thread. At frame top, `bump.reset()` rewinds the allocation pointer — mass deallocation in O(1), no per-object `Drop`.

```rust
use bumpalo::Bump;

struct SimState {
    back: Vec<Cell>,
    particles: ParticlePool,
    arena: Bump,  // per-frame scratch
    // ...
}

impl SimState {
    fn tick(&mut self, dt_ns: i64) -> Arc<Frame> {
        self.arena.reset();  // <-- O(1) mass dealloc of last frame's scratch

        // All per-frame allocations go through the arena:
        let damage_list: &mut Vec<(usize, usize)> =
            self.arena.alloc(bumpalo::vec![(0usize, 0usize); 256]);
        let ansi_buf: &mut bumpalo::collections::String =
            self.arena.alloc(bumpalo::collections::String::with_capacity_in(4096, &self.arena));

        // ... simulate, render into back buffer, compute damage ...

        // Ship an Arc<Frame> snapshot to the render thread.
        // Arc<Frame> itself is NOT in the arena (it must outlive this frame).
        Arc::new(Frame {
            cells: self.back.clone(),  // one alloc per frame, dropped when render thread drops the Arc
            damage: damage_list.to_vec(),
        })
    }
}
```

**Important:** `bumpalo::Bump::reset()` does NOT call `Drop` on allocated values. Only put `Copy` types or trivially-droppable types (Vec of Copy, String of chars) in the arena. If a per-frame value needs `Drop` (e.g. holds a file handle — rare), wrap it in `bumpalo::boxed::Box<T>` which runs `T::drop` on scope exit.

### 3.3 The `dhat` CI gate (zero-alloc assertion)

`dhat` is an in-process allocation profiler. Swap it in as the global allocator in a test build, run the hot loop, assert zero allocations after warmup:

```rust
// tests/alloc_budget.rs
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn hot_loop_zero_alloc_after_warmup() {
    let mut sim = SimState::new(test_config());
    // Warmup: 10 frames to stabilize (initial Vec growth, RNG init, etc.)
    for _ in 0..10 { sim.tick(16_666_667); }
    let stats_before = dhat::GlobalAlloc::get_stats();
    // Measured: 60 frames
    for _ in 0..60 { sim.tick(16_666_667); }
    let stats_after = dhat::GlobalAlloc::get_stats();
    assert_eq!(
        stats_after.total_bytes - stats_before.total_bytes, 0,
        "Hot loop allocated {} bytes after warmup — zero-alloc invariant violated",
        stats_after.total_bytes - stats_before.total_bytes
    );
}
```

CI runs this on every PR. **The hot loop allocates 0 bytes after warmup** — this is the single most important memory invariant.

### 3.4 The particle pool (`slotmap`)

The current `ParticlePool::spawn` does `self.active.iter().position(|&a| !a)` — O(n) per spawn, and on a full pool it silently drops. Replace with `slotmap::SlotMap`:

```rust
use slotmap::{SlotMap, new_key_type};
new_key_type! { pub struct ParticleKey; }

pub struct ParticlePool {
    particles: SlotMap<ParticleKey, Particle>,
}

pub struct Particle {
    pub x: f32, pub y: f32,
    pub vx: f32, pub vy: f32,
    pub life: f32, pub max_life: f32,
    pub ch: char,
    pub r: u8, pub g: u8, pub b: u8,
}

impl ParticlePool {
    pub fn spawn(&mut self, p: Particle) -> Option<ParticleKey> {
        self.particles.insert(p)  // O(1), returns None if capacity reached
    }
    pub fn kill(&mut self, key: ParticleKey) {
        self.particles.remove(key);  // O(1)
    }
    pub fn iter_active(&self) -> impl Iterator<Item = (ParticleKey, &Particle)> {
        self.particles.iter()  // skips empty slots
    }
}
```

`SlotMap` uses a `(index, version)` key — O(1) insert/remove, ABA-safe (a killed key stays invalid forever). `HopSlotMap` is faster for iteration (use it if profiling shows iteration is hot). Memory: `4 + max(sizeof(Particle), 4)` bytes per slot.

**Trade-off note:** `slotmap` uses `unsafe` internally (audited, widely deployed). If the project forbids `unsafe` deps, use `generational_arena` (zero `unsafe`, slightly slower). For Forgum, `slotmap` is the right choice.

### 3.5 Parallel particle integration (`rayon`)

The SoA layout is cache-friendly; `rayon::par_chunks_mut` parallelizes integration across cores with zero allocation:

```rust
use rayon::prelude::*;

impl ParticlePool {
    pub fn integrate(&mut self, dt: f32) {
        // SlotMap doesn't expose par_chunks_mut directly; either:
        // (a) extract SoA slices and par_chunks_mut them, or
        // (b) collect active keys, par_iter_mut over them.
        // Option (a) is faster; we keep a parallel SoA cache updated on spawn/kill.

        let active_count = self.particles.len();
        if active_count < 1024 {
            // Below 1k particles, rayon's overhead exceeds the gain — single-thread.
            for (_, p) in self.particles.iter_mut() {
                p.integrate(dt);
            }
        } else {
            // Parallel: split the SoA slices into 1024-wide chunks.
            self.x.par_chunks_mut(1024)
                .zip(&mut self.y).zip(&mut self.vx).zip(&mut self.vy).zip(&mut self.life)
                .for_each(|((((x, y), vx), vy), life)| {
                    integrate_chunk(x, y, vx, vy, life, dt);
                });
        }
    }
}
```

Rayon's work-stealing thread pool is shared across the process — multiple rayon calls in one frame reuse the same workers. For a dedicated pool (no global state pollution), use `rayon::ThreadPoolBuilder::new().num_threads(N).build()` and `pool.install(|| par_iter…)`.

### 3.6 Leak detection toolchain

| Tool | When | What it catches |
|------|------|-----------------|
| `dhat` (CI, every PR) | `cargo test --test alloc_budget` | Per-frame alloc regressions |
| `cargo +nightly miri test` (CI, nightly) | on `protocol.rs` + `terminal.rs` unsafe | UB: use-after-free, OOB, invalid atomics |
| `valgrind --leak-check=full` (manual, pre-release) | Linux x86_64 | Leaks in long runs; suppressed for intentional `LazyLock` statics via `forgum-leaks.supp` |
| `heaptrack` (manual, profiling) | Linux | Flamegraph of alloc hotspots |
| `tracing` + `tracing-flame` (always on, behind feature) | long-running daemon | Growing-allocation spans (leak signature) |

### 3.7 The leak-prevention checklist (CI-enforced)

- `rg 'Box::leak' crates/engine/src/` → 0 hits (reviewed exceptions only).
- `rg 'mem::forget' crates/engine/src/` → 0 hits.
- `rg 'static mut' crates/engine/src/` → 0 hits.
- `rg 'Rc<' crates/engine/src/` → 0 hits (use `Arc` for thread-safe; no `Rc` cycles possible).
- `dhat` zero-alloc test green.
- Miri clean on unsafe.
- `valgrind` 24-hour soak test: RSS growth < 1 MB (pre-release manual).

---

## 4. The `Renderer` trait (hardware + software rendering)

### 4.1 The trait

```rust
pub trait Renderer: Send {
    /// Render a frame. `damage` is the list of (x, y) cells that changed.
    /// Returns bytes written.
    fn render_frame(&mut self, frame: &Frame, damage: &[(usize, usize)]) -> io::Result<usize>;

    /// Clear the overlay region (on shutdown / resize).
    fn clear_overlay(&mut self, bounds: Rect) -> io::Result<()>;

    /// What backend this is (for logging / status).
    fn backend_name(&self) -> &'static str;
}
```

### 4.2 The four backends

```rust
// 1. AnsiRenderer — always available, the contract floor.
pub struct AnsiRenderer {
    out: BufWriter<Mutex<Stdout>>,
}
// Per-cell diff: MoveTo + SetFg + SetBg + Print, per dirty cell.
// Optimized with coalesced runs (consecutive same-color dirty cells → one Print).

// 2. SyncAnsiRenderer — wraps AnsiRenderer in Begin/EndSynchronizedUpdate (DEC mode 2026).
pub struct SyncAnsiRenderer {
    inner: AnsiRenderer,
}
// Same as AnsiRenderer but: queue!(out, BeginSynchronizedUpdate)?; ...render...; queue!(out, EndSynchronizedUpdate)?; flush()?;
// The terminal holds the previous frame until the new one is complete — no partial repaints visible.

// 3. KittyAnimRenderer — kitty graphics protocol (opt-in, when TERM_CAPS.kitty_graphics == true).
pub struct KittyAnimRenderer {
    out: BufWriter<Mutex<Stdout>>,
    img_id: u32,           // persistent image id; we update it with a=f frames
    last_frame_hash: u64,  // skip re-transmit if frame unchanged
}
// Emits: <ESC>_Ga=f,i=<img_id>,s=<w>,v=<h>,c=24,t=d,f=24;<base64-rgba><ESC>\
// Then:  <ESC>_Ga=a,i=<img_id>,c=<frame_num><ESC>\  (flip to new frame)
// The TERMINAL drives the animation loop; the daemon can sleep between frame transmits.
// tmux: wrap in <ESC>Ptmux;<ESC><seq><ESC>\ when allow-passthrough is on.

// 4. WgpuHybridRenderer — GPU compute (opt-in, --gpu feature flag).
pub struct WgpuHybridRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    particle_buffer: wgpu::Buffer,        // STORAGE, particle SoA
    sim_pipeline: wgpu::ComputePipeline, // integrates particles, writes RGB grid
    readback_buffer: wgpu::Buffer,       // MAP_READ, 80×24×3 bytes
    rgb_grid: Vec<u8>,                   // CPU-side copy after map_async
}
// Pipeline: dispatch compute → copy_buffer_to_buffer → map_async →
//           map RGB grid to Cell glyphs (luminance ramp) → emit via AnsiRenderer.
```

### 4.3 Backend selection (per frame, runtime)

```rust
fn select_renderer(caps: &TermCaps, opts: &RenderOpts) -> Box<dyn Renderer> {
    if opts.gpu {
        if let Ok(wgpu) = WgpuHybridRenderer::new(PowerPreference::LowPower) {
            return Box::new(wgpu);
        }
        log::warn!("--gpu requested but wgpu init failed; falling back to CPU");
    }
    if caps.kitty_graphics && opts.damage_coverage > 0.6 {
        return Box::new(KittyAnimRenderer::new());
    }
    if caps.sync_update {
        return Box::new(SyncAnsiRenderer::new());
    }
    Box::new(AnsiRenderer::new())
}
```

**The selection is per-frame for the kitty path** (damage coverage varies), but the `Renderer` is stable within a session for wgpu (init cost). In practice: pick `AnsiRenderer` or `SyncAnsiRenderer` at startup; escalate to `KittyAnimRenderer` per-frame when damage is high; `WgpuHybridRenderer` is a session-level opt-in.

### 4.4 The hybrid rule (cardinal)

> **The software path is always correct and always available. Hardware is an accelerator, never a dependency.**

- `forgum --gpu` on a headless server → `WgpuHybridRenderer::new()` returns `Err` → `log::warn!` → `AnsiRenderer` engages. The user sees no difference.
- `forgum` in a non-kitty terminal → `KittyAnimRenderer` never engages; `SyncAnsiRenderer` or `AnsiRenderer` works.
- `forgum` inside tmux without `allow-passthrough` → kitty sequences would be stripped → detect `allow-passthrough` off → degrade to `SyncAnsiRenderer`.
- The golden framebuffer hash test runs against ALL backends; the contract is that all backends produce the same visible output for the same input.

### 4.5 Capability probe (`OnceLock<TermCaps>`)

```rust
#[derive(Clone, Debug)]
pub struct TermCaps {
    pub truecolor: bool,       // COLORTERM=truecolor or DA1 + RGB flag
    pub sync_update: bool,     // DEC mode 2026 (Begin/EndSynchronizedUpdate)
    pub kitty_graphics: bool,  // DCS Gi=31,a=q probe
    pub sixel: bool,           // DA1 bit 4
    pub iterm2_images: bool,   // TERM_PROGRAM=iTerm.app
    pub unicode_9: bool,       // wcwidth probe
    pub tmux: bool,            // $TMUX set
    pub tmux_passthrough: bool,// tmux show -gv allow-passthrough == on
    pub cols: u16,
    pub rows: u16,
}

static TERM_CAPS: OnceLock<TermCaps> = OnceLock::new();

pub fn term_caps() -> &'static TermCaps {
    TERM_CAPS.get_or_init(|| {
        // 3-stage probe, < 60ms total:
        // 1. Cheap env scan: COLORTERM, TERM_PROGRAM, TMUX, ZELLIJ_SESSION_NAME, WEZTERM_EXECUTABLE, WT_SESSION
        // 2. DA1 probe: emit CSI c, parse reply for sixel bit 4 + RGB
        // 3. Kitty probe: emit <ESC>_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA<ESC>\<ESC>[c
        //    Race the replies with a 50ms timeout. Kitty replies before DA1 if supported.
        probe_terminal_capabilities()
    })
}
```

The probe runs once, lazily, on first render. Cached in `OnceLock` for the process lifetime. Cost: one relaxed atomic load on the fast path after init.

---

## 5. The fixed-timestep accumulator scheduler

### 5.1 Why fixed-timestep (not variable `dt`)

Variable `dt` (the current `let dt = 0.016;` hack) makes simulation framerate-dependent: a 144 Hz monitor animates 2.4× faster than a 60 Hz one. Fixed-timestep decouples simulation rate from render rate:

- **Simulation runs at a strict 60 Hz** (16,666,667 ns per step), regardless of monitor refresh or terminal flush speed.
- **Rendering interpolates** between the previous and current physics state with `alpha = accumulator / dt`, so a 144 Hz render looks smoother than 60 Hz without sim running faster.

### 5.2 The accumulator (integer nanoseconds, no float drift)

```rust
pub struct Scheduler {
    // Fixed sim rate
    sim_dt_ns: i64,             // 16_666_667 for 60 Hz
    accumulator_ns: i64,        // grows each frame, drained by sim steps
    last_instant: Instant,

    // Adaptive render FPS tiers
    idle_fps: u32,              // 5
    active_fps: u32,            // 60
    current_fps: u32,
    idle_frames: u32,
    idle_threshold: u32,        // 15 frames (~250ms) — matches spiral clamp

    // For interpolation
    prev_frame: Arc<Frame>,
    curr_frame: Arc<Frame>,
}

impl Scheduler {
    pub fn tick<F, R>(&mut self, now: Instant, mut sim: F, mut render: R)
    where
        F: FnMut(i64),              // sim(dt_ns)
        R: FnMut(&Frame, &Frame, f32),  // render(prev, curr, alpha)
    {
        let frame_ns = (now - self.last_instant).as_nanos() as i64;
        self.last_instant = now;

        // Spiral-of-death clamp: if a frame took > 250ms, accept slow-motion
        // rather than piling up sim steps and freezing.
        self.accumulator_ns += frame_ns.min(250_000_000);

        // Run as many sim steps as fit in the accumulator.
        while self.accumulator_ns >= self.sim_dt_ns {
            std::mem::swap(&mut self.prev_frame, &mut self.curr_frame);
            sim(self.sim_dt_ns);
            self.accumulator_ns -= self.sim_dt_ns;
        }

        // Interpolate: alpha = how far we are into the next sim step.
        let alpha = self.accumulator_ns as f32 / self.sim_dt_ns as f32;
        render(&self.prev_frame, &self.curr_frame, alpha);
    }
}
```

### 5.3 Why `i64` nanoseconds (not `f64` seconds)

After ~3 hours of uptime, `f64` precision degrades to ~1 ms, causing visible animation jitter. `i64` nanoseconds overflow in 292 years and maintain nanosecond precision forever. Convert to `f32` seconds only when passing to physics math (the physics math doesn't need long-range precision).

### 5.4 Adaptive FPS tiers (the existing design, validated)

| Tier | FPS | When | Sleep primitive |
|------|-----|------|-----------------|
| Active | 60 | damage_count > 0 | `thread::park_timeout(sim_dt_ns / 60)` |
| Idle | 5 | 15 frames of zero damage | `thread::park_timeout(200ms)` (battery-friendly, interruptible by resize signal) |
| Heartbeat | 1 | 60 frames of zero damage (deep idle) | `thread::park_timeout(1s)` + check control socket + re-probe terminal size |

**Transition rules (hysteresis):**
- Active → Idle: 15 consecutive frames of `damage_count == 0`.
- Idle → Active: 1 frame of `damage_count > 0` (immediate, no debounce).
- Idle → Heartbeat: 60 consecutive idle frames.
- Heartbeat → Active: 1 frame of `damage_count > 0` OR a control command arrives.

### 5.5 The spiral-of-death clamp

If a sim step takes longer than `sim_dt_ns` to compute (e.g. 10k particles on a slow CPU), the accumulator grows, you run more steps next frame, you fall further behind, and the process freezes. The fix:

```rust
self.accumulator_ns += frame_ns.min(250_000_000);  // clamp at 250ms
```

If a frame took 500ms, we only accumulate 250ms — the simulation runs in slow-motion for that frame rather than spiraling. The existing `idle_threshold = 15` frames at 60 Hz = 250ms is the same magic number, validating the design.

### 5.6 Windows timer resolution

`thread::sleep` on Windows defaults to 15.6ms granularity (the global timer interrupt) — useless for 60 FPS. On the active tier, call `timeBeginPeriod(1)` (via `windows-sys::Win32::Media::timeapi`) to get 1ms granularity; restore on idle. Document the side-effect: this affects the global timer resolution for the whole process (other threads' sleeps also become 1ms-granular).

```rust
#[cfg(windows)]
fn set_high_res_timer(enable: bool) {
    use windows_sys::Win32::Media::timeapi::{timeBeginPeriod, timeEndPeriod};
    unsafe {
        if enable { timeBeginPeriod(1); } else { timeEndPeriod(1); }
    }
}
// Active tier entry: set_high_res_timer(true);
// Idle tier entry:   set_high_res_timer(false);
```

### 5.7 Battery-friendly idle (`park_timeout`, not `sleep`)

On the idle tier, `thread::sleep(200ms)` keeps the thread in the scheduler's run queue and prevents CPU deep-sleep (C-states). `thread::park_timeout(200ms)` allows the OS to fully schedule the thread out and the core to enter C7, saving ~0.5-1W on a laptop. `park_timeout` is also interruptible: a resize signal can `unpark()` the thread early, so the engine responds to resize within 1 frame instead of waiting up to 200ms.

---

## 6. The render pipeline (per frame, render thread)

```
1. Receive Arc<Frame> from frame_rx (blocks until sim ships one).
2. Send () to request_tx (tell sim to start the next frame).
3. compute_damage(front, frame): walk cells, mark dirty where front[i] != frame[i].
   - Excludes Cell::dirty from PartialEq (BUG-E1 fix).
4. Coalesce dirty cells into runs: for each row, find contiguous dirty runs
   with the same (fg, bg). Emit one MoveTo + SetFg + SetBg + Print(run) per run.
   - Bucket particles by color before rendering (one SetFg per bucket, not per particle).
5. If renderer is SyncAnsiRenderer / KittyAnimRenderer:
   - queue!(out, BeginSynchronizedUpdate)?;
6. Emit the coalesced runs.
7. If SyncAnsiRenderer / KittyAnimRenderer:
   - queue!(out, EndSynchronizedUpdate)?;
8. out.flush()?;
9. Update front = frame.clone() (only the dirty cells; or swap if using double-buffer).
10. scheduler.adapt(damage_count) — adjust FPS tier.
```

### 6.1 Coalesced run rendering (the bandwidth win)

Without coalescing, a full 80×24 repaint emits ~12,800 escape sequences (one `MoveTo` + `SetFg` + `SetBg` + `Print` per cell). With coalescing, contiguous same-color runs collapse to one `MoveTo` + one `Print(run)`:

```rust
for y in 0..h {
    let mut x = 0;
    while x < w {
        let i = y * w + x;
        if front[i] == back[i] { x += 1; continue; }  // not dirty
        let run_start = x;
        let mut run = String::new();  // <-- allocate in bumpalo arena
        let fg = back[i].fg;
        let bg = back[i].bg;
        queue!(out, MoveTo(x as u16, y as u16), SetFg(fg), SetBg(bg))?;
        while x < w && front[y*w+x] != back[y*w+x]
              && back[y*w+x].fg == fg && back[y*w+x].bg == bg {
            run.push(back[y*w+x].ch);
            x += 1;
        }
        queue!(out, Print(run))?;
    }
}
```

Typical full repaint drops from ~12,800 sequences to ~a few hundred. SSH bandwidth drops from ~300 KB/s to ~5 KB/s.

### 6.2 `BeginSynchronizedUpdate` / `EndSynchronizedUpdate` (DEC mode 2026)

These wrap a frame's output so the terminal holds the previous frame until the new one is complete. Even if rendering is interrupted mid-frame (kernel pipe buffer full, signal arrives), the user sees a clean state, not a partial repaint. Supported by: xterm, kitty, wezterm, foot, contour, Windows Terminal, gnome-terminal, Alacritty (recent). Detected via the capability probe.

---

## 7. The control thread (signals + socket, NO tty reads)

### 7.1 Signal handling via `signal-hook` (the self-pipe trick)

```rust
use signal_hook::{consts::*, iterator::Signals};

fn control_thread(control_rx: Receiver<ControlMsg>, shutdown: Arc<ShutdownFlag>) {
    let mut signals = Signals::new([
        SIGWINCH, SIGTERM, SIGINT, SIGHUP, SIGTSTP, SIGCONT,
    ]).expect("signal install");

    'outer: loop {
        // Wait for either a signal or a control-socket message.
        crossbeam_select! {
            recv(signals.forever()) -> sig => {
                match sig {
                    Ok(SIGWINCH) => control_tx.send(ControlMsg::Resize).ok(),
                    Ok(SIGTERM) | Ok(SIGINT) | Ok(SIGHUP) => {
                        shutdown.set();
                        break 'outer;
                    }
                    Ok(SIGTSTP) => {
                        // Restore terminal, then re-raise SIGTSTP to actually suspend.
                        restore_terminal_for_suspend();
                        signal_hook::low_level::raise(SIGTSTP).ok();
                    }
                    Ok(SIGCONT) => {
                        // Re-acquire terminal after suspend.
                        reacquire_terminal_after_resume();
                    }
                    _ => None,
                };
            }
            recv(control_rx) -> msg => {
                if let Ok(m) = msg {
                    control_tx.send(m).ok();  // forward to sim thread
                }
            }
        }
    }
}
```

**Critical:** this thread NEVER calls `event::poll` or `event::read` on `/dev/tty`. That was BUG-B1 — the engine stole keystrokes from the shell. The control thread only listens to signals and the control socket.

### 7.2 The SIGTSTP/SIGCONT dance (Ctrl-Z suspend)

When the user presses Ctrl-Z, the default action is to stop the process. But if raw mode is enabled (foreground mode), the terminal stays in raw mode, the shell takes over with a broken terminal, and on `fg` the user sees garbage. The correct sequence:

1. **SIGTSTP handler:** restore terminal (`disable_raw_mode`, `LeaveAlternateScreen`, `cursor::Show`, `ResetColor`), then re-raise SIGTSTP with the default action so the process actually stops.
2. **SIGCONT handler:** re-enter raw mode + alt screen + cursor hide + force a full redraw.

`signal-hook`'s `low_level::raise` re-raises with the default disposition. This is the vim/less/nano pattern.

### 7.3 The control socket (per-session, mode 0600)

```
$XDG_RUNTIME_DIR/forgum/<session>.sock   (Unix)
\\.\pipe\forgum-<session>                 (Windows named pipe)
```

`<session>` is derived from `$TMUX_PANE` (if in tmux) or the shell's PID (otherwise). Two terminals get two sockets; `forgum daemon stop` kills only the current session's daemon.

| Command | Action |
|---------|--------|
| `STOP` | graceful shutdown (clear overlay, exit 0) |
| `PAUSE` / `RESUME` | toggle `running` without exiting |
| `EFFECT <name>` | hot-swap effect at runtime |
| `SPEED <f32>` | live speed multiplier |
| `COW <file>` | reload cow, re-render |
| `STATUS` | reply JSON `{fps, frame, effect, pid, backend}` |
| `PING` | reply `PONG` (health check) |

The control thread `accept`s connections (non-blocking, polled once per loop iteration) and reads line-delimited commands, pushing `ControlMsg`s onto the channel to the sim thread.

---

## 8. The daemon lifecycle (truly detached, PID-tracked)

### 8.1 Spawn (setsid / DETACHED_PROCESS)

```rust
fn spawn_daemon(config: &Config) -> io::Result<u32> {
    let exe = env::current_exe()?;
    let mut cmd = Command::new(exe);
    cmd.arg("daemon").arg("--config").arg(config_path)
       .stdin(Stdio::null())
       .stdout(Stdio::null())
       .stderr(log_file?);

    #[cfg(unix)] {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();       // detach from controlling tty's process group
                libc::umask(0o077);   // secure file perms
                Ok(())
            });
        }
    }
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        const DETACHED: u32 = 0x00000008;
        const NEW_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED | NEW_GROUP);
    }

    let child = cmd.spawn()?;
    let pid = child.id();

    // Write daemon.json for StopDaemon, sweep, status.
    write_daemon_state(pid, &DaemonState {
        pid,
        ob_y1: config.overlay_height,
        cols: term_size.0,
        pane: detect_pane_id(),
        session: detect_session_id(),
        socket_path: control_socket_path(),
        started_at: SystemTime::now(),
    })?;

    std::mem::forget(child);  // detach — don't wait
    Ok(pid)
}
```

- `setsid()` detaches from the controlling terminal's process group → survives shell exit (BUG-B7 fix).
- `stdin(Stdio::null())` → the daemon never touches the shell's input.
- `stdout/stderr → log file` → no stray bytes on the user's terminal.
- `daemon.json` → `StopDaemon`, `sweep`, `forgum daemon status` all read it.

### 8.2 The `precmd` sweep (kill -9 safety net)

`SIGKILL` skips `Drop`. If a user `kill -9`s the daemon, the overlay stays. The shell hook's `precmd` runs `forgum daemon sweep`:

```bash
__forgum_precmd() {
  if [ -f "$FORGUM_RUNTIME/daemon.json" ]; then
    pid=$(jq -r .pid "$FORGUM_RUNTIME/daemon.json")
    if ! kill -0 "$pid" 2>/dev/null; then
      # daemon died ungracefully — sweep the overlay
      rows=$(jq -r .ob_y1 "$FORGUM_RUNTIME/daemon.json")
      cols=$(jq -r .cols "$FORGUM_RUNTIME/daemon.json")
      printf '\x1b7'
      for y in $(seq 1 "$rows"); do
        printf '\x1b[%d;1H%*s' "$y" "$cols" ''
      done
      printf '\x1b8\x1b[0m'
      rm -f "$FORGUM_RUNTIME/daemon.json"
    fi
  fi
}
PROMPT_COMMAND="__forgum_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

This is the safety net that makes the feature bulletproof even against `kill -9`.

---

## 9. Workspace layout (the `crates/` split)

```
Forgum/
├── Cargo.toml                    # [workspace]
├── crates/
│   ├── platform/                 # forgum-platform — ALL #[cfg] lives here
│   │   ├── src/{lib,unix,windows,macos,signal,output,paths,shell,mux,socket,caps}.rs
│   │   └── tests/
│   ├── engine/                   # forgum-engine binary — zero #[cfg]
│   │   ├── src/{main,cli,config,cow,bubble,effects,particles,framebuffer,
│   │   │         region,scheduler,color,style_matcher,terminal,render,daemon,
│   │   │         herd,remote,theme,tmux,init,completions,
│   │   │         animation/{easing,verlet,layer,dna}}.rs
│   │   ├── tests/{alloc_budget,golden_visual,pty_keystroke,resize_cleanliness}.rs
│   │   └── benches/{engine,anim}.rs
│   └── protocol/                 # forgum-protocol — SceneConfig, ControlMsg, shared types
├── data/Cows data/Fortunes data/Templates data/animations.json   # bundled via include_dir!
├── Forgum.psd1 Forgum.psm1 Public/ Private/  # PowerShell module (thin)
├── scripts/completions/                       # generated, never hand-edited
├── install.sh install.ps1 setup.ps1
├── package-managers/{homebrew,scoop,winget,deb,rpm,aur}
├── Tests/                                     # Pester
└── .github/workflows/                         # CI matrix
```

**Rule:** `crates/engine/src/*.rs` contains **zero** `#[cfg]`. All platform branching lives in `crates/platform/`. The engine programs against `forgum_platform::TerminalHandle` / `Spawner` / `Paths` traits. This makes the engine logic portable and testable on a single host.

---

## 10. The `Cargo.toml` dependency matrix

```toml
[workspace.dependencies]
# Core
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
include_dir = "0.7"

# Terminal
crossterm = { version = "0.28", features = ["event-stream"] }

# Concurrency
crossbeam-channel = "0.5"
crossbeam-queue = "0.3"
rayon = "1.10"
signal-hook = "0.3"

# Memory
bumpalo = "3.16"
slotmap = "1.0"

# Animation
ezing = "0.2"              # easing functions
palette = "0.7"            # OKLCH color

# Logging
log = "0.4"
env_logger = "0.11"

# Optional / feature-gated
wgpu = { version = "22", optional = true }      # --gpu feature
rand = { version = "0.8", features = ["small_rng"] }

[features]
default = []
gpu = ["dep:wgpu"]
audio = ["dep:cpal", "dep:rustfft"]   # Phase 8 reactive effects

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"            # smaller binary; we use catch_unwind for terminal cleanup
```

**Note on `panic = "abort"`:** this shrinks the binary by removing unwind tables, but means `catch_unwind` won't work. The resolution: keep `panic = "unwind"` (default) for the engine crate, use `abort` only for the CLI shim. The RAII guards' `Drop` runs during unwinding; we need that.

---

## 11. Summary: why this is "professional, not a cheap knock-off"

| Aspect | Cheap knock-off | Forgum v2 (this doc) |
|--------|-----------------|----------------------|
| Threading | Single-threaded, blocking sleep | 3 threads (sim/render/control), scoped, no shared mutable state |
| Singletons | `static mut` or `lazy_static!` | `OnceLock`/`LazyLock` DCL, no `unsafe`, no `static mut` |
| Memory | `Vec<bool>` + O(n) spawn scan, unbounded allocs | `bumpalo` arena (zero-alloc hot loop), `slotmap` O(1) spawn, `dhat` CI gate |
| Rendering | One ANSI path, per-cell escape codes | `Renderer` trait, 4 backends, runtime selection, transparent fallback |
| Scheduler | `let dt = 0.016;` hardcoded | Fixed-timestep accumulator, integer ns, spiral clamp, 3-tier adaptive FPS |
| Signals | None; `Drop` is a no-op | `signal-hook` self-pipe, SIGTSTP/CONT dance, RAII + `catch_unwind` |
| Particles | SoA with linear scan | `slotmap` generational, `rayon` parallel integration, color bucketing |
| Color | HSV (non-uniform) | OKLCH (perceptually uniform), 256-color dithering fallback |
| Animation | `let dt = 0.016` + `time += dt * 50.0` | Easing functions per base, Verlet chains, per-instance phase randomization |
| Testing | A few unit tests | 6-tier pyramid: unit, integration, E2E-under-tmux, fuzz, bench, golden-visual + pHash |

This is the engine a senior Rust engineer ships.

---

**Companion documents:**
- `10-FINE-TUNED-MASTER-PLAN.md` — the phased plan.
- `12-PER-ANIMAL-ANIMATION-DESIGN.md` — the 7-axis DNA profile and the 109-cow table.
- `13-TEST-COVERAGE-MATRIX.md` — the test pyramid and per-phase gates.
