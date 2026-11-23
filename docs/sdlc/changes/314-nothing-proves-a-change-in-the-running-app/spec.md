# Spec: nothing proves a change in the running application

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `python3 scripts/verify-app.py launch --plan` prints the launch it would
   make and starts nothing: a home directory and a tmux socket directory inside
   one fresh folder that is not under the person's home, `TMUX` and `TMUX_PANE`
   removed from the environment, and a folder short enough that the hook socket
   and the tmux socket are each under 104 bytes, the length of a unix socket
   path on macOS. Fails without it:
   `verify_app_plans_a_launch_apart_from_the_persons_home_and_tmux`.
2. `stop` ends only the process the instance record names, and only while that
   process is still the binary the record names; anything else is refused with
   a non-zero exit and left running. Fails without it:
   `verify_app_stop_refuses_a_process_it_did_not_start`.
3. A feature's steps are lines of `verb words` in the section
   `## Seen in the running app` of its `docs/features/<id>/spec.md`. Every verb
   is one the runner has, read from the runner's own list, and every word a
   `click`, `see` or `gone` step names is text the application's source draws.
   Fails without it:
   `every_step_seen_in_the_running_app_uses_a_verb_the_runner_has_and_words_the_app_draws`.
4. `run --list` names exactly the features whose documents carry steps, at
   least three of them, each an id of `docs/features/index.yaml`. Fails without
   it: `verify_app_lists_the_features_whose_documents_carry_steps`.
5. A step is judged against what the window says, not assumed: `see` fails when
   the words are absent, `gone` fails when they are present, and `click` fails
   when no control or more than one control answers to the words (an exact name
   wins over a longer one that contains it). Fails without it:
   `verify_app_judges_a_step_against_what_the_window_says`.
6. A run that starts from a project gets its store from the application's own
   `save_store`, never from hand-written JSON, and the store it writes loads
   through `load_store_safely` with that project in it. Fails without it:
   `verify_app_seed_writes_a_store_the_app_loads`.
7. The pipeline sends a change that paints to the running application:
   `.claude/skills/feature-pilot/SKILL.md` names the reference at the verify
   step, the reference names the script and its five parts (launch, doctor,
   drive, evidence, cleanup), and
   `.claude/skills/sdlc/references/screen-approval.md` says the screen is looked
   at again after the build. Fails without it:
   `the_pipeline_sends_a_change_that_paints_to_the_running_app`.
8. A whole `run`, with a stand-in for the binary and for the tool that reads
   the window, starts the binary inside the planned home and nowhere else,
   writes the evidence folder (a verdict per step, a tree per step, a picture
   per `shot`, the binary's log), exits 0 when every step holds and 1 at the
   first that does not, and in both cases leaves no process it started and no
   scratch folder behind while the evidence folder remains. Fails without it:
   `verify_app_run_keeps_evidence_and_leaves_nothing_running`.

## Behaviour

`scripts/verify-app.py` is run by a session, not by the application's user; it
prints English. Its subcommands:

- `launch [--seed empty|project] [--plan]` — builds nothing; refuses when
  `target/debug/operon` is missing or an instance of this checkout is already
  alive. Writes `target/verify/instance.json`.
- `doctor` — read-only; exit 0 only when the recorded process is alive, is the
  recorded binary, its window answers in the accessibility tree, the binary is
  not older than the newest file under `src/`, and `agent-desktop` reports the
  accessibility permission granted. Each failed line says what to do.
- `run <feature> [--no-build] [--binary <path>]` — `cargo build --locked`,
  launch with the seed the steps ask for, doctor, the steps in order, stop.
  `VERIFY_APP_DRIVER` names a stand-in for `agent-desktop`; with `--binary` it
  is what the suite uses to run the whole path without a window. Exit 0 pass, 1 a step
  did not hold, 2 the run could not start. Prints the evidence folder.
- `run --list`, `eval-step <verb> <tree.json> <words>` (the judgement alone,
  for the suite), `shot <name>`, `tree`, `stop`.

States that are not the happy one:

- Missing binary, `agent-desktop` not installed, or permission not granted:
  exit 2 with the one command or setting that fixes it; nothing is started.
- A step fails: the run stops at that step, saves the window's picture and
  tree as they were, stops the application, exits 1 naming the step.
- Interrupted (`SIGINT`, `SIGTERM`): the application it started is stopped and
  the scratch folder removed before exit.
- An instance record whose process is gone: `launch` replaces it; `stop`
  removes the record and the scratch folder and exits 0.
- Two checkouts at once: each has its own record and scratch folder; neither
  sees the other's.
- `run` for a feature that is not in the catalogue, or whose document has no
  steps section: exit 2 naming the feature and `run --list`; nothing starts.
- A steps section past 64 lines, a line that is not `verb words`, or a first
  line that is not `start empty` or `start project`: exit 2 naming the line.
- A window tree past 8 MiB: that step fails, saying the tree was too large.
- `run` or `launch` while this checkout's instance is alive: exit 2 naming the
  recorded pid and `stop`; the live instance is not driven.
- `start project`: the seed is one project named `verify-demo`, an empty git
  repository created inside the scratch folder; no session.
- The steps cannot type into a field or press a key: operating without taking
  the keyboard from the person allows presses of controls only. A feature that
  needs typed input starts from a seed instead.

Evidence: `target/verify/<utc>-<feature>/` holds `NN-<name>.png` per `shot`,
`NN-tree.json` per step, `steps.txt` with each step's verdict, and
`operon.log`. `stop` removes the scratch home, never this folder.

The window appears on the person's screen while a run lasts; controls are
pressed through the accessibility tree, so the pointer and the keyboard stay
with the person.

## Design

Placement verdict: `NEW_IN_MODULE` in the development process, which no feature
of `docs/features/index.yaml` claims (`features: none`). Evidence: `git grep`
for `kittest`, `screencapture`, `accesskit` finds only in-process widget tests
in `src/tests.rs` and the `accesskit` feature of `eframe` in `Cargo.toml`; no
script starts the binary.

- `scripts/verify-app.py` — Python 3 standard library only. Isolation is by
  environment, since `app_data_directory` follows `HOME` and tmux follows
  `TMUX_TMPDIR` once `TMUX` is unset; the application is not changed. The
  scratch folder is `mkdtemp` under `/tmp` because the hook socket path must
  stay under 104 bytes, which the session's own scratch directory cannot give.
  The window is read and pressed with the `agent-desktop` CLI (an external
  tool the person has installed; not a dependency of the crate), run with
  `TMPDIR` set to the per-user directory it requires.
- External commands: `agent-desktop` (each call bounded at 30 s), `cargo build
  --locked` and `cargo test --locked verify_app_seed_writes_a_store_the_app_loads` (bounded at
  1800 s), `kill` of the recorded pid, `tmux -S <scratch socket> kill-server`.
- `src/tests.rs` — the eight tests above. The seed test writes to a temporary
  directory, or to the folder `OPERON_VERIFY_SEED` names when the script runs
  it.
- `docs/features/<id>/spec.md` — the new section, for the features proven live.
- `.claude/skills/feature-pilot/references/verify-in-the-app.md` — the five
  parts, for a session reading cold. A reference, not a skill: a skill costs a
  When-line in `AGENTS.md`, and the always-loaded text is at its ceiling.
- `docs/sdlc/README.md` — one row in the built table.

No persisted shape changes; nothing runs in the application.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing is drawn |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no mark |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | the verbs are read by the test from the runner's own list; the socket bound is checked with `hook_socket_path` rather than a retyped path |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | the store shape is untouched; the seed goes through `save_store` |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | no child is added to the application; the script's children are bounded |
| Documentation — user-facing docs change in all three languages together | no | no user-facing document changes |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | everything runs on this machine; no network call |
| Budgets — any new scan or output path states its byte and item ceiling | yes | a steps section is read up to 64 lines; a tree read up to 8 MiB |

## Flagged concerns

- **CI cannot run a journey** — a hosted runner has no accessibility
  permission. Answer: the suite checks everything that does not need a window
  (requirements 1 to 8), and the live runs are a `human` contract item recorded
  from this machine.

## Acceptance

- `cargo test --locked verify_app_`, `cargo test --locked every_step_seen_in_the_running_app` and
  `cargo test --locked the_pipeline_sends_a_change_that_paints` pass.
- `python3 scripts/verify-app.py run <feature>` exits 0 for every feature with
  steps, on this machine, and each evidence folder holds its pictures.
- After a run, `pgrep -f target/debug/operon` prints nothing, the scratch
  folder is gone, and the person's `~/Library/Application Support/com.local.operon`
  has the modification time it had before.
- `bash scripts/check-bands.sh` shows no metric in a higher tier than on main.

## Rejected alternatives

- A new `verify-app` skill — costs a When-line in `AGENTS.md`, which has 18
  bytes left under its cap.
- `egui_kittest` snapshot images — a new dev-dependency and a second renderer;
  it proves a widget tree, not the built binary with a real tmux.
- A data-directory flag in the application — a product change for a process
  need; the environment already gives the isolation.
- Hand-written store JSON for the seed — drifts from the persisted types.
- Typing through the keyboard — takes the keyboard from the person.
- Importing pstack's skills as they are — they assume Cursor's tools and a pull
  request flow this repository does not have, and most of what they hold is
  already here under other names.
