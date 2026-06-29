# Phase 4 — Render Pipeline & Real Effects

## Problem

Phase 3 built 10 effect types, a particle system, Verlet chains, and a DNA
schema — but **none of it is wired into the render loop**. The render loop
calls `effects::render_static_cow()` directly, ignoring all animation.
7 of 10 effects are scaffolded stubs that compute math and discard it.
Verlet chains are standalone with zero callers.

## Goal

Wire the effect system into the render loop, implement all 7 stubbed effects
with real animation, integrate Verlet chains for secondary motion, and
extract a `Renderer` trait for backend abstraction.

## Deliverables

| # | Deliverable | Files |
|---|-------------|-------|
| 4.1 | Effect dispatch in render loop | `render.rs`, `main.rs` |
| 4.2 | BreatheEffect: vertical oscillation | `effects.rs` |
| 4.3 | FloatEffect: whole-art drift | `effects.rs` |
| 4.4 | WalkEffect: leg character swap | `effects.rs` |
| 4.5 | FlyEffect: erratic float + flap | `effects.rs` |
| 4.6 | TalkEffect: mouth animation | `effects.rs` |
| 4.7 | SwayEffect: pendulum skew | `effects.rs` |
| 4.8 | DissolveEffect: scatter/reassemble | `effects.rs` |
| 4.9 | Verlet integration in effects | `effects.rs`, `verlet.rs` |
| 4.10 | Renderer trait + AnsiRenderer | `renderer.rs`, `render.rs` |
| 4.11 | tmux passthrough wrapping | `renderer.rs` |
