---
name: durability-invariants
description: Design intent behind Operon's persistence, cancellation-sidecar, store-migration, and process-lock code. Read before changing the local store, the cancellation path, tmux stop/retry logic, store migration, or any scan/output byte limit.
user-invocable: false
---

# Durability design intent

Only the things you **cannot** infer from reading the code or `README.md`. The
store, its locks, and `write_file_atomically_with_sync` live in `src/store.rs`;
every scan and output budget is a constant in `src/config.rs`, which is the only
place those numbers are written. Read those two files directly.

## `CommittedButNotSynced` is not a failure

`write_file_atomically_with_sync` returns `Durable` when the rename committed and
the parent-directory sync was confirmed, and `CommittedButNotSynced` when the
rename committed but the directory sync failed.

The second case means **the data is already on disk**. The app keeps the
committed in-memory state, shows a persistent warning, and retries. Mapping it
onto the failure branch — or collapsing `WriteOutcome` into a bool — makes the app
report that a change was rejected when it actually landed. That is the single
easiest way to break this code.

## Cancellation intent is durable before tmux changes

Intent goes to **both** the current index and the sidecar before tmux is touched,
and **at least one copy must confirm its containing-directory flush**. If neither
confirms, tmux is left running and persistence retries.

This is what lets an interrupted stop stay pending for retry, and a completed stop
reconcile as cancelled after restart. Never move the tmux kill ahead of
persistence, and never accept zero confirmed copies.

## Legacy files are the rollback snapshot

Every legacy index and sidecar must stay byte-for-byte untouched after migration —
they are the only pre-upgrade rollback path. A malformed legacy sidecar is
preserved for diagnosis rather than repaired.

`legacy_app_data_files` is ordered newest-first and the **first candidate that
exists wins**; the rest are not consulted. Reordering it, or continuing past a
successful import, lets an older snapshot overwrite a newer one. The pre-rename
`com.local.xirp-copy` directory is one of those candidates, and its instance lock
is held alongside this build's — a build under the previous product name keeps a
separate index, so nothing else stops the two from driving the same tmux sessions.

Likewise, a store **newer** than this build is preserved, never overwritten: the
app creates an empty current index and surfaces where the snapshot remains,
instead of looping on a startup failure.

`STORE_SCHEMA_VERSION` (currently 4) is independent of the file name
`store-v2.json`. They do not track each other.

## The instance lock has no stale-lock recovery, deliberately

`acquire_instance_lock` uses `flock(LOCK_EX|LOCK_NB)` because the kernel releases
it on crash. Do not add a stale-lock reclaim path — that reintroduces the
two-simultaneous-reclaimers race the current design avoids. (Packaging separately
uses `/usr/bin/lockf`, for the same reason.)

## Rules for changing a budget constant

The values are documented in `README.md` as user-visible behavior, so:

1. Update `README.md` in the same change.
2. A truncated result must still be **reported as truncated**, never returned as
   if complete.
3. Enforce by counting **actual reads on the open handle**, not a pre-read
   `metadata()` size — a concurrently growing agent log must not slip past.
4. For subprocess output, stop the oversized child **while reading**; do not
   buffer an unbounded diff and trim afterwards.

## Shared-session archive holds sensitive data

The cross-CLI restore archive keeps the entire original Claude JSONL transcript:
prompts, tool output, source code. Never copy it outside the app data directory
or log its contents.

Its directories are created at `SHARED_SESSION_DIRECTORY_MODE` (`0o700`) and the
snapshot at `SHARED_SESSION_FILE_MODE` (`0o600`), because `create_dir_all` and
`OpenOptions::create_new` both apply the umask and a stock Mac's leaves them
`0o755`/`0o644` — readable by every other account. `manifest.json` and
`restored-conversation.md` are written through `write_file_atomically` and are
protected by the directory, not by a mode of their own, so a new file added
beside them inherits that protection and a new *directory* does not. Widening
either mode, or creating an archive directory without `create_private_directory`,
is the finding.
`the_transcript_archive_is_readable_only_by_its_owner` is the only check.
