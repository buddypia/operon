# Intent: The keys every tabbed Mac app answers to are missing or hidden

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

The person asked: 「業界標準の一般的なショートカットを導入したい。タブを閉じる、新しいタブを開くなど」

The Sessions workspace is a row of tabs, and a person who uses a browser, an
editor, or a terminal on a Mac reaches for the same keys here. Today:

- ⌘T does nothing. A new terminal tab is only the + at the end of the strip.
- ⌘W closes the active tab, but nothing on screen says so: the Settings key
  list and the command palette do not mention it, and it cannot be remapped.
- ⌘⇧] / ⌘⇧[ (next / previous tab) do nothing. Only ⌃Tab works, and ⌃⇧Tab
  moves forward instead of back.
- A tab closed by mistake cannot be brought back with ⌘⇧T; the person has to
  find it again in the sidebar (a document tab has to be found in the file tree).

## Who feels it, and when

The person working in the Sessions workspace with several terminals and files
open, who presses ⌘T for one more terminal, ⌘⇧] to step through the tabs, or
⌘W then ⌘⇧T after closing the wrong one — the keys their hands already know
from Safari, Chrome, Terminal, iTerm2 and VS Code — and gets nothing, or the
wrong direction.

## Desired outcome

Adopted default set (decided, not asked: every key is remappable through the
existing keymap file, so a wrong default costs one line in that file):

| Key | What happens |
|---|---|
| ⌘T | A new plain terminal tab opens in the focused pane, exactly as the + does. From any page it brings the person to the Sessions workspace. |
| ⌘W | The active tab closes (unchanged behaviour). |
| ⌘⇧T | The most recently closed tab that can still be shown opens again and is selected — a terminal whose session still exists, or a file whose project still exists. Nothing to reopen says so briefly. |
| ⌘⇧] / ⌘⇧[ | The next / previous tab in the focused pane is selected, wrapping round. |
| ⌃Tab / ⌃⇧Tab | Next / previous, as browsers keep them; ⌃⇧Tab now goes back. |

Each of the five ⌘ keys appears in Settings → キー with its chord, can be
changed or turned off in the keymap file, and ⌘T and ⌘⇧T are rows in the
command palette with their chord beside them. Pressing ⌘⇧T never also opens a
new tab, and ⌘⇧] works on both US and JIS keyboards.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- A terminal pane owns its own keys; only ⌘ chords (which the pane never sends)
  and the existing ⌃Tab pair are the application's.

## Systems likely affected

The keymap (`src/app/keymap.rs`), the app-wide shortcut dispatch and tab
helpers (`src/app.rs`), the workspace drawing and palette (`src/app/screens.rs`),
message tables (`src/i18n_tables.rs`), tests in `src/tests.rs`.

## Open questions

None. The set above is the adopted default; ⌘1–⌘9 is listed under Not in scope
with its reason.

## Not in scope

- ⌘1–⌘9 selecting the Nth tab: ⌘1–⌘3 already open Home, Projects and Sessions,
  and taking them away would break keys people use today.
- ⌘⌥← / ⌘⌥→ as a third pair for next/previous: the keymap holds one chord per
  action, and ⌘⇧[ ] plus ⌃Tab already cover it.
- Remembering closed tabs across a relaunch; the list lives as long as the
  window.
- ⌘S, ⌘Q, ⌘H and ⌘+ / ⌘- / ⌘0 zoom: these already work (the editor, winit's
  default app menu, and egui's keyboard zoom) and are not changed. ⌘M is not in
  winit's default menu; window commands are a separate change.
