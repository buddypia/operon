# Spec: a change lands on evidence the machine re-runs, not on a reviewer's opinion

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **The record is evidence.** A change carries `verification.yaml`: the diff
   digest (the same recipe `scripts/check-review.sh` uses today) and one
   obligation per claim, each `ID METHOD :: COMMAND`. Once
   `docs/sdlc/verification-methods.yaml` exists in the judged commit's first
   parent, `scripts/check-review.sh` requires this record wherever it used to
   require reviewer verdicts, and reads no verdict, Important count, round, or
   carried finding. Test: `review_gate_requires_a_verification_record_once_the_method_table_exists`.
2. **Two kinds, five methods, closed set.** `docs/sdlc/verification-methods.yaml`
   names each method with the only command prefix it may run:
   `empirical-test` (`cargo test --locked `), `empirical-mutation`
   (`python3 docs/sdlc/changes/`, the change's own `mutations.py`),
   `empirical-measure` (`bash scripts/measure-`), `formal-model`
   (`cargo test --locked model_`), `formal-proof` (`cargo kani --harness `).
   A command outside its method's prefix, or holding a shell metacharacter, is
   refused. Test: `an_obligation_command_outside_its_method_prefix_is_refused`.
3. **Every claim is covered.** Each numbered requirement `n` in the change's
   `spec.md` has an obligation `Rn`; a route other than `docs` and `trivial`
   has at least one `empirical-mutation`. Test:
   `every_spec_requirement_has_an_obligation`.
4. **Risk asks for the kind, not for a person.** The method table maps every
   `high` surface of `docs/sdlc/risk.yaml` to the methods it requires, unioned
   with the route's: `store` requires `formal-model` and `empirical-mutation`;
   `bundle-swap`, `gate-configuration`, `unsafe-and-path` and `subprocess`
   (also pulled in by an added `Command::new`, `.spawn(`, `.output(` or
   `unsafe`, as today) require `empirical-mutation`; `dependencies` requires
   `empirical-test`. Test: `every_high_surface_names_the_methods_it_requires`.
5. **A formal model is exhaustive, mechanically.** `formal-model` tests are
   named `model_*` and call one helper, `exhaustive_check`, which walks every
   reachable state of a finite model under every event by breadth-first search,
   asserts the invariant in each, and on failure panics with the shortest event
   trace that reaches the violation. Tests:
   `exhaustive_check_reports_the_shortest_counterexample` and
   `every_model_test_goes_through_exhaustive_check`.
6. **CI re-runs the evidence.** A `verify` job in `.github/workflows/ci.yml`
   runs `scripts/run-obligations.sh` on every change directory the branch
   touches against `origin/main`. It re-checks requirements 2–3, runs every
   obligation through `scripts/run-bounded.sh`, and fails on the first that
   fails. `.claude/hooks/gate-merge.sh` already waits for this workflow, so a
   recorded obligation that does not hold cannot merge. Test:
   `ci_re_runs_every_obligation_the_branch_records`.
7. **A finding is a reproduction or nothing.** A verifier's finding counts only
   as a test that fails at the change's head. The author fixes the defect and
   records the test as an obligation `Fn` with method `empirical-test`, so the
   reproduction stays in the suite as a regression test. A worry without one is
   not recorded. `REVIEW.md` says this, and the record has no field for an
   unconfirmed finding. Test: `verification_record_has_no_field_for_an_unreproduced_finding`.
8. **History keeps its rule.** A commit whose first parent has no method table
   is judged by verdicts in `review.yaml`, exactly as today. This change itself
   is judged that way by `main`'s gate. Test:
   `review_gate_judges_history_before_the_method_table_by_verdicts`.
9. **The verifiers try to falsify; they do not judge.**
   `.claude/agents/empirical-verifier.md` writes failing reproductions and
   mutation sweeps, and `.claude/agents/formal-verifier.md` writes `model_*`
   tests and proof harnesses. Each works in an isolated worktree and returns a
   patch. Neither returns an approve or reject. Test:
   `verifier_agents_return_evidence_not_verdicts`.

## Behaviour

No screen changes. What a person and a session read comes from the gate and
the CI job, in Japanese like the gate's other refusals:

- Missing record: `❌ BLOCK — {change} · 検証記録がありません` followed by
  `{path} に、変更の主張ごとに検証（経験的・形式的）を 1 行ずつ書いてください。`
- Old digest: `❌ BLOCK — {change} · 検証記録が古い diff のものです`.
- Disallowed command: `❌ BLOCK — {change} · {id} のコマンドは {method} で許されていません`.
- Uncovered requirement: `❌ BLOCK — {change} · 要件 {n} を検証する行がありません`.
- Missing method for a surface: `❌ BLOCK — {change} · {surface} には {method} の検証が必要です`.
- Pass: `✅ SHIP — {change}`, then `検証 N 件（経験的 E · 形式的 F）· diff {digest}`;
  no Important, Nit, or round line.
- CI failure: the job's log names the obligation id, its command, its exit
  status and the last 40 lines of its output. A command that runs past its time
  limit is reported as `時間切れ` (exit 124).

## Design

- **`docs/sdlc/verification-methods.yaml`** (new, flat, positional like
  `routes.yaml`): `methods:` rows `name: "kind prefix"`, `surfaces:` rows
  `surface: "method,method"`, `routes:` rows `route: "method,method"`. It is a
  file `main`'s current judge does not read, so adding it widens nothing
  (lesson 039). Its presence in the first parent is the switch between the two
  rules.
- **`docs/sdlc/templates/verification.yaml`** (new) — the record's shape.
- **`scripts/check-review.sh`** — a new branch after the change directory is
  found: if the method table lands in the first parent, judge
  `verification.yaml` (requirements 1–4) and exit; otherwise run the verdict
  code unchanged. The digest, the change-directory selection, the scrub, and
  the paper-trail rules are shared and unchanged.
- **`scripts/run-obligations.sh`** (new) — the CI runner. It reads the records,
  validates them with the same rules (sourcing one shared function file,
  scripts/lib/verification.sh, so that the gate and the runner cannot
  disagree), and runs each obligation. Ceilings: 900 s per obligation and
  3600 s per job; output is capped at 64 KiB per obligation in the log.
- **`.github/workflows/ci.yml`** — a `verify` job, macOS, same toolchain and
  cache as `test`.
- **`src/tests.rs`** — `exhaustive_check` (generic over a `Clone + Eq + Hash`
  state, a list of events, a step function, an invariant; ceiling 1,000,000
  states, past which it panics rather than passing), the tests above, and the
  gate tests in temporary repositories like the existing gate tests.
- **Documents** — `REVIEW.md` rewritten around the two kinds of evidence;
  `docs/sdlc/README.md` stage 5, `.claude/skills/sdlc/SKILL.md` §8 and §10,
  `.claude/skills/feature-pilot/SKILL.md` §4, `docs/sdlc/routes.yaml` comments
  (the `review` column values stay the words `main`'s judge reads), and a lesson.
- **Agents** — the two verifier definitions are added. `rust-reviewer` and
  `subprocess-safety-reviewer` stay for now: `main`'s judge requires them for
  this change, and history refers to them.
- No external command is added to the app; nothing in `src/` outside
  `src/tests.rs` changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing is drawn. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No icon. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes, in kind | Method names and prefixes live only in `docs/sdlc/verification-methods.yaml`; the gate, the runner and the tests read them there. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | The store is not touched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes, for CI | The app spawns nothing new. The runner executes only commands matching a method prefix with no shell metacharacter, each bounded by `scripts/run-bounded.sh`. |
| Documentation — user-facing docs change in all three languages together | Checked | `README.md` does not describe the review gate (grep, 2026-11-04), so the three READMEs do not move. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | The evidence runs locally and in the existing CI. Nothing is sent to a review service. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | 900 s per obligation, 3600 s per job, 64 KiB of output per obligation, 1,000,000 states per model. |

## Flagged concerns

- **CI time.** A mutation sweep rebuilds the crate for each mutation, so
  re-running it in CI can take tens of minutes. Answer: run it anyway under the
  3600 s job ceiling, and measure the first runs. If the job ceiling is hit, a
  follow-up runs only the obligations whose files the branch's last commit
  touched. The ceiling makes a slow sweep a visible red, not a silent pass.
- **Branches already in flight.** A branch opened before this lands reaches
  `main` through a merge whose first parent has the table, so it needs a
  `verification.yaml`. Answer: accepted. The refusal says what to write, and
  no branch other than this one is open (checked 2026-11-04).
- **`store` requires a model that does not exist yet.** Answer: intentional.
  The first store change after this one also writes the store's first model.
  The follow-up is recorded in the backlog.
- **Kani is not installed and may not build this crate** (native
  `llama-cpp-sys-2`, `eframe`). Answer: `formal-proof` is in the method table
  but no surface requires it. A measured probe is a follow-up, and the method
  becomes required only once that probe shows it builds.

## Acceptance

- `cargo test --locked` passes, including the ten tests named under
  Requirements.
- `bash scripts/check-review.sh` on this branch passes by verdicts (the old
  rule), because `main` has no method table yet.
- On a scratch branch made after this lands, a code commit with no
  `verification.yaml` is refused with `検証記録がありません`, and one with a
  covering record passes.
- The CI `verify` job on this branch runs this change's own obligations and
  is green. A deliberately failing obligation on a scratch push turns it red.
- `docs/sdlc/changes/242-review-verdicts-rest-on-opinion/mutations.py`:
  each guard above goes red when the rule it guards is removed.

## Rejected alternatives

- Keep the reviewer verdicts and add evidence beside them. The rounds and the
  parked findings come from the verdicts, so this keeps the cause.
- Drop the agents entirely. A falsification attempt by a fresh context still
  finds real defects, and the change is in what it may return: a reproduction,
  not an opinion.
- Let the local hook run the obligations. The hook runs inside a ref update
  with a time budget, and `cargo test` and mutation sweeps do not fit in it.
  CI already gates the merge.
- Accept any command in the record. That puts arbitrary execution in CI and
  lets a record pass with `true`.
- TLA+ or Alloy for formal models. They need a JVM toolchain and a second
  language, and a model written apart from the code drifts from it. An
  exhaustive check in Rust runs against the real types.
- Kani now. It is unmeasured on this crate. Making it required before it is
  known to build would block every change.
