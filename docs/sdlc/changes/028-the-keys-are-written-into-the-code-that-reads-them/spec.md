# Spec: the keys come from a file, and the palette shows the ones that are bound

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Seven actions have ids and default chords: opening the palette, adding a
   project, the three pages, settings, and find-in-terminal. The ids are the
   contract with the file and do not change without a migration story.
2. A JSON file in the app's own data directory rebinds any of them by id. An
   absent file, an empty object, and an id the file does not mention all mean
   "keep the default". `null` means "no key at all".
3. A chord is parsed from a string: modifiers separated by `+`, then one key.
   `Mod` is ⌘ on this platform. An unparsable chord is a finding, not a panic.
4. A file whose resulting map gives one chord to two actions is refused whole.
   The defaults are used, and a notice names the chord and both actions.
5. A file that is not JSON, is not an object, or names an id this build does not
   have is refused the same way, with a notice that says which.
6. The palette draws each action's current chord at the right of its row, read
   from the same map the key handling reads. The two labels that have a
   shortcut typed into them lose it.
7. One palette action creates the file from the current defaults when it is
   absent and reveals it, so that the format does not have to be copied out of a
   README by hand. It never overwrites a file that is there.
8. The file is written the way every other file this app owns is written:
   to a temporary name and renamed, so a crash mid-write cannot leave half a
   keymap.

## Behaviour

The palette's rows gain a right-aligned chord in the muted text colour:

```
プロジェクトを開く                          ⌘1
セッションを開く                            ⌘3
設定を開く                                  ⌘,
プロジェクトフォルダを選択…                 ⌘O
ワークスペースをスキャン…
キー割り当てファイルを開く
```

A row with no chord shows nothing rather than a placeholder. Chords are drawn in
the macOS convention — `⌘⇧F`, glyphs adjacent, no separators — because that is
what the menu bar of every other application on this machine does.

`キー割り当てファイルを開く` writes the defaults and opens the containing folder
in Finder when the file is absent, and only opens the folder when it is there.
The notice afterwards reads `キー割り当てファイルを作成しました。` or
`キー割り当てファイルの場所を開きました。`

Not-happy states, each a notice and the defaults still running:

- Two actions on one chord —
  `キー割り当てファイルを読み込めませんでした: ⌘K が「パレットを開く」と「設定を開く」で重複しています。既定のキーで起動しました。`
- A chord that will not parse —
  `キー割り当てファイルを読み込めませんでした: 「Mod+」はキーの書き方として読めません。既定のキーで起動しました。`
- An id this build does not have —
  `キー割り当てファイルを読み込めませんでした: 「palette.opne」という操作はありません。既定のキーで起動しました。`
- Not JSON, or not an object —
  `キー割り当てファイルを読み込めませんでした: JSON として読めません。既定のキーで起動しました。`

The file is read once, at startup. A keymap that changed underneath a running
app would mean a key doing one thing in one window and another thing after a
save nobody saw; restarting is the honest boundary, and the notice above appears
at the same moment every other startup notice does.

## Design

`src/app/keymap.rs`, a child of `app` beside `screens.rs`:

- `KeymapAction { id, label, default }` and `KEYMAP_ACTIONS: &[KeymapAction]` —
  the seven, in the order the palette draws them. `label` is a message id.
- `parse_chord(&str) -> Option<Chord>` and `chord_label(Chord) -> String` — the
  grammar and its macOS spelling, inverse to each other over every default.
- `Keymap` — `Vec<(&'static str, Option<Chord>)>` in registry order, with
  `chord_for(id)` and `action_for(chord)`.
- `resolve_keymap(file: &str) -> Result<Keymap, String>` — pure. Takes the
  file's bytes and returns either a map or the Japanese sentence that says what
  is wrong with it. Every requirement above except 7 and 8 is decided here, with
  no filesystem and no window.
- `keymap_path()`, `read_keymap()`, `write_default_keymap()` — the three
  functions that touch the disk, and the only ones.

`Chord` is a `(egui::Modifiers, egui::Key)` pair in a struct that derives
`PartialEq` — `egui::Modifiers` compares by field, which is what "the same
chord" means. `Modifiers::command` is the ⌘ this platform reports.

`src/app.rs` holds a `keymap: Keymap` on `OperonApp`, built at construction from
`read_keymap()` with the error going straight into `self.notice`. The literal
`Modifiers::COMMAND` / `Key::K` pairs at the top of the frame become lookups of
`self.keymap.chord_for("palette.open")`.

`src/app/screens.rs` draws the chord on each palette row from the same
`chord_for`, and `palette_actions()` loses the two labels' baked-in shortcuts.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The chord is drawn in `palette.text_muted`, an existing role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes, and the answer is no | The four modifier glyphs are not icons. `every_icon_resolves_from_the_bundled_icon_font` asserts that every `ICON_VOCABULARY` entry is drawn by the Phosphor face and *not* by the UI face; ⌘⇧⌥⌃ are the opposite — they come from the text font, and are what the menu bar of every macOS application draws. They are spelled once, in `chord_label`, which is the same rule the icon vocabulary exists to enforce, and they get the mirror guard: a test that the bundled UI font has a glyph for each, so the palette cannot show four empty boxes. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The action ids are agreed between the file, the key handling, and the palette. They live once, in `KEYMAP_ACTIONS`, and the palette and the handler both read them from there. The file name lives in `src/config.rs` beside the other names this app owns. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No store change. The one write is the default file, written to a temporary name and renamed, and refused outright if a file is already there. A corrupt or truncated keymap degrades to the defaults with a notice, never a panic. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Revealing the folder uses the existing Finder call rather than a new one. |
| Documentation — user-facing docs change in all three languages together | Yes | The file, its path, and the seven ids in `README.md`, `README.ja.md`, `README.ko.md`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | A local file, read once. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The keymap file is read with a 64 KiB ceiling and refused above it. Seven actions is the whole vocabulary, so a file cannot name more than seven bindings that mean anything. |

## Flagged concerns

- **Where the file lives.** Settled: the app's own data directory, beside the
  store and the sidecars from changes 019 and 020, rather than a second home
  under `~/.config`. One place for everything this app owns is worth more than
  matching a second home under the user's home directory, and requirement 7 is what makes an
  awkward-to-type path a non-issue.
- **Reading the file once.** Settled: at startup. Watching it would mean the
  same key doing two things in one session depending on when it was last saved.

## Acceptance

- `cargo test --locked` passes, including:
  - `every_default_chord_survives_being_written_and_read_back`
  - `a_keymap_rebinds_an_action_by_id`
  - `a_chord_claimed_twice_refuses_the_whole_file`
  - `a_keymap_that_cannot_be_read_keeps_the_defaults_and_says_why`
  - `the_palette_draws_the_chord_the_handler_listens_for`
  - `writing_the_default_keymap_never_overwrites_one_that_is_there`
- In the running app: ⌘K shows every action with its key; rebind
  `palette.open` to `Mod+Shift+K`, restart, and ⌘⇧K opens the palette while ⌘K
  does not.

## Rejected alternatives

- **A settings pane.** The file is the editor, and a pane that edits bindings is
  a second source of truth for the same data plus a conflict dialog.
- **Watching the file.** Rejected above.
- **A `platforms` section.** Three platform blocks in a file read by a
  macOS-only application is a shape that means nothing here.
- **`null` versus `false` for unbinding.** Accepting both is possible. One spelling that
  works is better than two that do the same thing.
- **Leaving the shortcut in the palette label.** It is the defect this change
  is partly about.
