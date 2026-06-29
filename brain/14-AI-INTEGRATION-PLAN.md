# Forgum — AI Integration Plan (Future Horizon, v3)

> **The "make it think" blueprint.** This document defines how Forgum graduates from "a cowsay clone with colors" into **a terminal companion that understands your commands, reads your shell's mood, and renders the right cow saying the right thing at the right moment** — without ever slowing down your prompt or leaking your secrets to the cloud.
>
> It is the v3 horizon, layered *on top of* the v2 fine-tuned master plan (`10-…`). Nothing in v2 is contradicted; v3 adds a pluggable **`AiEngine`** that augments — never blocks — the render path. Read this after `10-FINE-TUNED-MASTER-PLAN.md`, `11-ENGINE-INTERNALS-V2.md`, and `12-PER-ANIMAL-ANIMATION-DESIGN.md`.
>
> **The one-sentence pitch:** when you type `git push` and it succeeds, Forgum shows a triumphant dragon exhaling particles of victory with the thought *"the kingdom ships tonight"*; when the same command fails, it shows a confused turtle muttering *"the gate refused us — check thy credentials, traveler."* The cow is no longer random. It is *contextual*.

---

## 0. Vision: from animated to aware

Forgum v2 makes cows **look alive**. Forgum v3 makes them **act aware**. Concretely, the terminal companion should:

1. **Classify** every cow file in its library along a 9-axis mood/trait DNA — once, offline, cached forever — so each cow carries a semantic identity, not just ASCII art.
2. **Listen** (locally, with strict redaction) to the command you just ran, its exit code, your git state, the time of day, and your recent shell rhythm.
3. **Select** the cow whose semantic identity best fits the current moment — not randomly, not by `cowsay -n`, but by **embedding-space nearest-neighbor** between *command intent* and *cow mood*.
4. **Compose** the cow's thought via a language model — contextual, tonally matched, optionally in your language, never repeating verbatim within a session.
5. **Tune** the animation (palette, speed, particle type, easing) to reflect the **sentiment** of the moment — green-gold for success, ember-red for failure, slow-breathe for idle.
6. **Speak** (optionally, via TTS) a moo that matches the cow's mood — a triumphant dragon's moo is *not* a sleepy cat's moo.
7. **Converse** (optionally) — a cow persona that knows your project, can explain why a command failed, suggest the next one, or roast your last commit.

All of this is **local-first** (your shell history never leaves the machine unless you explicitly opt in), **non-blocking** (the prompt renders in <5 ms from cache; AI work happens in the background and lands on the *next* render cycle), and **gracefully degraded** (no models installed → v2 behavior; cloud unreachable → local fallback; CPU too slow → embeddings-only).

This document specifies the **what**, the **why**, the **architecture**, the **phased rollout**, the **privacy model**, the **test/eval harness**, and the **risks**. It does not duplicate the rendering/scheduling/DNA work already specified in v2.

---

## 1. The AI capability matrix — 25 features, 5 tiers

Not all AI features are equal. Some are critical-path-adjacent (must be cached), some are background (can take seconds), some are user-invoked (can take minutes). The matrix below classifies every candidate feature by **tier** (latency budget), **engine** (which AI primitive it needs), **privacy class**, and **phase** (when it ships).

**Tier legend:** T0 = on prompt-render critical path (≤5 ms, cache-only), T1 = near-real-time background (≤300 ms), T2 = async background (≤3 s), T3 = user-invoked (≤10 s), T4 = batch/offline (minutes, scheduled).

**Privacy class:** L = local-only (never leaves machine), LO = local-preferred with opt-in cloud, C = cloud-only (explicit opt-in per call).

**Engine:** E = embeddings, L = LLM (text), V = VLM (vision), T = TTS, S = STT, I = image-gen, H = heuristics (no model, just rules).

| # | Feature | Tier | Engine | Privacy | Phase | One-liner |
|---|---------|------|--------|---------|-------|-----------|
| F01 | **Cow mood classification** | T4 | V+E | L | A | Auto-tag all 109 cows with a 9-axis mood DNA. |
| F02 | **Command intent classification** | T1 | E+H | L | B | Embed the just-run command, match to intent cluster. |
| F03 | **Command→cow semantic selection** | T1 | E | L | B | Pick the cow whose mood embedding is nearest the command intent. |
| F04 | **Contextual thought generation** | T2 | L | LO | C | LLM writes the cow's thought from command+exit+git context. |
| F05 | **Sentiment-adaptive palette/speed** | T1 | H+E | L | B | Map exit code + intent to OKLCH palette + speed multiplier. |
| F06 | **Natural-language cow search** | T3 | E+L | L | C | `"forgum show me a sleepy dragon"` → semantic cow retrieval. |
| F07 | **Tone modes** | T2 | L | LO | C | Sarcastic / zen / hype / doom / pirate persona switch. |
| F08 | **Error explanation cow** | T3 | L | LO | D | Failed command → cow explains why in plain language. |
| F09 | **Suggest-next-command cow** | T3 | L+H | LO | D | Cow proposes the next reasonable command. |
| F10 | **Git commit message generation** | T3 | L | LO | D | `forgum commit` → cow drafts the message from `git diff`. |
| F11 | **Code-review cow** | T3 | L | LO | D | Cow comments on a diff/PR with humor + substance. |
| F12 | **Predictive cow pre-render** | T2 | E+H | L | C | Learn command rhythm; pre-render the likely next cow. |
| F13 | **Voice moos (TTS)** | T3 | T | LO | E | Each cow has a voice; mood changes the moo. |
| F14 | **Voice commands (STT)** | T3 | S | L | E | `"hey cow, dragon"` hands-free. |
| F15 | **Custom cow generation (text→ASCII)** | T3 | L | LO | F | `"forgum new "cyberpunk cow with neon horns"` → new .cow file. |
| F16 | **VLM cow-art analysis** | T4 | V | L | D | Verify rendered frame matches intent; detect rendering bugs. |
| F17 | **Multi-language thoughts** | T2 | L | LO | C | Generate the thought in the user's `$LANG`. |
| F18 | **Personalized tone learning** | T4 | L | L | F | Learn which thoughts the user starred/ignored. |
| F19 | **Smart herd composition** | T3 | E+L | L | F | AI picks N complementary cows for a herd (no mood clashes). |
| F20 | **Mood-adaptive coloring (per-frame)** | T2 | E+H | L | C | Slowly drift palette toward sentiment over the session. |
| F21 | **Shell-history daily digest** | T4 | L | L | F | `"forgum day"` → cow summarizes your day in 3 thoughts. |
| F22 | **AI fortune feed (news/weather)** | T4 | L+web | C | F | Cow comments on today's weather/news (opt-in). |
| F23 | **Pair-programming cow (tmux)** | T3 | L+V | LO | G | Cow watches a tmux pane, comments as you code. |
| F24 | **Accessibility narrator** | T2 | L+T | L | E | Describe the animation for screen readers; speak the thought. |
| F25 | **Conversational cow REPL** | T3 | L+S+T | LO | G | `forgum chat` → full persona, voice in/out, project-aware. |

**The phasing** (A–G) is detailed in §9. The principle: **ship the cheapest high-value feature first** (F01 classification is a one-time offline batch job that unlocks F03 selection), **gate everything cloud behind explicit opt-in**, and **never let AI touch the prompt critical path except via cache**.

---

## 2. Core concept: the cow classification taxonomy

Today, Forgum's 109 cows are just files in a directory. The shell hook picks one at random (BUG-S3 — it reads the wrong config key). v3 gives every cow a **semantic identity**.

### 2.1 The 9-axis mood DNA (extends the v2 7-axis animation DNA)

v2's 7-axis DNA (`base`, `particles`, `speed`, `amplitude`, `palette`, `easing`, `phase_seed`) describes **how a cow moves**. v3 adds a parallel 9-axis DNA that describes **what a cow *means***:

| Axis | Type | Range | Example (dragon) | Example (turtle) |
|------|------|-------|-------------------|-------------------|
| `energy` | f32 | 0.0–1.0 | 0.85 (hyper) | 0.15 (sluggish) |
| `menace` | f32 | 0.0–1.0 | 0.80 (scary) | 0.05 (harmless) |
| `whimsy` | f32 | 0.0–1.0 | 0.30 (serious) | 0.70 (silly) |
| `majesty` | f32 | 0.0–1.0 | 0.90 (regal) | 0.25 (humble) |
| `chaos` | f32 | 0.0–1.0 | 0.60 (wild) | 0.10 (placid) |
| `warmth` | f32 | 0.0–1.0 | 0.20 (cold) | 0.80 (cozy) |
| `focus` | f32 | 0.0–1.0 | 0.70 (intent) | 0.40 (dreamy) |
| `archetype` | enum | 8 variants | `Monster` | `Slow` |
| `domain_tags` | Vec<String> | open set | `["fire","sky","predator"]` | `["water","patience","shell"]` |

The 8 `archetype` variants: `Hero`, `Monster`, `Trickster`, `Slow`, `Cute`, `Sage`, `Chaotic`, `Neutral`. Every cow maps to exactly one primary archetype; `domain_tags` carries the rest.

### 2.2 How cows get tagged (F01, the offline batch job)

```
forgum ai classify --all
```

runs **once** (and re-runs only when `.cow` files change, tracked by a content hash). The pipeline:

1. **Render** each cow to a PNG/Sixel/kitty-graphics frame (using the v2 software renderer — no GPU needed).
2. **VLM pass**: feed the rendered frame + the raw `.cow` ASCII to a vision-language model with the prompt *"Score this character on energy/menace/whimsy/majesty/chaos/warmth/focus (0–1 each), assign one archetype from {Hero,Monster,Trickster,Slow,Cute,Sage,Chaotic,Neutral}, and list 3–7 domain tags. Output JSON."*
3. **LLM arbiter pass**: a second model reviews the VLM JSON for internal consistency (e.g., a cow scored `menace:0.9` + `archetype:Cute` is flagged and re-scored).
4. **Embedding pass**: compute a 384-dim sentence embedding of a synthesized natural-language description (e.g., *"a hyperactive menacing serious regal wild cold focused fire-sky-predator monster"*) using `fastembed` (BGE-small-en, CPU, ~10 ms).
5. **Persist**: write `cow_dna.json` — `{ cow_id, mood_dna(9-axis), embedding(384), source_hash, classified_at, classifier_version }`.

**Backends:** local-first via `candle` (LLaVA-1.5-7B Q4, ~4 GB) or cloud (GPT-4o-mini / Claude Haiku / z-ai VLM endpoint) when the user opts in for the one-time batch. Cloud is *faster* for the batch but never required.

**Cost:** ~109 VLM calls × ~$0.0005 (Haiku) = **~$0.05** to classify the whole library once via cloud; ~8 minutes locally on a modern laptop with LLaVA-Q4. Trivial.

**The cache invariant:** `cow_dna.json` is content-addressed by the `.cow` file hash. A CI test asserts that adding a new cow without re-classifying fails the build — classification is a first-class build artifact, not a runtime surprise.

### 2.3 The classification is *advisory*, not *authoritative*

Users can override any axis per-cow in `~/.forgum/overrides.json`:

```json
{ "dragon": { "menace": 0.95, "domain_tags": ["fire","sky","predator","ancient"] } }
```

The override merges over the auto-classified DNA. This keeps the AI honest (a misclassification is a 5-second fix, not a model retrain) and respects user taste (your dragon *is* more menacing than the VLM thinks).

---

## 3. The flagship feature: command-aware cow selection (F02 + F03 + F05)

This is the feature the user asked for: **"AI for classification of cow files wrt thoughts according to users commands on the terminal."**

### 3.1 The selection pipeline (end-to-end)

```
   USER RUNS A COMMAND (e.g. `git push`)
                │
                ▼
┌───────────────────────────────────────────────────────────┐
│  SHELL PRECMD / PRECMD-like hook (forgum fn)              │
│  gathers context signals (all local, all redacted):       │
│    • last_command        = "git push"                      │
│    • last_exit_code      = 0                               │
│    • cwd_basename        = "forgum"                        │
│    • git_branch          = "main"                          │
│    • git_ahead_behind    = (2, 0)                          │
│    • git_dirty           = false                           │
│    • time_of_day         = "night"                         │
│    • session_idle_secs   = 12                              │
│    • recent_cmd_rhythm   = [build, test, push]             │
└───────────────────────────────────────────────────────────┘
                │
                ▼  (≤2 ms, pure local heuristics)
┌───────────────────────────────────────────────────────────┐
│  INTENT CLASSIFIER  (F02)                                 │
│  • fast rules first: regex → intent cluster                │
│    (git:*, cargo:*, npm:*, docker:*, ssh:*, rm:*, ...)     │
│  • if no rule hits: embed the command, NN-search the       │
│    intent-cluster centroids (384-dim, ~1 ms via hora)      │
│  • output: IntentVector {                                  │
│      cluster: "ship",                                      │
│      sentiment_seed: +0.8 (success-biased by exit code),  │
│      energy: 0.7,                                          │
│      focus: 0.8,                                           │
│      urgency: 0.4,                                         │
│      domain_tags: ["git","network","publish"]              │
│    }                                                       │
└───────────────────────────────────────────────────────────┘
                │
                ▼  (≤1 ms, cache hit expected)
┌───────────────────────────────────────────────────────────┐
│  COW SELECTOR  (F03)                                      │
│  • build a query embedding from the IntentVector's         │
│    natural-language rendering                              │
│    ("successful focused energetic git-network-publish     │
│     ship moment")                                          │
│  • cosine-NN over the 109 cow mood embeddings (hora HNSW)  │
│  • apply hard filters: never repeat within 5 commands,     │
│    respect user blocklist, weight by archetype match       │
│  • output: cow_id = "dragon"                               │
└───────────────────────────────────────────────────────────┘
                │
                ▼  (≤2 ms, table lookup)
┌───────────────────────────────────────────────────────────┐
│  SENTIMENT TUNER  (F05)                                   │
│  • exit_code 0 + intent "ship" → palette "victory",        │
│    speed_mult 1.15, particle "embers", easing "easeOutBack"│
│  • exit_code != 0 → palette "ember", speed_mult 0.7,       │
│    particle "smoke", easing "easeOutCubic"                 │
│  • overrides v2 DNA's palette/speed for *this render only* │
└───────────────────────────────────────────────────────────┘
                │
                ▼  (≤5 ms total — T0 budget met)
        ENGINE RENDER (v2 pipeline, unchanged)
                │
                ▼
   ANIMATED DRAGON, VICTORY PALETTE, ON THE PROMPT

   meanwhile, in the background (T2, async, lands next cycle):
┌───────────────────────────────────────────────────────────┐
│  THOUGHT GENERATOR  (F04)                                 │
│  • LLM call (local Llama-3.2-1B or cloud) with context:   │
│    "A majestic dragon just watched 'git push' succeed.    │
│     Branch main, 2 commits ahead. It is night.            │
│     Tone: regal, brief, mythic. Write ≤12 words."          │
│  • output: "the kingdom ships tonight"                     │
│  • cached by hash(command+exit+git+cow+tone)               │
│  • lands in the NEXT render cycle's thought bubble         │
└───────────────────────────────────────────────────────────┘
```

**Total critical-path cost: ≤5 ms.** The LLM thought runs async and lands on the *next* precmd. This is the non-negotiable design rule: **AI never blocks the prompt.**

### 3.2 The intent cluster taxonomy (the "command vocabulary")

There are **24 intent clusters**, each with a hand-seeded regex and a learned embedding centroid:

| Cluster | Regex seeds (examples) | Default sentiment bias |
|---------|------------------------|------------------------|
| `build` | `cargo build`, `make`, `npm run build`, `tsc` | neutral → success on 0 |
| `test` | `cargo test`, `pytest`, `jest`, `go test` | neutral → success on 0 |
| `ship` | `git push`, `npm publish`, `cargo publish`, `docker push` | success-biased |
| `fetch` | `git pull`, `cargo fetch`, `npm install`, `pip install` | neutral |
| `init` | `cargo new`, `npm init`, `mkdir`, `git init` | success-biased |
| `destroy` | `rm -rf`, `git clean -fd`, `docker rm` | menace-biased |
| `travel` | `ssh`, `scp`, `rsync`, `mosh` | neutral, focus-biased |
| `search` | `rg`, `grep`, `find`, `fd` | focus-biased |
| `read` | `cat`, `less`, `bat`, `head` | calm-biased |
| `edit` | `vim`, `nvim`, `code`, `emacs` | focus-biased |
| `serve` | `cargo run`, `npm start`, `python -m http.server` | energy-biased |
| `inspect` | `ps`, `top`, `htop`, `lsof`, `netstat` | focus-biased |
| `network` | `curl`, `wget`, `httpie`, `ping` | neutral |
| `package` | `brew`, `apt`, `pacman`, `winget` | neutral |
| `version` | `git status`, `cargo --version`, `node -v` | neutral |
| `commit` | `git commit`, `git add` | focus-biased |
| `branch` | `git checkout`, `git switch`, `git merge` | neutral |
| `container` | `docker`, `podman`, `kubectl` | neutral |
| `secret` | `*token*`, `*key*`, `*password*`, `aws s3` | menace-biased, **redacted** |
| `danger` | `sudo`, `chmod 777`, `dd of=/dev/` | menace-biased |
| `idle` | (no command in 60s) | warmth-biased |
| `error_recovery` | (last exit != 0) | calm-biased |
| `first_of_day` | (session start) | whimsy-biased |
| `unknown` | (no rule, no NN hit > 0.6) | neutral (random cow, v2 behavior) |

**The `secret` cluster is special:** commands matching it are **never sent to any cloud LLM**, even if the user opted into cloud. The redaction pipeline (§7) scrubs them to `"***REDACTED***"` before any model sees context. This is enforced by a CI test that runs the pipeline against a corpus of 200 secret-looking commands and asserts zero leakage.

### 3.3 The nearest-neighbor index

The 109 cow mood embeddings + the 24 intent-cluster centroids live in an in-process **`hora` HNSW index** (Rust, no daemon, ~50 KB on disk, sub-millisecond queries). Rebuilt on startup from `cow_dna.json` in <10 ms. No external vector DB — Forgum stays a single self-contained binary.

---

## 4. AI thought / fortune generation (F04 + F07 + F17)

### 4.1 The thought is *the cow speaking in character*

v2 cowsay shows a random fortune. v3 shows a **contextual thought in the selected cow's voice**. The prompt template:

```
SYSTEM: You are {cow_id}, a {archetype} character.
Your mood DNA: energy={e}, menace={m}, whimsy={w}, majesty={maj},
chaos={c}, warmth={wa}, focus={f}.
Your domain: {domain_tags}.
Tone mode: {tone_mode}  (one of: default, sarcastic, zen, hype, doom, pirate)
Language: {user_lang}

USER: You just witnessed this terminal moment:
  command: {redacted_command}
  exit code: {exit_code}  ({success|failure})
  context: {cwd_basename}, git {branch} ({ahead}↑{behind}↓, {dirty}), {time_of_day}
  session rhythm: {recent_cmd_rhythm}

Write ONE thought for your speech bubble. Rules:
  - ≤ {max_words} words (default 12, configurable)
  - stay in character and tone
  - never reveal the command verbatim
  - never include code, paths, or secrets
  - if exit code != 0, be helpful not mocking
  - output the thought text only, no quotes, no preamble
```

### 4.2 Tone modes (F07)

| Mode | Personality | Example thought (on `git push` success) |
|------|-------------|------------------------------------------|
| `default` | matches cow DNA | "the kingdom ships tonight" |
| `sarcastic` | dry, undercutting | "wow. a push. alert the bards." |
| `zen` | calm, present | "the commit flows. the branch is still." |
| `hype` | maximal enthusiasm | "LET'S GOOOO MAIN IS LIVE 🔥" (emoji off by default) |
| `doom` | bleak, funny | "you shipped. the heat death is unchanged." |
| `pirate` | yarrr | "ye cargo's aloft, cap'n. fair winds." |

Switchable per-session (`forgum tone doom`) or permanently in config. The mode is injected into the system prompt; the cow DNA still constrains the voice.

### 4.3 Caching (the L3 cache)

Thoughts are cached by `blake3(intent_vector || cow_id || tone_mode || lang || git_state_hash)`. A hit returns in <1 ms. The cache:

- **Persists** to `~/.forgum/cache/thoughts.db` (sled, ~1 MB max, LRU-evicted).
- **Never** stores the raw command — only the hash + the generated text. The command itself is irrecoverable from the cache. (Privacy: §7.)
- **Session de-dup:** within a session, the same intent+cow combo rotates through 3 cached variants before repeating, so the dragon doesn't say the same thing every push.

### 4.4 Backends

| Backend | When | Latency | Privacy | Cost |
|---------|------|---------|---------|------|
| Local Llama-3.2-1B Q4 (`candle`) | default | 0.8–2.5 s | L | free |
| Local Phi-3.5-mini Q4 | if 4 GB+ RAM | 1.5–4 s | L | free |
| Cloud (z-ai / OpenAI / Anthropic) | opt-in, `forgum ai cloud on` | 0.3–1.2 s | LO | ~$0.0001/thought |
| Heuristic fallback | no model installed | <1 ms | L | free (v2 fortune) |

**The fallback is mandatory:** if no LLM is available and the cache misses, Forgum falls back to the v2 fortune file. The cow still renders, the thought is still shown, the prompt is never blocked. AI is augmentation, never a dependency.

---

## 5. The "many more" — full feature catalog (deep dive on F08–F25)

### F08 — Error explanation cow
On non-zero exit, the cow's thought becomes a **plain-language hint**. The LLM is fed `{command, exit_code, stderr_tail (redacted)}` and asked for a ≤20-word hint + a suggested fix. Example: `cargo build` fails with linker error → dragon thinks *"the linker cannot find `openssl` — try `pkg-config` or set `OPENSSL_DIR`."* The stderr is **redacted** (paths under `$HOME` → `~/`, env-var values → `***`) before it touches any model.

### F09 — Suggest-next-command cow
After a successful `git push`, the cow's *second* thought bubble (a v3 feature: dual-bubble support) suggests `"git checkout dev && git pull?"`. Driven by a small fine-tuned classifier over command→next-command pairs (trained on the user's *own* anonymized history, local-only). Opt-in. Never sends history to cloud.

### F10 — Git commit message generation (`forgum commit`)
`forgum commit` runs `git diff --cached`, feeds it to the LLM with a Conventional Commits prompt, prints 3 candidate messages, and lets the user pick (fzf). Writes the commit. Respects `.gitmessage` template. **Never auto-commits** — always human-approved.

### F11 — Code-review cow (`forgum review [SHA]`)
Feeds `git diff SHA~1 SHA` to an LLM with a review prompt ("find bugs, style issues, and one thing you liked"). Outputs cow-branded comments to stdout or posts to GitHub PR via `gh` (if installed and authed). Opt-in, cloud-recommended (local models are weak at code review).

### F12 — Predictive cow pre-render
The engine keeps a **3-deep prediction queue**: based on the last 5 commands' rhythm (Markov chain over intent clusters), it pre-classifies the 3 most likely next intents and pre-warms their cow render + thought. When the user hits enter, the render is already in cache → 0 ms perceived latency. Runs on the control thread (v2's 3rd thread), never competes with the render thread.

### F13 — Voice moos (TTS)
Each cow archetype gets a voice profile: `Hero` = bright tenor, `Monster` = low growl, `Cute` = high squeak, `Sage` = slow baritone, `Chaotic` = pitched-up chipmunk. The "moo" is synthesized via the **TTS skill** (z-ai-web-dev-sdk TTS endpoint) or locally via **Piper** (`piper-rs`, ~30 MB voice models). Mood modulates pitch/speed: a victorious dragon's moo is a triumphant roar; a sleepy turtle's is a long low rumble. **Off by default** — must be explicitly enabled (`forgum voice on`). Respects `--quiet` and `TERM=dumb`.

### F14 — Voice commands (STT)
`forgum listen` opens the mic (via `whisper-rs`, Whisper-tiny.en, ~75 MB, CPU, ~300 ms latency) and waits for `"hey cow, <command>"`. Wake-word is a small VAD + keyword-spot model. Maps spoken intents to Forgum subcommands: *"show me a sleepy dragon"* → `forgum render --cow dragon --mood sleepy`. **Pure local.** Never records except during the active listen window.

### F15 — Custom cow generation (`forgum new "<description>"`)
`forgum new "a cyberpunk cow with neon horns and a cable tail"` calls an LLM with a **text-to-ASCII-art** prompt constrained to the cowsay art grammar (fixed width, `\\` speech-bubble-compatible, no wide chars). The output is validated by the v2 cow parser; if it fails, the model retries with the parse error in context. Saves to `~/.forgum/cows/cyberpunk.cow`, auto-classifies it (F01), and it's immediately selectable. Cloud-recommended (local 1B models struggle with constrained ASCII art); local fallback is a template-based procedural generator.

### F16 — VLM cow-art analysis
Two uses:
- **Build-time QA:** the VLM renders each cow and checks "does this look like a {archetype}?". Cows that fail are flagged in `cow_dna.json` with `"confidence": low` and excluded from semantic selection until a human reviews.
- **Runtime frame audit:** every Nth rendered frame is sampled, VLM-checked for "is the cow visible, centered, not clipped?", and anomalies are logged. This catches renderer regressions (kitty-graphics fallback, resize bugs) automatically.

### F17 — Multi-language thoughts
`forgum lang es` → thoughts generated in Spanish. The LLM is prompted in the target language with a translation-of-tone instruction. Local Llama-3.2-1B is multilingual; cloud models are better. Fallback: English.

### F18 — Personalized tone learning
Forgum tracks which thoughts the user "stars" (`forgum star` after a render — or implicitly, thoughts followed by a quick next command vs. a long pause). A local preference model (logistic regression over feature vectors) nudges future generations toward starred patterns. **All local, all anonymous.** No cloud sync. Reset with `forgum reset-personality`.

### F19 — Smart herd composition
`forgum herd --ai 5` asks the LLM/embeddings to pick 5 cows that **complement** each other (no two of the same archetype, varied energy levels, a deliberate "story arc" from calm→energetic). The selection is a constrained optimization: maximize archetype diversity × minimize embedding cosine similarity × respect user blocklist. Solved locally with a greedy + 2-opt in <50 ms.

### F20 — Mood-adaptive coloring (per-frame, session-wide)
The engine maintains a **session sentiment EMA** (exponential moving average over the last 20 commands' sentiments). The v2 OKLCH palette's hue drifts slowly toward the EMA: a day of successes → palette warms toward gold; a day of failures → cools toward ember. Subtle, never jarring. The drift rate is capped at ±5° hue per minute so the user never notices a sudden shift.

### F21 — Shell-history daily digest (`forgum day`)
At end of day (or on demand), Forgum summarizes your shell session: *"today you ran 247 commands, 231 succeeded, you shipped 4 times to main, fought the linker twice, and your favorite cow was the dragon."* Renders as a cow thought-bubble summary. **100% local** — reads only from the local anonymized intent log, never raw commands.

### F22 — AI fortune feed (news/weather)
Opt-in. Fetches weather (via `wttr.in`) and optionally a headline (via a user-configured RSS feed), generates a cow comment. `"brrr. -8°C. the dragon considers relocating."` Cloud-recommended for headline summarization; weather is pure heuristics. **Off by default.**

### F23 — Pair-programming cow (tmux)
In a tmux session with `forgum pair on`, the cow watches a designated pane (reads the visible buffer via tmux's capture-pane, never the scrollback). Periodically (every 60 s of inactivity, or on `forgum comment`) the LLM comments on what it sees: *"that function's getting long — maybe extract the validation?"* **Strictly opt-in, strictly local-preferred, redacted.** Never sends code to cloud unless explicitly toggled.

### F24 — Accessibility narrator
For screen-reader users, the cow's thought is **spoken** via TTS and a short alt-text description of the animation is emitted to stderr (`"a dragon, victory palette, ember particles, breathing slowly"`). Auto-detected via `TERM_PROGRAM=screen` or an explicit `forgum a11y on`. The description is LLM-generated once per cow and cached. This is the feature most likely to *matter* to a real user — accessibility is not a cool extra, it's table stakes for a "professional program."

### F25 — Conversational cow REPL (`forgum chat`)
The capstone. A full persona chat: the cow remembers the session, knows your git state, can run commands on your behalf (with confirmation), speaks via TTS, listens via STT. Backed by a local Llama-3.2-3B (Q4, ~2 GB) or cloud. Persona is the cow DNA + tone mode + a system prompt that says "you are {cow_id}, answer in character, be brief, be helpful." The REPL is a separate foreground mode (not the prompt overlay) — it does not interfere with the shell.

---

## 6. Architecture

### 6.1 Local-first principle (the constitution of v3)

1. **Shell history is radioactive.** Every byte of command text, stderr, env vars, and cwd is treated as a secret until proven otherwise. The default pipeline touches none of it with a cloud model.
2. **Cloud is opt-in per feature, per call.** `forgum ai cloud on` is global; individual features can be `forgum ai cloud thoughts on` / `... off`. The default for every cloud-capable feature is `off`.
3. **The redaction pipeline is the only path to a cloud model.** No code may call a cloud client without routing through `Redactor::scrub()`. Enforced by a CI grep: the cloud client crate may only be imported in `crates/ai/src/redactor.rs` and `crates/ai/src/cloud/`.
4. **Local models are downloaded lazily, verified by SHA-256, cached in `~/.forgum/models/`.** First use of a cloud-only feature with no local fallback prints a one-time prompt: *"this feature needs a model; install locally with `forgum ai models install llama-3.2-1b` (1 GB) or enable cloud with `forgum ai cloud on`?"*
5. **AI never blocks the prompt.** The T0 path (selection + tuning) is cache + heuristics only. The T2 path (thought generation) is async and lands on the next cycle. A CI test asserts the precmd hook completes in <5 ms with a warm cache, <15 ms cold.
6. **Graceful degradation is mandatory.** No model → v2 behavior. Cloud down → local fallback. Local too slow → embeddings-only. Embeddings missing → random cow (v1 behavior). Each step down is logged once at `info` level, never spammed.

### 6.2 The `AiEngine` trait

```rust
// crates/ai/src/lib.rs
pub trait AiEngine: Send + Sync {
    /// T4 batch: classify all cows. Called by `forgum ai classify --all`.
    fn classify_cows(&self, cows: &[CowArt]) -> Result<ClassificationBatch>;

    /// T1: embed a command string (post-redaction) into 384-dim.
    fn embed_command(&self, cmd: &str) -> Result<Embedding>;

    /// T1: embed a natural-language mood query (for `forgum find "sleepy dragon"`).
    fn embed_query(&self, query: &str) -> Result<Embedding>;

    /// T2: generate a thought. Async; caller decides timeout.
    fn generate_thought(
        &self,
        ctx: &ThoughtContext,
    ) -> Result<ThoughtHandle>; // handle resolves to text or cached text

    /// T3: generate ASCII art for a new cow.
    fn generate_cow(&self, desc: &str) -> Result<CowArt>;

    /// T4: VLM analysis of a rendered frame.
    fn analyze_frame(&self, frame: &Frame) -> Result<FrameAnalysis>;

    /// T3: TTS. Returns a path to a wav/opus file.
    fn synthesize_moo(&self, cow: &CowId, mood: &Mood) -> Result<PathBuf>;

    /// T3: STT. Listens for `timeout_ms`, returns recognized text.
    fn listen(&self, timeout_ms: u64) -> Result<String>;
}
```

### 6.3 The backend chain

```rust
// crates/ai/src/backends.rs
pub enum Backend {
    /// Pure heuristics + embeddings. No LLM. Always available.
    Heuristic { embeddings: FastEmbed },

    /// Local LLM + local embeddings + local VLM (candle).
    LocalFull {
        embeddings: FastEmbed,
        llm: CandleLlm,
        vlm: CandleVlm,
    },

    /// Local embeddings (privacy), cloud LLM/VLM (quality), opt-in.
    Hybrid {
        embeddings: FastEmbed,
        cloud: CloudClient,  // z-ai / OpenAI / Anthropic / Ollama
        redactor: Redactor,
    },

    /// Cloud everything. Only if user explicitly `forgum ai cloud mode full`.
    CloudFull { cloud: CloudClient, redactor: Redactor },
}
```

Selection at startup: `Heuristic` (always) → upgrade to `LocalFull` if models present → upgrade to `Hybrid` if cloud opted-in → `CloudFull` only on explicit `mode full`. The `AiEngine` impl delegates each method to the best available backend, with per-method fallback.

### 6.4 The DCL singleton (consistent with v2)

```rust
// crates/ai/src/singleton.rs
use std::sync::OnceLock;

static AI_ENGINE: OnceLock<Arc<dyn AiEngine>> = OnceLock::new();

pub fn ai() -> Arc<dyn AiEngine> {
    AI_ENGINE.get_or_init(|| init_engine()).clone()
}

// init_engine() reads config, probes models, constructs the Backend chain.
// Resettable only in tests via a `#[cfg(test)]` swap helper.
```

No `static mut`, no `unsafe`, no `lazy_static!` — matches v2 principle #5.

### 6.5 Caching layers

| Layer | What | Where | TTL | Eviction |
|-------|------|-------|-----|----------|
| L0 | Cow DNA + embeddings (F01 output) | `~/.forgum/cow_dna.json` + in-mem `hora` index | until `.cow` files change | n/a |
| L1 | Command→intent classification | in-mem LRU, 1024 entries | session | LRU |
| L2 | Intent→cow selection (the NN result) | in-mem LRU, 256 entries | session | LRU |
| L3 | Generated thoughts | `~/.forgum/cache/thoughts.db` (sled) | 30 days | LRU, 1 MB cap |
| L4 | TTS audio (synthesized moos) | `~/.forgum/cache/voice/` | forever | LRU, 50 MB cap |
| L5 | Predictive pre-renders | in-memory ring buffer, 3 slots | session | ring |

The L0 cache is the only one that survives across versions (content-addressed). L1–L5 are session-scoped or size-bounded.

### 6.6 The async pipeline (critical path vs background)

```
precmd hook (T0, ≤5 ms, synchronous):
  1. gather context signals           (~0.5 ms)
  2. intent classify (L1 cache or heuristic+embed)  (~1 ms)
  3. cow select (L2 cache or hora NN)  (~1 ms)
  4. sentiment tune (table lookup)     (~0.5 ms)
  5. render (v2 pipeline, cache-warm)  (~2 ms)
  ──► prompt restored, user types

background (T2, fire-and-forget on a scoped thread):
  6. thought generate (L3 cache or LLM)  (~1 s)
  7. predictive pre-render (L5)           (~0.5 s, on control thread)
  8. session sentiment EMA update         (~0.1 ms)
  ──► lands on the NEXT precmd's thought bubble
```

The thought from step 6 is shown on the *next* prompt, not the current one. This is the core latency trick: **the user never waits for the LLM.** If they push 5 times in 10 seconds, thoughts may lag by one render — acceptable, invisible in practice.

---

## 7. Privacy & security model

### 7.1 Threat model

- **Adversary 1: cloud LLM provider.** Could log prompts, infer habits, leak via breach. → Mitigation: redaction, opt-in, local-default, per-feature cloud toggle.
- **Adversary 2: local malware.** Could read `~/.forgum/cache/`. → Mitigation: cache stores **only hashes + generated text**, never raw commands. `thoughts.db` is `0600`. The intent log is `0600` and rot13-obfuscated (not crypto — just to defeat casual `strings`).
- **Adversary 3: a malicious `.cow` file.** Could contain crafted ASCII that confuses the VLM. → Mitigation: VLM output is JSON-validated against a strict schema; classification runs in a `catch_unwind`; unparseable cows are skipped, not crashed.
- **Adversary 4: supply chain (model weights).** → Mitigation: every model download is SHA-256 verified against a pinned manifest; manifests are signed with the release key.

### 7.2 The `Redactor` (the only path to cloud)

```rust
// crates/ai/src/redactor.rs
pub struct Redactor {
    patterns: Vec<Regex>,        // tokens, keys, passwords
    home_prefix: PathBuf,        // $HOME → ~/
    blocklist: Vec<String>,      // user-specified
}

impl Redactor {
    /// Returns a scrubbed copy. Never the original.
    pub fn scrub(&self, input: &str) -> Scrubbed<String> { ... }

    /// Returns true if input matched ANY redaction pattern.
    /// If true, the call MUST NOT go to cloud even if opted in.
    pub fn is_sensitive(&self, input: &str) -> bool { ... }
}
```

Patterns cover: AWS keys (`AKIA…`), GitHub tokens (`ghp_…`, `gho_…`), GitLab (`glpat-…`), Slack (`xox…`), JWTs (`ey…`), private keys (`-----BEGIN …`), env-var assignments containing `key|token|secret|password|passwd|pwd|api_key|auth`, and any path under `$HOME`. `is_sensitive()` short-circuits the cloud path: a `secret`-cluster command is **never** sent to cloud, full stop, even with cloud on.

A CI test corpus of 500 sensitive-looking inputs asserts every one returns `is_sensitive == true` and `scrub()` contains none of the original secret substring.

### 7.3 The intent log (local-only, anonymized)

To support F09 (suggest-next) and F18 (personalization), Forgum keeps a local log of **intent vectors only** — never raw commands:

```jsonc
// ~/.forgum/intent.log (0600, obfuscated)
{ "ts": 1735022400, "intent": "ship", "exit": 0, "cow": "dragon", "starred": false }
{ "ts": 1735022412, "intent": "fetch", "exit": 0, "cow": "turtle", "starred": false }
```

Reconstructing the actual command from this log is impossible. `forgum privacy purge` deletes it. `forgum privacy show` prints exactly what's stored (transparency).

### 7.4 Telemetry policy

- **Default: off.** Always.
- If the user runs `forgum telemetry on`, Forgum sends a daily aggregate: `{ intents: {ship: 4, test: 23}, cows_shown: {dragon: 6, turtle: 1}, errors: 0 }`. **No commands, no paths, no text.** Sent to a self-hosted endpoint, configurable.
- The telemetry code is isolated in `crates/ai/src/telemetry.rs` and CI asserts it is not imported by any other module — preventing accidental data flow.

---

## 8. Data model

### 8.1 `cow_dna.json` (the L0 cache, content-addressed)

```jsonc
{
  "version": 3,
  "classifier_version": "llava-1.5-7b-q4+v2",
  "cows": [
    {
      "cow_id": "dragon",
      "source_hash": "blake3:7f3a…",
      "mood": {
        "energy": 0.85, "menace": 0.80, "whimsy": 0.30,
        "majesty": 0.90, "chaos": 0.60, "warmth": 0.20, "focus": 0.70,
        "archetype": "Monster",
        "domain_tags": ["fire","sky","predator"]
      },
      "description": "a hyperactive menacing serious regal wild cold focused fire-sky-predator monster",
      "embedding": [0.0123, -0.0456, … 384 dims …],
      "confidence": 0.92,
      "classified_at": "2026-01-15T08:00:00Z"
    }
    // … 108 more
  ]
}
```

### 8.2 `thoughts.db` (sled, the L3 cache)

Key: `blake3(intent_vector || cow_id || tone_mode || lang || git_state_hash)`.
Value: `{ text: String, generated_at: i64, backend: "local"|"cloud", starred: bool }`.
No raw command. No raw stderr. Irrecoverable.

### 8.3 `~/.forgum/models/manifest.json` (model registry)

```jsonc
{
  "models": [
    {
      "id": "llama-3.2-1b-instruct-q4_k_m",
      "url": "https://huggingface.co/.../resolve/main/model.gguf",
      "sha256": "abc123…",
      "size_bytes": 758000000,
      "role": "llm-thought",
      "min_ram_mb": 2048
    },
    {
      "id": "bge-small-en-v1.5",
      "url": "https://huggingface.co/.../resolve/main/model.onnx",
      "sha256": "def456…",
      "size_bytes": 33000000,
      "role": "embeddings",
      "min_ram_mb": 256
    }
    // … vlm, stt, tts voices
  ]
}
```

`forgum ai models install <id>` downloads, verifies, registers. `forgum ai models list` shows installed + available. Lazy: the first feature needing a model prompts to install.

---

## 9. Phased integration roadmap (v3, phases A–G)

Each phase is **test-gated**: the phase is "done" when its CI gate is green on all Tier-1 platforms (Linux x86_64, macOS aarch64, Windows x86_64). No phase starts until the prior is green.

### Phase A — Classification infrastructure (F01)
**Goal:** every cow has a 9-axis mood DNA + embedding, persisted to `cow_dna.json`.
- `crates/ai/` skeleton: `AiEngine` trait, `Backend::Heuristic` impl (embeddings only, no LLM).
- `fastembed` integration (BGE-small-en, 384-dim).
- VLM classification pipeline (local LLaVA-Q4 + cloud fallback).
- `forgum ai classify --all` CLI command.
- `cow_dna.json` schema + content-addressed invalidation.
- CI gate: 109 cows classified, JSON schema-valid, ≥80% confidence on a golden set of 20 hand-labeled cows.

### Phase B — Command-aware selection (F02, F03, F05)
**Goal:** the prompt-render path picks a contextually-appropriate cow in ≤5 ms.
- Context-signal gatherer (shell hook side).
- Intent classifier (regex + hora NN over cluster centroids).
- Cow selector (hora NN over cow embeddings).
- Sentiment tuner (palette/speed override table).
- L1 + L2 caches.
- **Critical-path latency test:** precmd ≤5 ms warm, ≤15 ms cold, asserted in CI.
- CI gate: 200-command golden corpus, manual-expected-cow match ≥75%.

### Phase C — Contextual thoughts (F04, F07, F06, F17, F20)
**Goal:** the cow says something contextual, tonally matched, cached.
- `Backend::LocalFull` (candle + Llama-3.2-1B).
- Async thought generator (lands on next cycle).
- L3 thought cache (sled).
- Tone modes + natural-language cow search (`forgum find "sleepy dragon"`).
- Multi-language + session-sentiment EMA palette drift.
- CI gate: thought cache hit-rate ≥60% on a 100-command session replay; 0 critical-path overruns.

### Phase D — Productivity cows (F08, F09, F10, F11, F16)
**Goal:** the cow is useful, not just decorative.
- Error-explanation cow (stderr redaction mandatory).
- Suggest-next-command (local Markov over intent log).
- `forgum commit` (Conventional Commits, fzf picker).
- `forgum review` (cloud-recommended, local fallback).
- VLM frame audit (build-time + runtime sampling).
- CI gate: error-explanation redaction test (500 sensitive stderrs → 0 leaks); commit-message quality eval (10-diff golden set, human-acceptable ≥8/10).

### Phase E — Voice (F13, F14, F24)
**Goal:** the cow speaks and listens.
- TTS integration (Piper local + z-ai cloud).
- Per-archetype voice profiles + mood modulation.
- STT wake-word (`whisper-rs` + VAD).
- Accessibility narrator (alt-text + spoken thought).
- CI gate: TTS latency <800 ms local; STT wake-word false-positive rate <5% on a 10-min silence corpus.

### Phase F — Generation & personalization (F15, F18, F19, F21, F22)
**Goal:** the cow creates and learns.
- `forgum new "<desc>"` (text→ASCII, validated, auto-classified).
- Personalized tone learning (local logistic regression over starred thoughts).
- Smart herd composition (greedy + 2-opt).
- Daily digest (`forgum day`).
- Optional news/weather fortune feed (opt-in, cloud).
- CI gate: generated cows parse + render without error on 50 descriptions; herd composer produces archetype-diverse herds on 20 test cases.

### Phase G — Conversational REPL (F23, F25)
**Goal:** full persona chat.
- `forgum chat` REPL (local Llama-3.2-3B or cloud).
- Tmux pair-programming mode (capture-pane, redacted, opt-in).
- Voice in/out for the REPL.
- Session memory (summarized, local-only).
- CI gate: REPL responds in <2 s local; pair-mode never sends unredacted code to cloud (200-pane corpus → 0 leaks).

**Estimated timeline:** A–B (1 week), C (1 week), D (1 week), E (1.5 weeks), F (1.5 weeks), G (2 weeks). **~8 weeks** for the full v3 vision, shipped incrementally — each phase is independently useful and releasable.

---

## 10. API surface (`forgum ai ...`)

```bash
# classification
forgum ai classify --all              # F01: re-tag every cow
forgum ai classify --cow dragon       # re-tag one
forgum ai show dragon                 # print the 9-axis DNA

# selection
forgum ai why                         # explain the last selection
forgum ai why --verbose               # intent vector + NN distances
forgum find "sleepy dragon"           # F06: semantic cow search

# thoughts
forgum tone doom                      # F07: switch tone
forgum star                           # F18: star the last thought
forgum thought --regenerate           # force a new thought

# generation
forgum new "cyberpunk cow"            # F15: generate a cow
forgum commit                         # F10: draft a commit message
forgum review HEAD~1                  # F11: review a diff

# voice
forgum voice on                       # F13: enable TTS moos
forgum listen                         # F14: STT command

# models & cloud
forgum ai models list                 # installed + available
forgum ai models install llama-3.2-1b
forgum ai cloud on                    # global cloud opt-in
forgum ai cloud thoughts on           # per-feature
forgum ai cloud mode full             # cloud everything (explicit)

# privacy
forgum privacy show                   # what's stored
forgum privacy purge                  # delete intent log + caches
forgum telemetry on                   # opt in to aggregate telemetry

# daily / chat
forgum day                            # F21: daily digest
forgum chat                           # F25: conversational REPL
forgum pair on                        # F23: tmux pair-programming
```

### Config (`~/.forgum/config.toml`)

```toml
[ai]
enabled = true                 # master switch (off → pure v2)
default_backend = "auto"       # auto | heuristic | local | hybrid | cloud
cloud = false                  # global cloud opt-in
lang = "en"                    # F17
tone = "default"               # F07
voice = false                  # F13
max_thought_words = 12

[ai.features]                  # per-feature overrides
error_explain = true
suggest_next = true
predictive_prerender = true
pair_programming = false       # requires explicit on
fortune_feed = false           # news/weather

[ai.privacy]
redact_home_paths = true
blocklist_commands = ["aws*", "gcloud*"]  # never classify, never log
intent_log = true
telemetry = false

[ai.models]
llm = "llama-3.2-1b-instruct-q4_k_m"
embeddings = "bge-small-en-v1.5"
vlm = "llava-1.5-7b-q4"
tts = "piper-en_US-ryan-medium"
stt = "whisper-tiny-en"
```

---

## 11. Testing & evaluation

AI features break the standard unit-test pyramid because their outputs are probabilistic. v3 adds **two new test tiers** on top of v2's six:

### 11.1 The 8-tier test pyramid (v3)

| Tier | What | Count | Gate |
|------|------|-------|------|
| 1. Unit | pure functions, redactor, intent regex | ~400 | 100% pass, every PR |
| 2. Integration | AiEngine backends, cache layers, model loading | ~80 | 100% pass |
| 3. E2E-under-tmux | full precmd → render → thought pipeline | 30 (v2's 20 + 10 AI) | 100% pass |
| 4. Bench | latency budgets (T0 ≤5 ms, T2 ≤3 s) | 15 | no regression >10% |
| 5. Fuzz | redactor, cow parser, VLM JSON schema | 5 targets, 10M runs | no panics, no leaks |
| 6. Golden-visual | rendered frames, blake3-hashed | 109 cows × 4 palettes | byte-exact |
| **7. Eval-golden** (new) | classification accuracy, thought quality, selection match | 5 eval suites | ≥ thresholds below |
| **8. Perceptual** (new) | VLM-judged "does this cow fit the moment?" | weekly, 100 samples | ≥0.7 agreement |

### 11.2 The eval suites (tier 7)

- **Classification eval:** 50 hand-labeled cows (gold mood DNA). Auto-classification must match within ±0.15 per axis on ≥80%, and archetype match ≥90%.
- **Selection eval:** 200-command corpus with human-expected cow (e.g., `git push` success → expect a Hero/Monster with energy >0.5). Match ≥75%.
- **Thought eval:** 30 scenarios × 3 tone modes. Human-rated 1–5 on (in-character, contextual, not-repetitive, brief). Mean ≥3.5.
- **Redaction eval:** 500 sensitive inputs → 0 leaks. **Hard gate.**
- **Latency eval:** 1000 precmd replays, p99 ≤5 ms warm, p99 ≤15 ms cold. **Hard gate.**

### 11.3 The perceptual check (tier 8)

Once a week, CI picks 100 random (command, cow, thought) triples, renders them, and asks a VLM: *"does this cow's mood fit this terminal moment? answer yes/no + 1-line reason."* Agreement with a human-labeled subset must be ≥0.7. Below threshold → page a maintainer (not auto-fail; AI drifts and a human should investigate).

### 11.4 The eval harness

```
forgum eval <suite>           # runs a suite locally, prints report
forgum eval --ci              # runs all hard-gate suites, exits non-zero on fail
forgum eval --update-golden   # regenerates golden sets (maintainer only)
```

All evals are **deterministic-seeded**: the LLM is called with `temperature=0` and a fixed seed; the golden sets are versioned in `eval/golden/`. A regression is a real regression, not noise.

---

## 12. Risks & mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Cloud LLM leaks a secret despite redaction | Low | **Critical** | Redactor is the only path; `is_sensitive()` hard-blocks; 500-input corpus test; local-default. |
| Local model too slow on low-end machines | Medium | High | Auto-detect RAM/CPU; fall back to Heuristic backend; `max_thought_words` auto-lowered; user informed once. |
| VLM misclassifies cows → wrong selections | Medium | Medium | Confidence threshold (≥0.8); low-confidence cows excluded from selection; human override JSON; weekly perceptual check. |
| Model download fails / supply-chain attack | Low | **Critical** | SHA-256 pinned manifest, signed; fallback to Heuristic; `forgum ai models verify` command. |
| Thought cache grows unbounded | Low | Low | sled LRU, 1 MB cap; `forgum privacy purge`; CI asserts cache size after a 10k-command replay. |
| AI makes the cow *annoying* (too talkative) | Medium | Medium | Default `max_thought_words=12`, default thought shown on ~50% of prompts (configurable), `forgum quiet` to suppress for a session. |
| Tone mode generates offensive content | Low | High | Tone modes are constrained personas; LLM is `temperature=0` for default tone; cloud models use provider safety filters; user blocklist on domain_tags. |
| Pair-programming cow sends code to cloud without consent | Low | **Critical** | Feature is `off` by default; requires explicit `forgum pair on` + cloud opt-in; redactor scrubs; CI 200-pane leak test. |
| Battery drain from background LLM | Medium | Medium | Thought generation is debounced (max 1 per 3 s); predictive pre-render capped at 3 slots; on battery (`power_status`), drop to Heuristic backend. |
| Model drift across versions | Medium | Low | `classifier_version` field in `cow_dna.json`; mismatch triggers re-classify prompt; eval golden sets versioned. |

---

## 13. Future horizon (beyond v3)

- **Fine-tuned small models.** A 500M-parameter model fine-tuned on (command, cow, thought) triples could run in <500 ms locally and match cloud quality for the narrow thought-generation task. Trainable from the user's *own starred* thoughts (F18 data) — a genuinely personalized cow.
- **Multi-cow narratives.** A herd where cows *interact* — the dragon breathes fire, the turtle ducks. Requires a small scene-graph + scripted or LLM-driven interactions. Phase H+.
- **Cross-terminal herd sync.** Two developers' cows see each other over a shared rmux session and react (F25 + the rmux/herdr integration from `05-…`). Phase I+.
- **Cow-as-agent.** The cow can *run* commands (with confirmation): "the dragon notices your tests are failing — run `cargo test -- --nocapture`?" A supervised agent loop. Phase J+ — requires careful UX so it never feels like the cow took over.
- **On-device training.** Apple Silicon MPS / Vulkan compute for local fine-tuning. The cow that knows you best is the one you trained.

---

## 14. References & prior art

- **fastembed** — Rust, ONNX, BGE models. https://github.com/Anush008/fastembed-rs
- **candle** — HuggingFace Rust ML framework. https://github.com/huggingface/candle
- **hora** — approximate nearest neighbor, Rust. https://github.com/hora-search/hora
- **whisper-rs** — Whisper STT bindings. https://github.com/tazz4843/whisper-rs
- **piper** — fast local neural TTS. https://github.com/rhasspy/piper
- **Ollama** — local LLM daemon (alternative backend). https://ollama.com
- **z-ai-web-dev-sdk** — cloud LLM/VLM/TTS/STT/image-gen (this environment's in-house SDK; usable from Forgum via HTTP). Documented in the project Skills system.
- **Conventional Commits** — for F10. https://www.conventionalcommits.org
- **cowsay** — the original. https://en.wikipedia.org/wiki/Cowsay
- **lolcat** — the original colorizer. https://github.com/busyloop/lolcat

---

## 15. The one-paragraph summary

Forgum v3 adds an **`AiEngine`** that classifies every cow along a 9-axis mood DNA (offline, once), listens to the command you just ran (locally, redacted), selects the cow whose semantic identity best fits the moment (≤5 ms, cached), generates a contextual in-character thought (async, lands next cycle, cached), tunes the animation palette to the sentiment, and — optionally, explicitly opt-in — speaks, listens, explains errors, drafts commits, reviews code, and chats as a full persona. It is **local-first** (your shell history never leaves the machine by default), **non-blocking** (the prompt never waits for a model), **gracefully degraded** (no model → v2 behavior), and **test-gated** (8 tiers including eval-golden and perceptual checks). It ships in 7 phases over ~8 weeks, each independently useful. The result is a terminal companion that feels less like a screensaver and more like a small, opinionated, weirdly loyal collaborator who happens to be a dragon.
