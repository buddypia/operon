# Spec: approve by evaluation unless the decision is significant, and earn that trust from escapes

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A significance rule exists as `.claude/skills/sdlc/references/approval.md`,
   stated as a principle: a decision goes to a person when a wrong answer would
   change what the product is or where it is going, be unrecoverable or costly to
   undo, remove or weaken a protection, or change what a user sees enough to read
   as a different product. When unsure, escalate. Edits to the rule itself, to
   `scripts/check-approvals.sh`, or to the categories always go to a person.
2. The rule names every `paused` surface in `docs/sdlc/risk.yaml` and splits it by
   what the edit does (table in **Design**); `bundle-swap` always goes to a person.
3. The rule's UI split is structural: new screen or modal, re-arranged layout, or
   a feature removed or moved goes to a person; wording, spacing, and colour or
   state inside existing palette roles goes to the evaluator.
4. Every approval — automatic, by a person, or an escalation — is one line in
   `approvals.log` in the change directory, bound to the 16-hex digest of the
   artifact it judged.
5. `scripts/check-approvals.sh <change-dir>` exits 1 when: a line is malformed; an
   artifact claims approval (intent/spec `Status: approved`, a plan past draft,
   `screen: approved`) with no `approve` line at its current digest; an automatic
   approval in a demoted category has no matching person approval; an escape
   names no automatic approval or a lesson without a `## NNN` heading. It applies
   to changes numbered above 082.
6. An escape demotes its category; the category is restored after `clean_run`
   (5) consecutive clean approvals, where clean means an automatic `approve`
   matched by a person `approve` at the same artifact and digest, and a person
   `revise` or `reject` resets the run. Demotion is computed from the ledgers each
   run, never stored.
7. `.claude/hooks/gate-stop.sh` runs the checker on the changes in play, refuses a
   finish on its failures (once per position, as today), and prints one Japanese
   summary line per session position.
8. `scripts/harness-metrics.sh` emits `auto_approvals`, `approval_escapes`, and
   `demoted_categories`; `docs/sdlc/bands.yaml` bands the last two.
9. `SKILL.md`, `screen-approval.md`, `state-and-resume.md`, `docs/sdlc/README.md`,
   `REVIEW.md`, and `.claude/hooks/guard-stage.sh` stop saying every artifact and
   every screen needs a person, and point at the rule. `CLAUDE.md` and `AGENTS.md`
   do not grow.
10. Nothing changes what `refs/heads/main:scripts/check-review.sh` reads:
    `risk.yaml` rows stay four values with no new block; `routes.yaml`,
    `review.yaml`, and `.claude/agents/` are untouched.

## Behaviour

What the owner sees:

- Most stage boundaries pass without a question. When one does not, the question
  says why it is significant, in the rule's words.
- At the end of a session, when approvals were recorded in the changes in play,
  one line:
  `自動承認 3 件（plan 2・screen 1）· 人へ 1 · 逃れ 0 · 降格中 なし — bash scripts/check-approvals.sh --list で抜き取り確認できます`
  Printed once per position; a second stop at the same position is silent.
- When a claim is unbacked, the Stop gate refuses once and names it:
  `承認台帳: 083-foo/plan.md は approved を名乗っていますが、現在の digest 9c1e… に approve の行がありません。`
- `bash scripts/check-approvals.sh --list [category]` prints the merged ledger for
  sampling; `--summary` prints the one line; `--metrics` prints the three counts.
- When the owner reports a defect in an automatically approved artifact, the agent
  records an escape line and a lesson with a guard. From then on that category's
  approvals go to the owner, with the evaluator's verdict shown beside them, until
  five in a row agree.

Empty states: no ledgers → summary prints nothing; a ledger with only comments is
valid. Legacy changes (≤ 082) are not checked.

## Design

**The rule** — `references/approval.md` (≈2.5 KB against `reference_bytes`' 22000
warn; `screen-approval.md`'s closing section is trimmed to pay for part of it).
Paused surfaces:

| Surface | Person | Evaluator |
|---|---|---|
| `store` | persisted shape, `STORE_SCHEMA_VERSION`, migration, write/sync order, cancellation | an edit touching none of these |
| `unsafe-and-path` | new or changed `unsafe`, `PATH` resolution | other `src/main.rs` edits |
| `bundle-swap` | always | — |
| `gate-configuration` | relaxation: a check removed or narrowed, a bypass or env escape added, a tier lowered, what the main-branch judge reads changed, the judge or trampoline edited | tightening: a refusal, check, guard, or surface path added |
| `dependencies` | a new crate or feature | a version bump inside an existing requirement |

The direction of a paused edit is judged from `plan.md` before editing; the
diff reviewers still judge the diff, and a diff exceeding its plan is Important.

Per artifact:

- **intent** — person when it opens a direction, adds or removes something a user
  sees, or was originated by the agent (a band, a lesson follow-up); evaluator
  when it restates the person's request, quoted inside it.
- **spec** — evaluator against intent and the policy table; person for a flagged
  concern that is a trade-off.
- **plan** — evaluator against the spec; person on a paused surface's person side
  or a departure from the spec. Plan mode is used only when a person decides.
- **screen** — the ASCII layout is committed as `screen.md` so it has a digest.
  Structural → person (approve / revise / reject as today). Small → evaluator
  against the Before/After frames, and after the build against a screenshot.
- **contract `human` items** — evaluator when a file backs the observation (a
  `mutations.py` run); person when it is "looked right on screen" and the screen
  rule sends it there.

**The evaluator** — a fresh subagent that did not write the artifact, briefed
with approval.md's Evaluator section. It reads the artifact, its referent
(intent → spec → plan; `screen.md` → screenshot), approval.md, and the risk rows
the plan names. Verdicts: `approve`, `revise` (a concrete finding against the
referent), `escalate` (significant, taste, or cannot judge). At most two `revise`
rounds, then `escalate` — change 035 showed that open-ended AI review against a
moving target does not converge, so the evaluator judges only against a fixed
referent and never becomes a condition of the commit judge. No new
`.claude/agents/` definition.

**The ledger** — `approvals.log` per change directory: excluded from the review
digest, so appending never voids a review, and no cross-worktree conflicts.

```
# <utc> <by> <category> <artifact> <digest> <verdict> <reason...>
2026-09-26T09:14:02Z auto plan plan.md 9c1e04ab77d2f310 approve follows R1-R4; no paused edit
2026-09-26T09:20:40Z auto screen screen.md 51aa0e9d2c7b8e41 escalate adds a modal
2026-09-26T09:31:05Z person screen screen.md 51aa0e9d2c7b8e41 approve owner: これでいい
2026-10-02T11:00:00Z escape screen 083-foo/screen.md 51aa0e9d2c7b8e41 lesson-051 footer overlaps at 520px
```

`by` ∈ `auto person escape`; `category` ∈ `intent spec plan screen contract` plus
the paused surface names read from `risk.yaml`; digest is
`shasum -a 256 | cut -c1-16` (the convention of `scripts/check-review.sh`), and
for `plan.md` covers the bytes before `## Departures from the plan`. An escape is
written in the change that fixes the defect, naming the approving change's
artifact and digest, with the lesson as its verdict.

**Starting trust** — the artifact categories (`intent spec plan screen contract`)
start trusted. The paused-surface categories start demoted: the evaluator runs in
shadow and the person decides, until five clean approvals earn them automatic
approval. (Recommended default adopted; the owner may reverse it in one line.)

**Checker** — `scripts/check-approvals.sh`, bash 3.2 and awk, modes
`<change-dir>`, `--list [category]`, `--summary`, `--metrics`. `clean_run=5` is
written only there.

**Wiring** — gate-stop runs it for each change in play, folds failures into its
refusal and its position hash, and prints the summary through `systemMessage` on
the exit-0 path, deduped by a stamp line. harness-metrics gains three keys;
bands gain `approval_escapes` and `demoted_categories` at `max 0 1 2 3`.
`risk.yaml`'s `gate-configuration` row gains `scripts/check-approvals.sh` and
`.claude/skills/sdlc/references/approval.md`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No drawing code changes; `src/` outside `src/tests.rs` is untouched. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyphs. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `clean_run` lives only in the checker and the test reads it from there; categories are read from `risk.yaml`, not retyped; the digest convention matches `check-review.sh`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | The store is untouched; the ledger is a repository file, append-only by convention. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Partly | No Rust subprocess. The hook and checker are bash; they follow `every_gate_script_parses_under_the_stock_shell` and `every_gate_script_reads_its_pipes_to_the_end`, and gate-configuration sends the diff to `subprocess-safety-reviewer`. |
| Documentation — user-facing docs change in all three languages together | No | `README.md` files are not touched; only `docs/sdlc/` and `.claude/` prose. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The ledger stays in the repository; no network. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The checker reads only `docs/sdlc/changes/*/approvals.log` and `lessons.md` headings; the summary is one line; `--list` is on demand. `reference_bytes` stays under its warn. |

## Flagged concerns

- **A ledger line proves a line was written, not that a separate evaluator ran.**
  The same exposure `REVIEW.md` states for `review.yaml`. Answered: sampling and
  escapes are the answer, and the demotion makes an escape cost something.
- **An escape is recorded only if the agent consults the ledger when a defect is
  reported.** No hook sees a person's report. Answered: approval.md and the
  root-cause skill say so; the first real escape seeds an eval.
- **Starting trust for paused categories.** Answered by adopting the recommended
  default (demoted, shadow evaluation) and recording it; the owner can reverse it.
- **The ledger path departs from the intent's "beside lessons.md".** Answered: a
  central file would move the review digest and put two change directories in an
  escape commit; per-change files avoid both. The intent listed it as likely only.

## Acceptance

- `cargo test --locked` passes, including
  `the_approval_ledger_rows_are_the_width_the_checker_reads`,
  `the_approval_checker_refuses_a_claim_the_ledger_does_not_back`,
  `the_approval_checker_demotes_on_escape_until_a_clean_run`,
  `the_approval_checker_refuses_an_escape_it_cannot_trace`,
  `the_approval_rule_names_every_paused_surface_and_the_structural_ui_cues`, and
  `the_stop_gate_reports_automatic_approvals_in_one_line`; `6 ignored`.
- Each of those tests watched failing against its mutation.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` silent;
  `bash scripts/check-bands.sh` shows no new breach.
- `bash scripts/check-approvals.sh --summary` on this change's own ledger prints
  the one line.
- `git diff main -- docs/sdlc/routes.yaml .claude/agents` is empty and every
  `risk.yaml` row is still four values.

## Rejected alternatives

- A fifth value in `risk.yaml` or a sixth in `routes.yaml` — the main-branch judge skips the row (lesson 039).
- An `approval:` block in `risk.yaml` — the judge reads its rows as surfaces.
- Evaluator verdicts in `review.yaml` — the judge reads them as reviewer verdicts.
- A new `.claude/agents/` evaluator — grows `steering_bytes` and becomes a valid reviewer name.
- A central `approvals.log` — moves the digest, conflicts across worktrees, splits escapes across two change dirs.
- Stored demotion state — drifts from the ledger.
- A significance list of paths — the owner rejected it; a path cannot tell a label from a modal.
- Making `check-review.sh` require ledger lines — edits the judge and repeats 035's convergence trap.
