# Spec: adopt four mechanisms, and pay for them by making conditional knowledge conditional

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A work-type routing table exists as a flat, parser-proof data file at
   `docs/sdlc/routes.yaml`, giving each kind of request its mandatory stages, the
   gate that closes it, its attempt ceiling, and whether a human must sign it off.
2. `.claude/skills/sdlc/SKILL.md` classifies the request into exactly one route
   before allocating a change directory, and names the route in its report.
3. The `bugfix` route cannot be satisfied without a regression test that was
   observed failing for the stated reason before the fix. The discipline that
   produces it lives in `.claude/skills/root-cause/SKILL.md` and the route names
   that skill.
4. A route's attempt ceiling is enforceable by the session: on reaching it, work
   stops and a `handoff.md` is written into the change directory from
   `docs/sdlc/templates/handoff.md`. The template requires what was tried, what
   was observed, and the narrowest remaining hypothesis.
5. Knowledge that applies only when a particular file is open lives in
   `.claude/rules/`, each file declaring a `paths:` frontmatter list. No file in
   that directory may omit `paths:`, because a rule without it loads on every turn
   and `always_loaded_bytes` stops describing what an agent actually pays.
6. `always_loaded_bytes` falls below its 9 716 baseline, and every policy moved
   out of `CLAUDE.md` is reachable from a one-line pointer that stays in it.
7. `scripts/harness-metrics.sh` emits `conditional_steering_bytes` and `rules`,
   both banded in `docs/sdlc/bands.yaml`. The new mechanisms are counted, not
   assumed.
8. A Rust reviewer with its own context exists at
   `.claude/agents/rust-reviewer.md`, scoped to the axes this codebase can
   actually violate, and `REVIEW.md` names it as a pass.
9. A `Stop` hook refuses a session's first attempt to finish while a Rust or
   manifest change sits in the tree with no gate run behind it, and cannot refuse
   the same tree twice.
10. Every mechanism above has a guard in `src/tests.rs` that fails when the
    mechanism goes missing, and `scripts/harness-metrics.sh` counts those guards
    in `invariant_tests`.
11. What was deliberately omitted is recorded in `docs/sdlc/README.md`, with
    the reason, so the decision is not re-derived.

## Behaviour

No user-visible behaviour changes. `src/` gains tests and loses nothing; the
application is not touched. What changes is what an agent reads and when.

Before: an agent opening any file pays 12 392 bytes of `CLAUDE.md` and
`AGENTS.md`, including the colour system, the icon vocabulary, the transcript
vocabulary, the identifier SSOT rule, the terminal-identity rule, and the pinned
API-docs rule — six policies of which at most one is usually relevant.

After: it pays roughly 9 660 bytes, and reads the colour rule when it opens
`src/theme.rs`, the transcript rule when it opens `src/history.rs`, and the Rust
craft rules when it opens any `.rs` file. `CLAUDE.md` keeps a one-line pointer to
each, so the rule's existence is never conditional — only its body is.

The failure state that matters: a path-scoped rule fires when Claude **reads** a
matching file, not when it writes one. Editing an existing file is covered,
because this harness requires a Read before an Edit. Creating a brand-new file is
not. That gap is why the guards for colour, icons, transcript kinds, and the
identifier SSOT stay in `src/tests.rs` — they were always the enforcement, and
the rules were always only the reminder.

## Design

| New file | What it is |
|---|---|
| `docs/sdlc/routes.yaml` | six routes, one positional tuple each: `<stages> <attempts> <gate> <requires> <human>` |
| `docs/sdlc/templates/handoff.md` | the artifact an exhausted attempt ceiling produces |
| `.claude/rules/rust.md` | Rust craft axes and the pinned-API rule; `paths:` covers `src/**/*.rs` and `Cargo.toml` |
| `.claude/rules/palette-and-glyphs.md` | colour and icons; `paths:` covers the files that paint |
| `.claude/rules/transcripts.md` and `.claude/rules/identifiers.md` | another CLI's record kinds and terminal identity; the identifier SSOT rule |
| `.claude/skills/root-cause/SKILL.md` | the Iron Law and its four phases, compressed to Operon's loop |
| `.claude/agents/rust-reviewer.md` | scoped Rust reviewer: immediate-mode allocation, `unwrap` in reachable paths, `unsafe` argument |
| `.claude/hooks/gate-stop.sh` | `Stop`: unproven Rust change in the tree blocks the first finish |

### Design decisions and rationale

| Component | Design | Rationale |
|---|---|---|
| Pipeline routing | `docs/sdlc/routes.yaml` (~30 lines) | Maps request kind directly to mandatory stages, verification gates, and attempt limits without heavyweight orchestration layers. |
| Attempt ceiling | The `attempts` column plus `docs/sdlc/templates/handoff.md` | Enforces a deterministic stopping point with a committed handoff artifact when an agent thrashes. |
| Systematic debugging | `.claude/skills/root-cause/SKILL.md` | Focuses purely on root-cause identification and regression test reproduction in Operon's local cargo test loop. |
| Scoped code review | `.claude/agents/rust-reviewer.md` | Tailored specifically to Operon's architecture: immediate-mode per-frame allocation, unwrap in reachable paths, unsafe arguments. |
| Completion gate | `.claude/hooks/gate-stop.sh` | Mechanically validates git status and cargo tests via bash hooks rather than advisory checklists. |
| Path-scoped rules | `.claude/rules/` with mandatory `paths:` frontmatter | Ensures rule context is loaded only when relevant source files are accessed, keeping always-loaded bytes low. |

Architectural scope boundaries recorded in `docs/sdlc/README.md`: the harness avoids unnecessary multi-tiered orchestrator abstractions, redundant discovery phases, and excessive hook layers, focusing strictly on local-first verification.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | nothing is painted; no `Palette` role is added or read. The colour *rule text* moves from `CLAUDE.md` to `.claude/rules/palette-and-glyphs.md`, and `design_md_documents_exactly_what_the_app_paints` is untouched and still the enforcement |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | no glyph is drawn; `src/glyphs.rs` is not edited |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | the `Stop` hook's stamp path is written once, in the hook, and the guard reads it out of the hook rather than restating it — the lesson-004 shape applied to a shell script |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | no persisted shape changes. The hook's stamp lives under `target/`, which is cargo's cache and is not persisted state |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | no new `Command::new` in `src/`. The hook spawns `git` and `cargo` from bash, which is the same tier as `.claude/hooks/gate-commit.sh` and is bounded by its `timeout` in `.claude/settings.json` |
| Documentation — user-facing docs change in all three languages together | No | `README.md` and its translations are unchanged; every document touched is harness-internal and English |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | nothing added reaches the network; no dependency is added; `unsafe_blocks` stays at 2 |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `conditional_steering_bytes` and `rules` are the ceilings, banded in `docs/sdlc/bands.yaml`; the `Stop` hook reads at most the `git` name list and writes one stamp line |

## Flagged concerns

- **A path-scoped rule fires on read, not on write.** — Resolved, not dismissed:
  editing an existing file is covered because Edit requires a prior Read in this
  harness, and creating a new file is not. Answer: the rules were never the
  enforcement. Every policy moved out of `CLAUDE.md` keeps its test in
  `src/tests.rs`, and `CLAUDE.md` keeps a pointer line naming the rule file, so a
  session that never triggers the rule still knows it exists. Written into
  `docs/sdlc/README.md` as a known limitation rather than left as a surprise.
- **`steering_bytes` is already in warn (107 077 against a 105 000 threshold).** —
  It crossed the 125 000 diagnose threshold and landed at 127 176. Diagnosed
  rather than waved through, and the diagnosis is recorded in `plan.md`: the
  +20 099 is the three new documents plus lesson 010, and roughly 2 700 of it was
  recovered by deleting duplication the investigation found — the route table had
  been written in three places, which is the SSOT violation this repository has a
  policy against. What remains cannot be cut without dropping a requirement.
  The band's own rule for `diagnose` is read-only investigation, not a halt;
  `propose` at 150 000 is the tier that sends a change back to stage 1. So this
  change stops at diagnose, with the finding below written down rather than left
  for the next reader to rediscover.
- **The instrument is mis-specified, and this change is the wrong one to fix it
  in.** — `steering_bytes` claims to measure "what an agent pays before it reads
  a single line of code", and its glob picks up `docs/sdlc/lessons.md`: an
  append-only incident ledger, 18 KB and growing, that nothing loads before code
  and that is structurally identical to the `docs/sdlc/changes/` artifacts the
  glob already excludes. As written, the metric breaches for honestly recording
  incidents, which inverts what it is for. The same glob misses
  `docs/sdlc/routes.yaml`, which *is* read before stage 1. Both are real defects.
  Neither is fixed here: correcting a formula inside the change that breached it
  is indistinguishable, to a later reader, from adjusting the instrument to clear
  the reading. It is deferred to its own `intent.md` on the `docs` route, where
  the argument can be judged on its merits.
- **Does a `Stop` hook that blocks fight an active `/goal` hook?** — Answer: it
  must be idempotent per tree state, which requirement 9 states and the stamp
  implements. A gate that can refuse the same tree twice is a loop, not a gate.

## Acceptance

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`,
  including `every_rule_declares_the_paths_it_applies_to` and
  `every_route_the_pipeline_offers_names_stages_that_exist`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `always_loaded_bytes` no longer breached.
- `bash scripts/harness-metrics.sh` — `rules` is 3, `conditional_steering_bytes`
  is non-zero, `invariant_tests` has risen, `subprocess_sites` and
  `unsafe_blocks` unchanged.
- Deleting `paths:` from any file in `.claude/rules/` fails the suite. Deleting a
  route's attempt ceiling fails the suite. Both verified by mutation, not asserted.

## Rejected alternatives

- **Introduce an elaborate multi-tiered harness.** Complex multi-script suites with
  dozens of sub-skills, heavy hook layers, and business discovery phases would multiply
  the steering budget for mechanisms that do not apply to Operon's local architecture.
- **Heavyweight orchestration frameworks.** Heavyweight orchestrator architectures requiring
  complex multi-file schemas and dozens of auxiliary skills add cognitive and context
  overhead that Operon's direct, flat pipeline avoids.
- **Keep the policies in `CLAUDE.md` and simply add the new material.** Honest,
  and it pushes `always_loaded_bytes` past 16 000 — the propose threshold — for
  knowledge that is relevant on a minority of turns.
- **Put the rules in `.claude/skills/` instead of `.claude/rules/`.** A
  skill loads when invoked or judged relevant; a path rule loads when the file it
  governs is opened. For "never write a colour literal in drawing code", the file
  being open is the trigger, and relying on a judgement call is how the policy
  gets skipped exactly when it is needed.
- **Add the rules to `steering_bytes`.** Would collapse the distinction the
  mechanism exists to create. `conditional_steering_bytes` is a separate number
  because it is a separate kind of cost.
