# Intent: the keyboard is fixed, and the screen says a key that may not be the one

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Every shortcut this app answers to is decided in its source. ⌘K opens the
palette, ⌘O adds a project, ⌘1 through ⌘3 change page, ⌘, opens settings, ⌘F
searches a terminal — and none of them can be anything else. A person whose
hands already know a different arrangement, or whose keyboard makes one of these
awkward, has no way to say so.

There is a smaller problem underneath, and it is the one that is already wrong
rather than merely missing: two of the palette's rows have their shortcut typed
into the label. The row reads `プロジェクトフォルダを選択…  ⌘O` because somebody
wrote `⌘O` in the string. Nothing checks that against the key the code actually
listens for, so the palette is a place where the app tells you which key to
press and is not obliged to be right.

## Who feels it, and when

Anybody who wants a different key, immediately and permanently — there is no
workaround, not even a bad one. And anybody reading the palette, every time,
since the label is the only place the app names a shortcut at all.

## Desired outcome

The shortcuts are data. A file names each action by an id and gives it a chord,
the app reads it at startup, and what the palette shows is what the app will
answer to — because it is drawn from the same place the key handling reads.
A file that contradicts itself is refused as a whole and says why, rather than
being applied halfway.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface: the keymap is a file of its own, not
  a store field, and no schema version moves.
- A new top-level module is a line in `src/main.rs`, which is also paused. This
  goes beside `src/app/screens.rs` as a child of `app`.
- No new dependency: `serde_json` is already here.
- Terminal keys are not in scope to remap. A TUI owns its own keyboard, and the
  keys that reach it are not this app's to reassign.

## Systems likely affected

A new child module under `src/app`, the shortcut handling at the top of the
frame in `src/app.rs`, and the palette's rows in `src/app/screens.rs`.

## Open questions

- What happens when two actions claim one chord? — answered by the person, in
  `spec.md`.
- Where does a person see the key that is actually bound? — answered by the
  person, in `spec.md`.

## Not in scope

- A settings pane that edits bindings. The file is the editor.
- Remapping anything that reaches a terminal pane, or the palette's own
  navigation keys — arrows, Enter, Escape — which belong to the widget that has
  focus rather than to the app.
- Chord sequences, double-taps, and per-scope bindings. One chord, one action.
