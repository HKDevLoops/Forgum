# Phase 7 — Demo + Follow + Rotate Design Spec

> The showcase command, focus-aware follow, and theme rotation.

---

## Deliverables

| # | Feature | Description |
|---|---------|-------------|
| D1 | `forgum demo` | Showcase: quiet → popup → aurora → rotate → follow |
| D2 | `forgum theme rotate [--interval N]` | Cycle through themes every N minutes |
| D3 | `forgum herd follow [--pane X]` | Only focused pane animates; others idle |

---

## D1: `forgum demo`

Runs in sequence:
1. `herd quiet` — calm the fleet
2. Print dramatic reveal text (3s pause)
3. `herd effect aurora --all` — set aurora on all daemons
4. Print success message

For tmux: also print the tmux config suggestion for popup and follow.

---

## D2: `forgum theme rotate`

`forgum theme rotate [--interval N]` — applies a random theme every N minutes.
Reads available themes, picks one at random, applies, sleeps, repeats.
Runs until Ctrl-C.

---

## D3: `forgum herd follow`

`forgum herd follow [--pane X]` — sets all daemons to idle except the specified pane (or current pane).
Uses the control socket to send `SPEED 0.1` to non-focused daemons and `SPEED 1.0` to the focused one.
