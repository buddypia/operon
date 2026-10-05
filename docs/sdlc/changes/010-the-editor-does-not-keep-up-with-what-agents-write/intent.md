# Intent: the editor shows what was loaded, not what the agent just wrote

- **Status**: draft
- **Opened**: 2026-09-01

## Problem

Operon's editor exists because agents write their plans, notes, and reviews into
the working tree a person is reading. Three things fall short of that purpose
today.

**The page goes stale without saying so.** An agent writes to a file while it is
open in the front tab, and what is on screen stays what was loaded. Nothing marks
it. The way out — reload from disk — is a button a person has to already know is
there, and has to think to press. The person who most needs the file is the
person least likely to notice: they are watching a terminal, not the tab.

**Rendered Markdown is rebuilt from nothing on every frame.** The rendered view is
the default for Markdown, which is what agents write, so the pane a person spends
the most time in is the one that re-does the most work sixty times a second. On a
long plan this is felt as the window going heavy while nothing is happening.

**The renderer does not know two of the marks agents write most.** A checklist
reads as literal `[ ]` and `[x]` brackets, and a fenced block with a language on
it is drawn as flat grey prose, uncoloured — while the same file one tab over, in
the source view, is coloured. A plan read as raw syntax is a plan read in the
wrong format, which is the reason the rendered view exists at all.

## Who feels it, and when

Someone with an agent running in a managed terminal and that agent's plan or
review open in the Files tab. It is the ordinary way this app is used: start the
agent, then read what it produces while it produces it. Every checkpoint the
agent writes is a moment where the screen and the disk disagree, and every plan
it writes has a checklist and a fenced command in it.

## Desired outcome

- A file open in the editor either shows what is on disk now, or says on screen
  that it does not — and offers both the way back to disk and the way to see what
  changed.
- A person who has typed into the buffer never loses what they typed to a
  refresh they did not ask for.
- Scrolling a long rendered document costs no more per frame than scrolling the
  same document as source.
- A checkbox is drawn as a checkbox, and a fenced block with a language is
  coloured the way that language is coloured everywhere else in this app.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI. Nothing that reads the
  filesystem may run in a draw path.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

A guess, to be corrected by `spec.md`: `src/app.rs` for the editor state and its
background work, `src/markdown.rs` and `src/ui/markdown_view.rs` for what the
rendered view knows, `src/ui/syntax.rs` for colouring a fence, `src/files.rs` for
naming a fence's language, `src/glyphs.rs` and `DESIGN.md` for the checkbox
marks, `src/git.rs` for reading a file that may have changed, and `src/tests.rs`.

## Open questions

- Should a file that changed on disk while the buffer is untouched reload by
  itself, or wait to be asked? Answered in `spec.md`: it reloads, and says it
  did. Every editor a person has used behaves this way, and a prompt for a
  question with one sensible answer is a prompt that gets clicked through.

## Not in scope

- Side-by-side diff. The unified diff already there is the same information.
- Find and replace, multiple cursors, code completion, a language server.
- Rendering images, footnotes, or HTML inside Markdown.
- Editing from the rendered view, or saving from it.
- Watching files that are not the front tab, and watching the tree for files
  appearing and disappearing.
