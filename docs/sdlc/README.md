# The Operon development pipeline

Operon's development process is Anthropic's
[AI-native SDLC playbook](https://claude.com/blog/the-ai-native-sdlc-playbook),
mapped onto a one-person, local-first, macOS-only Rust application. The playbook's
shape is kept: six stages, each producing a version-controlled artifact the next
stage reads, with automation running up to a gate and a human standing at it.

Two principles do the work, and everything below is a consequence of them:

- **Policy is applied while the work is written, not discovered in review.** The
  colour system, the icon vocabulary, the identifier SSOT rule, and the
  durability invariants are all read *before* code exists, not after.
- **A rule that must hold is a failing check, not a sentence.** Prose in
  `AGENTS.md` is a promise; `.claude/hooks/gate-commit.sh` is a gate. Every
  mechanism this document introduces is checked by a test in `src/tests.rs` that
  fails when the mechanism goes missing.

## The six stages

| Stage | Artifact | Gate |
|---|---|---|
| 1. Plan | `intent.md` | the artifact is committed |
| 2. Design | `spec.md` | `scripts/check-readiness.sh` returns Go; the screen is approved |
| 3. Build | `plan.md`, code, tests | plan approved before any file is edited |
| 4. Test | the suite, `evals/` | `.claude/hooks/gate-commit.sh` |
| 5. Deploy | `REVIEW.md` passes | `.claude/hooks/guard-bash.sh` checks the release preconditions |
| 6. Maintain | `docs/sdlc/bands.yaml`, `docs/sdlc/lessons.md` | a breach re-enters at stage 1 |

Artifacts for one change live together:

```
docs/sdlc/changes/001-ai-native-sdlc/
  state.yaml   route, stage, what is spent, and one line on what to do next
  intent.md    what problem, whose, and why now
  spec.md      what the change must do, and which policies it touches
  plan.md      which files, in what order, with what proof
  handoff.md   only if an attempt ceiling was reached
```

`state.yaml` is read first, before the prose. The three documents say what the
change is *for*; the data file says where it *is*. Reading three documents to
work out the position is what it exists to stop — and the four statuses that are
not progress (`awaiting-user`, `blocked`, `failed`, `archived`) are the reason
the set is closed: **stopping is a state, not the absence of one.** A session
that stops without recording one is indistinguishable from a session that
finished.

Templates are in `docs/sdlc/templates/`. `.claude/skills/sdlc/SKILL.md` walks a
change through the stages and creates the artifacts from those templates.

### Stage 0 — which stages apply

`docs/sdlc/routes.yaml` is the table, read before stage 1, and it is the only
copy: `feature`, `modify`, `bugfix`, `refactor`, `security`, `docs`, `trivial`, each with its
mandatory stages, the gate that closes it, a ceiling on failed gate attempts, and
whether a person must sign it off. `.claude/skills/sdlc/SKILL.md` reads it and
adds what a table cannot say — how to classify an ambiguous request.

Two files decide the rest of stage 0. `docs/sdlc/risk.yaml` maps the surfaces a
change touches to a tier and an autonomy level — `paused`, `supervised`, `full` —
and is the one place that list is written, so
autonomy is decided by what is being touched rather than by how the session feels
about it. `docs/sdlc/templates/state.yaml`, copied into the change directory, is
what the change carries from there on.

Routes exist because "run the whole pipeline" and "skip the pipeline" were the
only two answers, and a bug fix needs neither: it needs the failing test first
and no `intent.md` at all. The attempt ceiling is the other half — a session that
has failed the same gate three times is missing something, and
`docs/sdlc/templates/handoff.md` is worth more than the fourth try.

### When to skip stages

The pipeline is for changes worth a paper trail. Skip to stage 3 for a typo, a
translation fix, a one-line correction, or a change confined to a single function
with no user-visible effect. Skip nothing when the change adds user-visible
behaviour, touches more than one module, changes a persisted shape, adds a
dependency, or alters a policy.

A change that skips stages says so in its commit message. That is the whole
ceremony.

## Stage 1 — Plan

Describe the problem in your own words, argue with Claude until it is concrete,
and let it write `intent.md` from `docs/sdlc/templates/intent.md`. Correct what it
misunderstood. Commit.

`intent.md` states the problem, not the solution. If it names a file, it is
probably a `plan.md` wearing the wrong hat.

## Stage 2 — Design

`spec.md` is written with the repository's policies open, because in this project
the policies are documents an agent can read:

| Policy | Owner document | What it forbids |
|---|---|---|
| Colour | `DESIGN.md` | a colour literal at a drawing call site |
| Icons | `src/glyphs.rs` (`ICON_VOCABULARY`) | a glyph literal at a call site |
| Identifier SSOT | `src/config.rs` | the same string written in two places |
| Durability | `.claude/skills/durability-invariants/SKILL.md` | an unordered write, a lost `CommittedButNotSynced` |
| Subprocess safety | `src/exec.rs`, `src/agents.rs` | a raw `.output()`, a bypassed validation gate |
| Documentation | `CONTRIBUTING.md` | a `README.md` change in one language only |
| Local-first | `CONTRIBUTING.md` | telemetry, accounts, cloud calls, new `unsafe` |

Every one of them is enforced by a test, so a spec that says "this needs a new
colour role" is also saying "this needs a `DESIGN.md` entry and three palette
rows, or the suite goes red". Concerns the spec cannot resolve are listed under
**Flagged concerns** and answered before stage 3 starts.

### The gates before stage 3

Two, and both are recorded in `state.yaml`.

`bash scripts/check-readiness.sh <change-dir>` asks whether the spec can be
implemented from, exiting `0` Go, `2` Conditional Go, `1` No-Go. It blocks on a
surviving template placeholder, a `TODO`, a `Status:` nobody chose, an empty
**Policy conformance** cell, a flagged concern with no body, a missing section,
and an empty **Acceptance**. It strips backticked spans first, because committed
specs legitimately contain `Vec<Range<usize>>` and `<uuid>` — a checker that
flagged those is a checker people learn to skip.

It judges whether the document is *filled in*, never whether the design is right,
and says so in its own output. A Go is a floor.

The second gate applies when the change paints: `docs/sdlc/risk.yaml` marks the
drawing surface `screen yes`, and the screen is agreed as an ASCII layout — at a
realistic width, with the Japanese strings, and with the empty and error states —
before the code exists. Lesson 005 was a layout bug that shipped under a green
widget test; a widget test does not prove a layout.

Who approves — this layout, an intent, a spec, a plan — is decided by
significance, not by stage: `.claude/skills/sdlc/references/approval.md`. A
decision that changes the product's direction, cannot be undone cheaply, weakens
a protection, or would read to a user as a different product goes to a person;
the rest to an evaluator that did not write it. Every verdict is a line in the
change's `approvals.log`, and `scripts/check-approvals.sh` holds the claims to
it. An escape — a defect a person finds in something approved automatically —
hands that kind of approval back to a person until it earns trust again, so
automatic approval widens only as fast as it is shown to be right.

## Stage 3 — Build

**Plan mode is the default.** Anything past a one-line fix starts in plan mode
with the approved `spec.md` in context. Interrogate the plan — what breaks, what
was assumed, what the alternative was — before accepting it. The approved plan is
committed as `plan.md`; if the implementation departs from it, the plan is
updated, not abandoned.

Institutional knowledge has four homes and they are not interchangeable. The
distinction that matters is *what makes it arrive*:

- `CLAUDE.md` — what a new joiner needs on day one, plus every mistake Claude has
  made twice. Loaded on every turn, so its size is a tax on every task.
- `.claude/rules/` — arrives because a **file was opened**. Each rule declares a
  `paths:` list and loads when Claude reads a matching file, so the colour system
  is present in `src/theme.rs` and absent in `src/store.rs`. This is where the
  four policies that used to sit in `CLAUDE.md` live now; moving them took
  `always_loaded_bytes` from 12 392 back under its 9 716 baseline.
- `.claude/skills/` — arrives because a **task was recognised**.
  `durability-invariants` is read when persistence changes; `sdlc` when a change
  needs a paper trail; `root-cause` before a bug is fixed; `ship` when a build is
  released.
- `.claude/agents/` — a scoped reviewer with its own context.
  `durability-reviewer` and `subprocess-safety-reviewer` are the two surfaces
  where a mistake is unrecoverable rather than merely wrong; `rust-reviewer`
  covers the craft axes no test can see.

A path rule fires when Claude **reads** a matching file, not when it writes one.
Editing an existing file is covered, because a read comes first. Creating a
brand-new file is not — so every policy in `.claude/rules/` keeps its test in
`src/tests.rs`. The rule is the reminder; the test was always the enforcement.
`every_rule_declares_the_paths_it_applies_to` refuses a rule with no `paths:`,
because such a rule loads unconditionally and `always_loaded_bytes` would stop
describing what an agent pays.

Build-phase hooks stay fast and file-scoped: `.claude/hooks/guard-write.sh`
refuses writes into `dist/` and `target/` and refuses to put a credential in the
diff. Nothing at this stage compiles.

## Stage 4 — Test

Claude checks its own work before a human sees it. The commands, and what healthy
output looks like, are in `CLAUDE.md`. For a bug fix the order is fixed: write the
failing test first, confirm it fails for the stated reason, then fix the code
without touching the test.

`.claude/hooks/gate-commit.sh` makes that deterministic at the commit boundary. It
runs fmt and clippy when a Rust or manifest file changed, and it refuses a commit
that lost a test or gained an `#[ignore]` — the two shapes "make the test pass"
takes when it goes wrong in a Rust repository. The suite itself runs in CI on the
pushed branch, and `.claude/hooks/gate-merge.sh` refuses the merge into `main`
until it has passed there (change 128).

`.claude/hooks/gate-stop.sh` holds a change to what it promised: it refuses the
first attempt to finish while an open change's `machine` contract item is still
pending, and — because a gate that can refuse the same tree twice is a loop — it
stamps the position and never refuses it again. Uncommitted Rust is not its
concern: a commit touching Rust runs the gates, and `scripts/package-macos.sh`
runs them over the tree before it builds the application (sdlc 096).

`evals/` is the regression suite for the steering itself. Each eval is a real
prompt plus a deterministic check, seeded from a mistake that actually happened;
`scripts/run-evals.sh` runs each one against a throwaway worktree. When the
suite, `CLAUDE.md`, or `.claude/` changes, the evals are what says whether the
change helped — measured over repeated runs with an interval, because one run is
one sample (sdlc 103). `evals/README.md` says how to read a measured run and how
to improve the steering against the suite without fitting it to the suite.

## Stage 5 — Deploy

`REVIEW.md` is the review policy: which passes run, what counts as Important
versus a Nit, what is excluded, and which reviewers each change needs. It is
what `/code-review` and `.github/workflows/claude-review.yml` both read, so a
policy change lands in one place.

The agent that wrote a change does not approve it, and since change 035 merging
does not wait for a reader either: `scripts/check-review.sh` runs from git's own
`.githooks/reference-transaction`, on the commit object before the branch moves
to it, and refuses a change whose required reviewers have not returned a verdict
carrying **this** diff's digest. A stale approval reads like a fresh one. It is
git's hook and not the Claude Code one because only git knows what a commit
records — `-a`, a pathspec, an amend, a `-C` — and seven rounds of review of a
hook that tried to read that from the command text are the record of why. The
Stop hook, `.claude/hooks/gate-stop.sh`, then reads the branch itself: every
commit since the last one it saw pass goes through the same script, so a commit
that reached the branch around git's hook is named before the session ends.

### Environment tiers

| Tier | Here | Autonomy |
|---|---|---|
| Development | `cargo run`, `cargo test` | free — pre-approved in `.claude/settings.json` |
| Staging | `dist/Operon.app` | written only by `scripts/package-macos.sh`; direct edits blocked |
| Production | `/Applications/Operon.app` | gated — `.claude/hooks/guard-bash.sh` runs `scripts/check-release-preconditions.sh` |

The production gate put three questions to a person, and all three were facts
about the working directory at that instant, so the answer was a re-run of what
the session had just done or a recollection. Change 032 replaced the questions
with the checks: `allow` requires the canonical swap command, the three gates
passing on the tree as it stands, `dist/Operon.app` reproduced `diff -r`-identical
by `scripts/package-macos.sh` from that tree, and no Operon running. Any failure,
and any inability to check, falls back to the same `ask` naming the condition. The
boundary did not move; it got stricter, because a person could answer "yes" to all
three while none was true.

## Stage 6 — Maintain

`docs/sdlc/bands.yaml` holds a control band per harness metric. `scripts/check-bands.sh`
compares the current `scripts/harness-metrics.sh` reading against those bands and
names the tier of any breach:

- **warn** — recorded, no action.
- **diagnose** — read-only investigation: what moved, and when.
- **propose** — write an `intent.md` and re-enter at stage 1.

Because these are deterministic counters rather than a noisy production signal,
the tiers are explicit thresholds, not standard deviations. The playbook's
Western-Electric rules assume a distribution; `unwrapped_spawns` has no
distribution, it has a value, and the value going from 2 to 3 means something
specific happened.

`docs/sdlc/lessons.md` is the incident ledger. Every entry names the mistake, the
root cause, and — the column that matters — which test or eval now catches it.
An entry with an empty guard column is unfinished work.

`.github/workflows/scheduled-scan.yml` runs the band check and a dependency audit
weekly, so a breach that no commit caused is still found.

## What the pipeline is made of, and what it leaves out

Change 007 chose the mechanisms below and left the rest. Recorded here so the
decision is not re-derived the next time someone looks at it.

Built:

| Mechanism | Where |
|---|---|
| work-type routes | `docs/sdlc/routes.yaml` — kind → stages → ceiling, without a sub-skill graph, model routing, or evidence caches |
| the attempt ceiling | the ceiling plus `docs/sdlc/templates/handoff.md`, because a table an agent prints to itself is prose |
| root-cause discipline before a fix | `.claude/skills/root-cause/SKILL.md` — the Iron Law and the phase order; no technique catalog |
| a Rust craft reviewer | `.claude/agents/rust-reviewer.md` — immediate-mode per-frame work, ownership, and panic sites |
| a stop gate on completion claims | `.claude/hooks/gate-stop.sh`, against each change's `state.yaml` contract |
| path-scoped rules | `.claude/rules/` with `paths:` frontmatter, plus a guard that no rule may omit `paths:` |

Not built:

| What | Why not |
|---|---|
| a business and discovery half — market research, pricing, GTM, betting tables | Operon is a terminal manager that already exists; there is no brief to turn into a project |
| a JSON project-config, domain map, and ownership index | path indirection and a state machine for a feature-folder layout. Here `git` and the six-stage artifacts are the state, and one crate with a module map needs no ownership index |
| HTML review decks | thousands of lines of Node per checkpoint. Reviewing before everything is built is the real aim, and the routes table gets at it by making stages mandatory per kind |
| more Node hooks, beyond the six worktree and git guards in `.cli/hooks/` (sdlc 051, 073) | each would cost more than it returns; the guards exist because nothing else stopped a commit on `main` |
| enforced pre-flight and post-flight checklists, outside the `create-pr` skill (which does not run here) | an agent printing a checklist to itself is prose with no guard, and the gates already run at the commit and stop boundaries |

The rule this produced: a mechanism is worth building when it survives being
expressed as a flat data file, a bash script, or a Rust test. If it needs a
runtime to explain itself, it solves a problem this repository does not have.

### The second pass: the orchestrator itself

Change 007 stopped at routing, because an orchestrator needs machinery Operon
does not have. Change 008 built what was left, by building the machinery rather
than the contract:

| Mechanism | Where |
|---|---|
| per-change state | `docs/sdlc/templates/state.yaml`, flat and positional. A format a parser cannot drift from needs no repair tool |
| the position of a change | the six stages as `stage`, plus a closed `status` set carrying the four exceptional ones. Two vocabularies for one position is one too many |
| a one-line resume | the `resume` line, capped at one sentence |
| the readiness gate | `scripts/check-readiness.sh` — one bash script, with no funnel or product-metric phases: a free local tool has none |
| screen approval | `references/screen-approval.md` — the gate logic without the rendering |
| a question limit | the `questions` ceiling, beside the attempt ceiling |
| autonomy by risk | `docs/sdlc/risk.yaml` over the store, `unsafe`, the `PATH` setup, the bundle swap, and the gate configuration — which is `REVIEW.md`'s list, made machine-readable |
| a verifiable completion contract | the `contract` block, refused by `.claude/hooks/gate-stop.sh` |
| sub-skills | none. The six stages are the decomposition |

The rule that pass produced: **detail belongs in the cheapest tier that still
delivers it.** `.claude/skills/sdlc/references/` holds 11 KB that loads only when
the skill reaches for it, which is why the orchestrator cost 1.9 KB of steering
rather than 13 KB. `reference_bytes` exists so that tier cannot be used to hide.

## What this project does not do

Named honestly, so nobody looks for machinery that is not here:

- **Claude Tag / chat-based on-call.** There is no team channel. Incidents arrive
  as the author noticing something.
- **DORA metrics.** The deployment pipeline is one atomic bundle swap.
  `gate_seconds` from `scripts/harness-metrics.sh` and the band-breach count are
  the lagging indicators instead.
- **Hosted scheduled security scanning.** `.github/workflows/scheduled-scan.yml`
  runs `cargo audit` and the band check; a hosted scanner would be additive, not
  a replacement for either.
- **Managed settings and enterprise controls.** One developer, one Mac.
  `.claude/settings.json` is the org policy, and it is version-controlled, which
  is the property that actually mattered.
- **Connectors for non-git users.** Everyone here uses git.

## Measurement

`scripts/harness-metrics.sh` prints one JSON object; diff it across a change.

| Signal | Key |
|---|---|
| How much an agent pays before reading code | `always_loaded_bytes`, `steering_bytes` |
| How much it pays only when a file is opened | `conditional_steering_bytes`, `rules` |
| How much it pays only when a skill reaches for it | `reference_bytes` |
| Whether a change can say where it stands | `states`, `risk_surfaces`, `routes` |
| How much the gates actually cover | `tests_in_ci`, `invariant_tests`, `tests_ignored` |
| How large a thing must be held in mind | `largest_module_lines`, `modules` |
| Where the dangerous surfaces are | `unsafe_blocks`, `subprocess_sites`, `unwrapped_spawns` |
| What the pipeline itself costs | `gate_seconds`, `evals`, `hooks`, `sdlc_changes` |

The numbers are read out of the repository, never estimated. The point is the
trend: a mechanism that stops earning its `steering_bytes` should be deleted, and
the diff is how that argument gets made.
