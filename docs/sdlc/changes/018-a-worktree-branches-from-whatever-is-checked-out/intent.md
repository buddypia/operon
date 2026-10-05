# Intent: a new worktree branches from whatever the project happens to have checked out

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

Making a worktree is how a person starts a new piece of work in parallel with the
one they are already running. It should start from the project's mainline. It
starts from whatever commit the project folder is sitting on instead — which,
for anyone who uses Operon the way it is meant to be used, is very often the
last branch they were working in. The new work is then built on top of someone
else's unfinished change, and nothing says so. The mistake is found later, at
review or at merge, when the diff contains commits its author never wrote.

The second half of the same problem is that the attempt can simply be refused. A
person who wants a second attempt at the same task types the same name, and gets
「worktree のパスが既に存在します」. The obvious thing to do — try it again, from
the same starting point, beside the first one — is the thing that is hardest to
do, and it is exactly what parallel work is for.

## Who feels it, and when

Every worktree created from a project folder that is not sitting on its mainline,
which is the normal state of a folder that has had any work done in it. And
every second attempt at a task, because the first attempt took the name.

## Desired outcome

A worktree created from Operon starts from the project's mainline unless the
person is told otherwise, and the message that confirms it says which ref it
started from. Asking for a name that is taken produces a second worktree beside
the first rather than a refusal, and the confirmation names the branch that was
actually made. A first `git push` from the new worktree creates its upstream
without needing `--set-upstream`.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Every child process goes through the wrappers in `src/exec.rs` with a timeout.
- `src/app.rs` is at its size warn band, so logic belongs in `src/git.rs`.

## Systems likely affected

`src/git.rs` gains the base-ref detection, the candidate loop, and the post-add
configuration. `src/app.rs` calls them from `create_worktree` and reports the
branch and base in its existing notice. `src/config.rs` holds the ceiling on
suffix attempts. `src/i18n_tables.rs` gains rows for the new messages.

## Open questions

- **Should creating a worktree fetch first?** Answered in `spec.md`: no. A fetch
  is a network call on a path that must not hang, and the failure mode of a
  stale `origin/main` is strictly better than today's failure mode of an
  arbitrary branch. It is named in the confirmation so a stale base is visible.

## Not in scope

- Choosing the base ref from the interface. This change gives the good default
  and shows it; a picker is a screen and needs one agreed first.
- Automatic names for unnamed work (generated creature names and a retirement
  registry) — that wants a store field, and the store is a paused surface.
- Deletion safety, the prepared-checkout pool, the setup script. Separate changes.
