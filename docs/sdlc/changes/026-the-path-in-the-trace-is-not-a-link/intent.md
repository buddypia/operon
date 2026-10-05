# Intent: the file the trace names is right there, and it is only text

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

An agent fails, prints a stack trace, and the useful part is a file and a line.
Operon has the file — it has an editor with a project tree and a Git badge on
every file the agent touched — and no way to get from the trace to it. So a
person reads the path, remembers it, goes to the editor, expands the tree, finds
the file, and scrolls to the line. Four steps to follow a pointer that was
already on screen.

The same is true of the address a dev server prints. It is right there, and it
has to be retyped.

## Who feels it, and when

Every failure that names a file, which is most of them, and every dev server
that prints where it is listening.

## Desired outcome

The path in the output is the way to the file, and the address is the way to the
page. Following one costs a click. Nothing that cannot be followed pretends it
can be.

## Constraints this change inherits

- The terminal pane virtualises its rows, and nothing may become work
  proportional to fifty thousand lines per frame.
- No filesystem call in a draw path.
- The pane owns typing. A click must not take the keyboard away from an agent
  that is being talked to.
- macOS only; local-first; Japanese for what a person reads.
- No new dependency: a pattern crate is a paused surface, so the two shapes are
  recognised by hand.

## Systems likely affected

`src/ui/terminal.rs` for finding and drawing. `src/app.rs` for resolving a name
to a file and for what a click does. The editor, which has never had a reason to
scroll to a line before.

## Open questions

- **What does a click do?** Answered before the code: it opens, directly, with
  no menu in the way. The other two actions are on the right button.

## Not in scope

- Paths without an extension, which are real but are also every other word.
- A configurable external editor. The editor here is the one that already knows
  about the project.
- Line ranges, columns, and location forms other than a path followed by a line.
