# Spec: Handoff destinations take a model and effort, and an unchanged conversation's bundle is reused until it expires

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Per-destination handoff options, remembered.** `RecentAgentSettings`
   gains `handoff: HashMap<String, HandoffLaunchSettings>` (`model`, `effort`,
   both `#[serde(default)]`). Loading keeps only keys in
   `FULL_HISTORY_RESTORE_TARGETS`, clears a model that `is_safe_agent_option`
   refuses or that is longer than the existing field ceiling, and clears an
   effort not in `agent_effort_options(agent)`. A file written before the field
   existed loads it empty. Test:
   `handoff_launch_settings_round_trip_and_drop_what_the_gate_refuses`.
2. **The destination starts on the chosen model and effort.**
   `import_full_cli_history_with_note` takes a `&HandoffLaunchSettings` and
   builds the destination command through `build_agent_launch_command`
   (custom command, mode, flags, and extra arguments all empty), then
   `command_with_managed_native_session_id`. Codex with `gpt-5-mini`/`low`
   yields `codex --model gpt-5-mini -c model_reasoning_effort=low`; Claude with
   `sonnet`/`high` yields `claude --model sonnet --effort high` followed by
   `--session-id` and the import ID; empty options yield today's bare `codex`,
   `claude` with its session ID, and `agy`. A refused option returns an error
   before any archive directory is created. Test:
   `a_handoff_starts_its_destination_on_the_chosen_model_and_effort`.
3. **Both entry points use the same options.** `handoff_cli_session` (which
   `handoff_managed_session` also ends in) reads
   `recent_agent_settings.handoff[target]`, refuses synchronously with the
   builder's own message in `notice` when the options are refused (no background
   task is spawned), and otherwise passes them to the import, so the persisted
   destination `Session.agent_command` carries them. Test:
   `a_handoff_from_the_app_records_the_destination_model_in_the_session`.
4. **An unchanged conversation reuses its archive and its note.** Before
   snapshotting, the import looks under the archive root for a compact-handoff
   archive whose manifest names the same source provider, source conversation
   ID, and source transcript path, whose recorded source fingerprint (byte
   length and modification time in nanoseconds, taken before the copy) equals
   the transcript's current one, and which has not expired. When found, no new
   snapshot is taken: the bundle is rendered from that archive's snapshot for
   the new destination, the note is the new one when non-empty and the
   archive's recorded note otherwise, `handoff-bundle.md` and `manifest.json`
   are rewritten in place with `last_used_at` set to now (the original
   `created_at` kept), and `ImportedNativeSession.reused_bundle_created_at`
   carries that `created_at`. Test:
   `an_unchanged_conversation_handed_off_again_reuses_its_snapshot_and_note`.
   A candidate whose snapshot file is missing, is not a regular file, or reads
   to no conversational turns is not reused: the import takes a fresh snapshot
   instead of failing. Test: `a_reuse_whose_snapshot_is_gone_takes_a_fresh_one`.
5. **A conversation that moved on is snapshotted again.** When the transcript's
   fingerprint differs from every candidate's, a fresh archive is written and
   `reused_bundle_created_at` is `None`; the new bundle contains the turns
   added since. Test: `a_conversation_that_moved_on_is_snapshotted_again`.
6. **Expiry deletes only expired compact-handoff archives.**
   `sweep_expired_handoff_archives(archive_root, now)` removes a directory
   directly under the archive root only when its name parses as a UUID, it is a
   real directory (not a symlink), its `manifest.json` (read within
   `HANDOFF_MANIFEST_MAX_BYTES`) names the compact-handoff restore kind, and
   `last_used_at` (or `created_at` for a manifest without it) plus
   `HANDOFF_ARCHIVE_TTL_SECONDS` (seven days) is not after `now`. A fresh
   archive, an archive whose manifest names another restore kind (the earlier
   full-history restore), a directory with no or an oversized manifest, a
   non-UUID name, and a symlink — and the directory it points at — are left as
   they are. It returns how many it removed. Test:
   `handoff_archives_unused_for_the_ttl_are_deleted_and_nothing_else_is`.
7. **An expired archive is not reused.** The import runs the sweep before
   looking for a reusable archive, so an archive past its expiry is removed and
   a fresh snapshot taken. Sweep, lookup, and the in-place rewrite run under one
   process-wide lock, so a sweep cannot remove an archive between its lookup and
   its reuse. Test: `an_expired_archive_is_swept_rather_than_reused`.
8. **The app sweeps once at startup.** The first frame spawns
   `BackgroundKey::HandoffArchiveSweep` on the archive root beside the store;
   later frames do not spawn it again. Test:
   `the_app_sweeps_expired_handoff_archives_once_at_startup`.
9. **The notice says whether the bundle was reused.**
   `finish_full_history_import` shows the reuse notice when
   `reused_bundle_created_at` is `Some`, and today's creation notice otherwise.
   Test: `finish_full_history_import_says_when_a_bundle_was_reused`.
10. **The session menu names the options it will use.**
    `handoff_menu_label(agent, settings)` reads `{p0} へ Handoff` when both
    options are empty and `{p0} へ Handoff（{launch}）` otherwise, where
    `launch` is the non-empty values joined by ` · `. Test:
    `the_session_menu_names_the_handoff_model_and_effort`.

## Behaviour

- **History tab card (`ui_cli_sessions`)**: under the existing
  `復元先 [picker] [⟲ Handoff バンドルで引き継ぐ]` row, a second row:
  `モデル（任意） [既定のモデルを使う]  推論の深さ [既定（CLI に任せる） ▾]`.
  The row edits the options of the destination currently chosen in that card's
  picker; switching the picker shows that destination's own options. The text
  field's hover reads `{p0} への Handoff で使うモデル。空欄なら {p0} の既定のモデルです。`.
  The model is saved when the field loses focus, the effort when it changes,
  and both again when the handoff button is pressed.
- **Session list menu (`render_restore_options`)**: each `{p0} へ Handoff` entry
  reads `{p0} へ Handoff（gpt-5-mini · low）` when options are set for that
  destination; hover text is unchanged.
- **Refused options**: a model containing a character the launch gate refuses
  (for example `;`) shows `エージェントの起動オプションが不正です。` in the notice bar;
  nothing is copied, no session is created, no modal opens.
- **Reuse**: the completion notice reads
  `{source} の会話 {source_conversation} の Handoff バンドル（作成: {created}）を再利用し、{destination} の新規セッションを起動しています…{breakdown}`,
  where `created` is `relative_time` (`たった今`, `3分前`, `2時間前`). The destination
  receives the earlier note when the note field was left empty.
- **Fresh creation**: today's notice
  `{source} の会話 {source_conversation} から Handoff バンドル（直近 {records} ターン）を作成し、…`
  is unchanged.
- **Expiry**: nothing is shown. An archive whose last use is seven days old
  disappears at the next startup or handoff. A destination session that later
  tries to read the snapshot path named in its bundle finds it gone; the bundle
  itself is already in that session's conversation.
- **Empty / error states**: unchanged from change 179 (unreadable transcript,
  no conversational turns, missing tools, worktree mutation in progress). A
  failure inside the sweep (an entry that cannot be read or removed) is skipped
  silently and retried by the next sweep; it never fails a handoff.
- **Too large**: one pass inspects at most `HANDOFF_ARCHIVE_SCAN_MAX_ENTRIES`
  (4 096) entries under the archive root and reads each manifest within
  `HANDOFF_MANIFEST_MAX_BYTES` (256 KiB). Entries past the ceiling are neither
  reused nor swept in that pass; a missed reuse falls back to a fresh snapshot,
  which is the behaviour today.

## Design

Placement verdict: **`EXTEND`** — handoff exists (`src/history.rs`
`import_full_cli_history_with_note`, change 179); remembered launch options
exist (`src/agents/settings.rs` `RecentAgentSettings`). Route `modify`.

- **`src/config.rs`**: `HANDOFF_ARCHIVE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60`,
  `HANDOFF_ARCHIVE_SCAN_MAX_ENTRIES: usize = 4_096`,
  `HANDOFF_MANIFEST_MAX_BYTES: u64 = 256 * 1024`,
  `HANDOFF_MANIFEST_FILE_NAME: &str = "manifest.json"`,
  `SHARED_SESSION_SNAPSHOT_FILE_NAME: &str = "source.jsonl"`,
  `HANDOFF_ARCHIVE_RESTORE_KIND: &str = "compact handoff bundle"`. The three
  strings replace the literals in `src/history.rs` and `src/cli.rs`
  (`snapshot_native_transcript`).
- **`src/agents/settings.rs`**: `HandoffLaunchSettings { model, effort }`,
  `RecentAgentSettings::handoff`, `sanitize_handoff_launch_settings`, applied in
  `load_recent_agent_settings`. `handoff_menu_label` and
  `handoff_launch_summary` live here beside the type they read.
- **`src/cli.rs`**: `ImportedNativeSession` gains
  `reused_bundle_created_at: Option<u64>`.
- **`src/history.rs`**:
  - `import_full_cli_history_with_note(project, source, target, note, launch,
    archive_root, import_id)`; the test-only five-argument wrapper passes
    `HandoffLaunchSettings::default()`.
  - A `static HANDOFF_ARCHIVES: Mutex<()>`; a poisoned lock is taken with
    `PoisonError::into_inner` (the guarded data is `()`; nothing to corrupt).
  - `sweep_expired_handoff_archives(archive_root, now) -> usize` (public to the
    crate; the app's startup task calls it), `reusable_handoff_archive(…)`,
    `read_handoff_manifest(directory) -> Option<serde_json::Value>`,
    `transcript_fingerprint(path) -> Option<(u64, u64)>`.
  - Manifest `schema_version` 6 adds `created_at` (kept across reuse),
    `last_used_at`, `source_fingerprint { bytes, modified_ns }`, and `note`.
    Version-5 manifests (none written by a released build carry a fingerprint)
    are never reused and expire by `created_at`.
- **`src/app.rs`**: `BackgroundKey::HandoffArchiveSweep`,
  `BackgroundResult::HandoffArchivesSwept`, `handoff_archive_sweep_started:
  bool`, `ensure_handoff_archive_sweep_started()` called from `update` beside
  `ensure_hooks_started` (a bool test per frame; the directory walk is on the
  background thread). `handoff_cli_session` reads and validates the options
  before spawning; `finish_full_history_import` picks the notice.
- **`src/app/screens.rs`**: the second card row and the menu label. The text
  field binds the in-memory `HandoffLaunchSettings` for the card's target,
  inserted once when absent (no allocation per frame after that).
- **Persisted shape**: `store-v2.json` is untouched. `recent-agent-settings.json`
  gains one defaulted field; an older build ignores it on load and drops it on
  its next save, which loses only the handoff options.
- **External commands**: none added. The destination launch already goes
  through tmux via `start_session`; its command line is now built by
  `build_agent_launch_command`, which applies `is_safe_agent_option` and the
  effort table.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The new row uses default widget text and the existing `text_muted` role used by the `復元先` label; no role is added. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new glyph; the reuse notice and menu label carry text only. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `manifest.json`, `source.jsonl`, and the restore kind become constants, written by the import and read by the sweep and the lookup; tests 4, 6, and 7 age and reuse archives the import itself produced rather than hand-written literals. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No store shape change, so no `STORE_SCHEMA_VERSION` bump. Archive directories are still created through `create_private_directory` (`0700`); rewritten files go through `write_file_atomically` inside them. Deletion is confined to UUID-named real directories directly under the archive root with a compact-handoff manifest; full-history archives — the only copy a past restore relied on — are never touched. The settings file gains a defaulted field. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child. The destination command is built by `build_agent_launch_command`, which runs `is_safe_agent_option` on model and effort and checks effort against `agent_effort_options`, and the resume path keeps the options through `launch_option_shapes`. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, `README.ko.md`: handoff options, reuse of an unchanged conversation, seven-day expiry, and the two new budgets. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Everything is local file work; no network call and no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The archive walk is capped at 4 096 entries per pass and each manifest read at 256 KiB, both constants in `src/config.rs` and stated in the READMEs; the snapshot read keeps `TRANSCRIPT_FILE_MAX_BYTES`. |

## Flagged concerns

- **The bundle names a snapshot path that expiry may later delete.** A
  destination session resumed more than seven days after its last handoff can
  no longer open the full transcript its bundle points at. Answered: the
  bundle's excerpt and task are already in that session's own conversation, the
  source transcript remains in the source CLI's store, and seven days since
  last use is long against the minutes-to-hours a continuation reads its
  snapshot in. The constant can move without a shape change.
- **An empty note on reuse carries the earlier note.** A person who
  deliberately wants no note on the second attempt gets the first one.
  Answered: the case this change exists for is the wrong-destination retry,
  where the note field has just been cleared by the first attempt; carrying the
  note is what makes the retry free. Typing a new note replaces it.
- **"Unchanged" is judged by size and modification time.** A transcript
  rewritten to the same length within the same nanosecond would be judged
  unchanged. Answered: every CLI here appends to its transcript, and a
  timestamp at nanosecond resolution plus the byte length is what `make` and
  `rsync` trust for the same question.
- **The entry ceiling counts archives the sweep never deletes.** Full-history
  archives from before change 179 share the root and are walked too, in
  `read_dir`'s arbitrary order, so in principle they could keep compact
  archives past the ceiling unseen. Answered: this machine holds 28 such
  archives against a ceiling of 4 096, and none are written any more (change
  179 retired that restore), so their number cannot grow toward it; a compact
  archive missed by one pass is swept by a later one or, at worst, kept.

## Acceptance

- `cargo test --locked` passes, including
  `handoff_launch_settings_round_trip_and_drop_what_the_gate_refuses`,
  `a_handoff_starts_its_destination_on_the_chosen_model_and_effort`,
  `a_handoff_from_the_app_records_the_destination_model_in_the_session`,
  `an_unchanged_conversation_handed_off_again_reuses_its_snapshot_and_note`,
  `a_reuse_whose_snapshot_is_gone_takes_a_fresh_one`,
  `a_conversation_that_moved_on_is_snapshotted_again`,
  `handoff_archives_unused_for_the_ttl_are_deleted_and_nothing_else_is`,
  `an_expired_archive_is_swept_rather_than_reused`,
  `the_app_sweeps_expired_handoff_archives_once_at_startup`,
  `finish_full_history_import_says_when_a_bundle_was_reused`, and
  `the_session_menu_names_the_handoff_model_and_effort`, with
  `the_transcript_archive_is_readable_only_by_its_owner` still green.
- `cargo test --locked -- --ignored` for the live-CLI restore tests, since the
  shared-session archive moved, or a stated reason it could not run.
- In the running app: setting `gpt-5-mini` for Codex in a history card, handing
  a conversation to Claude Code by mistake and then to Codex CLI, shows the
  reuse notice the second time, and the Codex terminal's command line carries
  `--model gpt-5-mini`.

## Rejected alternatives

- **Reuse from a button on the recent-handoff line**: a second destination
  picker and model row per past handoff; the same card the person already used
  does the job when the import recognises the unchanged conversation itself.
- **Use the launch sheet's remembered model for handoffs**: changes what
  today's handoffs start on without anyone choosing it, and couples a cheap
  continuation model to the everyday one.
- **Expire by creation time only**: a bundle reused on day six would lose its
  snapshot a day later while a session that just received it is running.
- **Record reuse data in `CliHandoff`**: a persisted store shape change for
  data the archive's own manifest already holds.
- **Summarise the conversation with a model**: costs tokens on every handoff,
  which is the cost this change exists to avoid.
