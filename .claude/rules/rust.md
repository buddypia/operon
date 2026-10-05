---
paths:
  - "src/**/*.rs"
  - "Cargo.toml"
---

# Writing Rust in this crate

There is no Tokio here, no async, and one SQLite file that is read once per
restore. The axis that matters most is the one a non-drawing codebase lacks:
**this is an immediate-mode GUI, and the draw path runs every frame.**

## The draw path runs sixty times a second

`eframe`/`egui` 0.31 rebuilds the whole UI each frame. Anything inside
`OperonApp::update`, the drawing run in `src/app/screens.rs`, or the widgets they
call in `src/ui/` runs at frame rate, so
work that is merely wasteful elsewhere is a stutter here:

- No `Command::new`, no `fs::read`, no `std::fs` metadata call, and no directory
  scan in a draw path. Those belong in a background task whose result the frame
  reads.
- No allocation per frame that could be hoisted: a `format!` in a loop over rows,
  a `to_string()` to satisfy a `&str` parameter, a `Vec` rebuilt to be searched
  once. Cache it on the app state or compute it when the input changes.
- Parsed and highlighted text is derived state. `src/markdown.rs` and
  `src/ui/syntax.rs` are expensive; their output is cached and the cache key is
  the thing to check, not the parser.

Blocking the frame is the failure mode this codebase actually ships. A slow
review comment about an extra `clone()` matters less than a `Command::new`
thirty lines into a widget.

## Panics

Production code holds a handful of `unwrap()`/`expect()` calls. A new one needs
a reason at the call site saying why the `None` or `Err` is unreachable, not
merely unlikely — `git diff` shows which ones are new. Everything else
returns `anyhow::Result` and reaches a person as a Japanese string through
`tr` or `tf!`.

`panic!`, `todo!`, and `unreachable!` do not belong in a path a user can reach.
A panic in an `egui` frame takes the window with it and loses whatever was on
screen.

## `unsafe`

`unsafe_blocks` in `scripts/harness-metrics.sh` is the count, and
`docs/sdlc/bands.yaml` bands it at **propose** — not at warn — so a new block is a
pipeline entry, not a review comment. The band's own comment says where the
reading stands. Every block needs a `// SAFETY:`
comment stating the invariant and why this call site upholds it, and
`.claude/agents/subprocess-safety-reviewer.md` reads it.

## Children, and things that come back from outside

Every external tool goes through `run_command_with_output_limit` or
`run_command_with_timeout` in `src/exec.rs`. A raw `.output()` waits forever and
buffers without limit, which is why `unwrapped_spawns` is a banded metric and not
a lint. Launch commands additionally pass `is_safe_agent_command` and
`is_safe_agent_option` in `src/agents.rs`.

The one SQL surface is Antigravity's trajectory store in `src/history.rs`. It is
read-only, opened with `SQLITE_OPEN_READ_ONLY`, and every value crosses through
`rusqlite::params!`. A query built with `format!` is an Important finding even
against a local file: the path and its contents come from another product.

## Ownership, when it is worth mentioning

Only when the borrow checker was fought rather than satisfied:

- A `clone()` that exists to escape a borrow, where restructuring the scope or
  taking `&mut` at the caller would do. A `clone()` of a small `Copy`-ish struct
  once per user action is not worth a comment.
- `String` where `&str` reaches, `Vec<T>` where `&[T]` reaches — in a signature,
  which is where it propagates. Inside a function body it is noise.
- An explicit lifetime where elision applies.

## Before writing egui, rfd, or directories API calls

Check the pinned version's docs with context7 first. `eframe`/`egui` 0.31,
`rfd` 0.15, and `directories` 6.0 all changed APIs across recent minor
releases; write from the docs, not from recall.

---

`.claude/agents/rust-reviewer.md` reviews against these axes with its own
context. This rule is what applies them while the code is being written; the
review is the second opinion, and `REVIEW.md` says the session that wrote a
change does not approve it.
