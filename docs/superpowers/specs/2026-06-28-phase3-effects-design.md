# Phase 3 — Per-Animal Animation DNA + Effects Polish

> **Goal:** effects look right, perform well, and use per-cow styling.
> Every cow animates uniquely with its own speed, easing, color palette,
> particle behavior, and phase randomization. `[dep: 1]` `[par: 2]`

**Invariants hardened:** #8 (every cow unique), #4 (zero-alloc in particle path).

---

## 1. Architecture

```
animations.json  ──► CowDna loader ──► EffectConfig
                                            │
                    ┌───────────────────────┘
                    ▼
            Effect trait
            ├── BreatheEffect
            ├── FloatEffect
            ├── WalkEffect
            ├── FireEffect (particles)
            ├── PulseEffect
            ├── GlitchEffect
            ├── FlyEffect
            ├── TalkEffect
            ├── SwayEffect
            └── DissolveEffect
                    │
                    ▼
            render_loop calls effect.update(dt) + effect.render(fb)
```

---

## 2. Task Breakdown

| Task | Bug | Deliverable |
|------|-----|-------------|
| 3.1 Easing functions | (new) | 8 functions: linear, sine_inout, cubic_inout, cubic_out, cubic_in, back_out, expo_out, bounce_out |
| 3.2 Effect trait | E3 | `update(dt, cols, rows)`, `render(fb)`, `is_done()`, `on_resize(w, h)` |
| 3.3 DNA schema | E3 | `CowDna` struct + `animations.json` loader |
| 3.4 Color module | (new) | OKLCH gradient, lolcat rainbow, 256-color dither, gaussian glow |
| 3.5 Particle system | E16/17 | `ParticlePool` with Fire/Bubbles/Stars/Zzz/Pulse/Glitch types |
| 3.6 Verlet chains | (new) | `VerletChain<N>` for tails/capes |
| 3.7 Phase randomization | (new) | `(phase_seed ^ instance_id) * 0.618` golden ratio |
| 3.8 Coalesced rendering | F1 | Row-run coalescing + fg/bg color in render_damage |
| 3.9 Render loop integration | (new) | `update(dt)` + `render(fb)` per frame |

---

## 3. Test Gate (Phase 3 DoD)

- [ ] All 8 easing functions produce correct values at t=0, t=0.5, t=1
- [ ] Effect trait dispatch works for all 10 base types
- [ ] DNA loader reads animations.json correctly
- [ ] OKLCH gradient produces smooth color transitions
- [ ] Fire particles spawn, move, and die correctly
- [ ] Verlet chain maintains distance constraints
- [ ] Phase randomization produces different values for different seeds
- [ ] Gaussian glow falls off correctly
- [ ] 256-color quantization produces valid xterm indices
- [ ] All existing tests still pass
- [ ] Clippy clean, fmt clean, cfg-grep clean
