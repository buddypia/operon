# Plan: a document carries the root it is relative to

- **Spec**: `./spec.md`

## Files that change

- `src/app.rs` — `OpenDocument`, `active_document`, `document_index`,
  `open_document`, the open/save/watch requests, `open_terminal_target`.
- `src/app/screens.rs` — the tab row's badge, the review pane's full path, and
  every caller of `open_document`.
- `src/tests.rs`, three READMEs.

## Order of work

1. **`OpenDocument` gains `root`, and `absolute()` becomes the only join.**
   Nothing else changes yet, so the compiler names every site that has to be
   revisited. That list is the work.
2. **The key widens to `(project, root, path)`** — `active_document`,
   `document_index`, and the two `BackgroundKey` variants that carry a path.
3. **Tests, watched failing by mutation.** The save test comes first: it is the
   one whose failure destroys a file rather than annoying somebody.
4. **`open_terminal_target` chooses the root**, and keeps its refusal for a path
   under neither.
5. **The tab badge**, drawn only when the root is not the project's.
6. **The READMEs**, whose 026 bullet currently claims something untrue for a
   worktree session.

## Risks

- **A save into the wrong copy.** The failure this change could introduce, and
  it is worse than the defect being fixed: today the application refuses and
  writes nothing. Structural rather than careful — `absolute()` is the only
  place a root meets a path, and
  `no_source_file_joins_a_project_path_to_a_document_path` scans the source for
  any other site doing it by hand. Watched failing with the join written back
  at a call site.
- **Two files becoming one tab.** The project's `src/app.rs` and a worktree's
  have equal relative paths. If the key does not widen, opening the second
  shows the first — and saving writes the first's buffer over whichever the
  path resolves to. `the_same_relative_path_under_two_roots_is_two_documents`
  holds it.
- **A background result landing on the wrong document.** `BackgroundKey::FileOpen`
  and `FileSave` carry `(Uuid, PathBuf)`. With two roots in play, a read of the
  worktree's file could be applied to the project's document. The key carries
  the root for the same reason the document does.
- **Work in a draw path.** The badge is a `&str` already on the document; the
  tab row does no new allocation beyond the label it already builds.

## Assumptions, and what would falsify them

- **`OpenDocument` is not persisted.** It is declared in `src/app.rs`, not
  `src/models.rs`, and no `Serialize` derive is on it — so no schema version
  moves. Falsified by the store test, which would fail if the shape were
  persisted.
- **A session's worktree path is the right root.** It is what
  `resolve_terminal_path` already requires the file to be under, so a file that
  underlines is by construction under it.

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, N five higher.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — nothing newly breached.
- Each new guard watched failing under a mutation aimed at what it guards, with
  the mutation confirmed to have landed where it was aimed.
- In the running app: click a path in a worktree session, edit, save, and
  confirm the project's copy is untouched.

## Departures from the plan

- **A second identity for one file, found while the reviewer was reading.** The
  plan named the save-into-the-wrong-copy risk and missed the way it actually
  arrives: `document_root_for` canonicalises its root and `open_document` took
  `project.path` as stored, so on macOS the same file could produce
  `/var/...` and `/private/var/...` — two `DocumentId`s, two tabs, two save
  keys. `open_document_at` now canonicalises once. A test printed both spellings
  side by side, so this was reached rather than reasoned about.
- **A flake in the shared suite, fixed on the way past.**
  `the_readiness_gate_refuses_a_spec_that_is_not_filled_in` built its fixture inside the real
  change directory that two other tests walk. `cargo test` runs in parallel and a second session runs the
  same suite against the same tree, so either walker could catch the fixture
  mid-life; it failed about one run in three. The checker takes a path, so the
  fixture had no reason to live in the tree it checks. Not this change's work,
  and taken anyway because it was making every gate reading unreliable for two
  sessions at once.
