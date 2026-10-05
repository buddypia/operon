# Spec: apply the AI-native SDLC playbook to Operon

- **Intent**: `./intent.md`
- **Status**: approved
- **Source**: [The AI-Native SDLC Playbook](https://claude.com/blog/the-ai-native-sdlc-playbook)

## Requirements

1. All six stages of the playbook exist in this repository or are named as
   deliberately absent with a reason.
2. Stages 1 and 2 produce committed `intent.md` and `spec.md` from templates.
3. Plan mode is the documented default, and the accepted plan is committed as
   `plan.md` before any source file is edited.
4. The three gates run automatically at the commit boundary, and a commit that
   loses a test or gains an `#[ignore]` is refused.
5. `/Applications/Operon.app` cannot be replaced without a human decision, and
   the block states what to confirm.
6. A credential-shaped string cannot be written into a tracked file.
7. The review policy exists in exactly one place and is what both `/code-review`
   and CI read.
8. Every harness metric worth watching has a control band, and a breach names its
   response tier.
9. Every new mechanism is checked by a test that fails if the mechanism goes
   missing.
10. The eval suite runs real prompts against the committed steering, seeded only
    from mistakes that actually happened.
11. The cost of the whole addition is reported as a byte and count delta from
    `scripts/harness-metrics.sh`.

## Behaviour

Nothing a user of the app sees changes. What changes is what an agent working in
this repository experiences:

- A write into `dist/` or `target/`, to a credential file, or of a
  credential-shaped string is refused with the reason.
- `git commit` with a Rust change pauses while the gates run, then reports which
  ran. With no Rust change it does not compile anything.
- `git push`, `git tag`, `gh release`, and `scripts/replace-macos-bundle.sh` stop
  and ask, naming what to confirm.
- `rm` of the packaging lock file or of the installed bundle is refused outright.

## Design

Nine new files of machinery, five of documentation, five evals, three workflows,
and edits to the six documents that already steer the repository. No application
module changes.

The mapping decisions that carry the design:

| Playbook concept | Here |
|---|---|
| Ten roles | one person; only the gate positions are real |
| Production | `/Applications/Operon.app` |
| Staging | `dist/Operon.app` |
| Release manager authorization | an `ask` decision from `.claude/hooks/guard-bash.sh` |
| Policy owners read live at spec time | the policy documents this repo already has, named per row in `docs/sdlc/templates/spec.md` |
| "Block the agent editing test files during a fix" | count tests and `#[ignore]`s against `HEAD` — the two shapes the failure takes in a Rust repository |
| Control bands with Western-Electric rules | explicit thresholds, because these are deterministic counters with no distribution |
| `bands.yaml` parsed by a YAML library | a flat positional-tuple subset parsed by awk, so the file cannot drift from its parser |

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — three palette rows plus a `DESIGN.md` entry, WCAG 2.1 AA | no | no drawing code changes |
| Icons — `ICON_*` constant plus `ICON_VOCABULARY` entry | no | no drawing code changes |
| Identifier SSOT — one place, and test the round trip | yes | the three gate strings stay in one list checked across six files; the prefix guard is rewritten to derive its literals from `MANAGED_TMUX_PREFIX` instead of restating them |
| Durability — see the durability skill; schema change means a version bump | no | nothing persisted changes |
| Subprocess safety — children through `src/exec.rs`, launches through the gates | no | no `Command::new` added to the crate; the new subprocesses are shell scripts run by the harness, not by the app |
| Documentation — user-facing docs in all three languages | yes | `CONTRIBUTING.md` gains the pipeline section in English, 한국어, and 日本語; `README.md` is untouched because this is process, not product |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | the app is unchanged; the API-dependent parts are CI jobs that skip without a key |
| Budgets — a new scan or output path states its ceiling | yes | the commit gate tails failure output to 60 lines; the eval runner shares one `CARGO_TARGET_DIR` rather than building per worktree |

## Flagged concerns

- **The commit gate costs about a minute per commit.** Accepted: that is the
  price of the promise being real. Mitigated by skipping cargo entirely when no
  Rust or manifest file changed. Resolved.
- **The gate's 600-second timeout can be short of a cold build, and a hook
  timeout is non-blocking.** Stated in the script header: after `cargo clean`,
  run the gates by hand once. Not fixable from inside a hook. Accepted.
- **`steering_bytes` grows by roughly half.** Accepted, and made visible: the
  band on it is what forces the argument later. Bloat is a real cost, and the
  answer is a measurement, not a promise to be careful.
- **The GitHub workflows cannot be executed.** There is no remote. They are
  written, linted with `actionlint`, and reported as unexercised. Open, and
  closes when a remote exists.

## Acceptance

- `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`
  all pass, with five new invariant tests among them.
- `bash scripts/check-bands.sh` reports no breach.
- `actionlint .github/workflows/*.yml` is clean.
- Every hook is exercised by hand — blocked write, gated commit, fast-path
  commit, asked push, denied `rm` — because a hook that was only configured is a
  hook that was not tested.
- `bash scripts/run-evals.sh --list` lists five, and one runs end to end.
- The metric delta is reported.

## Rejected alternatives

- **A `Stop` hook running the full suite after every turn.** Correct in spirit,
  wrong in placement: the playbook puts heavy checks at the commit boundary
  precisely because a per-turn suite makes the loop unusable.
- **A hook that blocks all edits to `src/tests.rs`.** The file is 8,957 lines and
  edited constantly; the gate would fire on legitimate work far more often than
  on the failure it targets. Counting is the version that only fires on the
  failure.
- **Deriving the pipeline docs into `README.md`.** `README.md` is for people
  choosing whether to use the app.
- **A YAML library for `bands.yaml`.** A dependency for one flat file, in a
  project whose stated policy is to scrutinize dependencies.
