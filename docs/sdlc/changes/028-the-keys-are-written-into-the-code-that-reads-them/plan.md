# Plan: the keys come from a file

- **Spec**: `./spec.md`

## Order

1. **`src/app/keymap.rs`, pure first.** The registry, `parse_chord`,
   `chord_label`, `Keymap`, and `resolve_keymap`. None of it touches a disk or a
   window, so all of requirements 1–5 are decided and tested before anything can
   be wired up wrong.
2. **Tests, watched failing by mutation.** Including the inverse pair:
   `chord_label(parse_chord(s)) == s` over every default, which is what stops
   the palette showing a chord the handler is not listening for.
3. **`src/glyphs.rs`** — the four modifier marks and their `ICON_VOCABULARY`
   rows, before any code writes one at a call site.
4. **`src/config.rs`** — the file name, once.
5. **The disk half**: `keymap_path`, `read_keymap` with its byte ceiling,
   `write_default_keymap` with a temporary name and a rename and a refusal when
   the file exists.
6. **`src/app.rs`** — the field, the startup read, and the six literal chords at
   the top of the frame replaced by lookups.
7. **`src/app/screens.rs`** — the chord column, the two labels losing their
   baked-in shortcut, and the new palette action.
8. **READMEs in three languages**, and the roadmap row.

## What breaks, and how it is caught

- **The palette shows a key the handler does not answer to.** The whole point of
  the change, and the easiest thing to reintroduce: one of the two reads the map
  and the other keeps a literal. Caught by
  `the_palette_draws_the_chord_the_handler_listens_for`, which asserts the
  palette's drawn chord equals `chord_for` for the same id, and by
  `every_default_chord_survives_being_written_and_read_back` for the spelling.
- **A conflict applied halfway.** A keymap where some bindings took and some did
  not is one nobody can reason about. `a_chord_claimed_twice_refuses_the_whole_file`
  asserts that the returned map is the default map, not a partial one.
- **A bad file taking the app down at startup.** The read happens before there
  is a window to show an error in, so every failure has to be a `Result` that
  becomes a notice. `a_keymap_that_cannot_be_read_keeps_the_defaults_and_says_why`
  walks a list of broken files — not JSON, an array, an unknown id, an
  unparsable chord, a chord with no key — and asserts a message and the defaults
  for each.
- **Overwriting a keymap somebody wrote.** The one destructive thing here.
  `writing_the_default_keymap_never_overwrites_one_that_is_there` writes a file,
  asks for the default to be written, and asserts the bytes did not move.
- **Work in a draw path.** The file is read once at construction, not per frame.
  The palette's chord lookup is a walk of a seven-entry `Vec` and a `String`
  built from at most five glyphs, per visible row, which is the same order as
  the label beside it.

## Assumptions, and what would falsify them

- **`egui::Modifiers` compares the way "the same chord" means.** It derives
  `PartialEq` over its bool fields, so `COMMAND` on macOS is `mac_cmd` plus
  `command`. Falsified by two chords that look identical comparing unequal,
  which the round-trip test over the defaults would show.
- **Startup has somewhere to put a notice.** `self.notice` is drawn by the frame
  loop and is what every other startup problem uses.

## Not done here

- A settings pane, file watching, chord sequences, per-scope bindings, and
  anything that reaches a terminal pane. Each is argued in `spec.md`.

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, with
  N six higher than before this change.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no metric newly breached.
- Each of the six new tests watched failing under a mutation that names what it
  guards, and the mutation confirmed to have landed on the line it was aimed at.
- In the running app: ⌘K lists every action with the key beside it; the palette
  action writes the file and reveals it; rebinding `palette.open` to
  `Mod+Shift+K` and restarting makes ⌘⇧K open the palette and leaves ⌘K doing
  nothing; putting the same chord on two ids shows the conflict notice at
  startup and leaves the defaults running.

## Departures from the plan

- **Step 3 was wrong and was dropped.** The plan put the four modifier glyphs
  into `src/glyphs.rs` with `ICON_VOCABULARY` rows, and `spec.md` said the same.
  `every_icon_resolves_from_the_bundled_icon_font` asserts that every entry in
  that vocabulary is drawn by the Phosphor face and *not* by the text face; ⌘⇧⌥⌃
  are the opposite of that. They are spelled once in `chord_label` instead, and
  they got the mirror guard — `every_chord_modifier_glyph_is_drawn_by_the_ui_font`
  — so the failure the icon rule exists to prevent, four empty boxes on screen,
  is still caught. `spec.md` was corrected rather than left disagreeing.
- **`palette_actions` lost a duplication rather than gaining a column.** Four of
  its six labels were already written in `KEYMAP_ACTIONS`, so those rows read
  their label from the registry. The alternative was the same Japanese string in
  two files, which is the thing the identifier rule is about.
