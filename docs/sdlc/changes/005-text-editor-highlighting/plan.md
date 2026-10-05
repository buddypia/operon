# Plan: Syntax-highlighted project text editor and review flow

- **Spec**: `./spec.md`
- **Approved**: 2026-08-29
- **Status**: done

This plan is approved from the stated goal: provide an editable text surface for
common project formats and preserve a visible final diff of agent work.

## Files that change

| File | Change |
|---|---|
| `src/files.rs` | Add the pure, shared editor-language classification. |
| `src/ui/syntax.rs` | Replace lexical rules with cached grammar-backed layouts and a semantic-palette Syntect theme. |
| `src/ui/mod.rs` | Register and re-export the focused syntax module. |
| `src/app.rs` | Use cached syntax and gutter layouts; refresh Git views after external working-tree changes. |
| `src/tests.rs` | Prove classification, grammar layout safety, cache behaviour, Diff freshness, and editor rendering. |
| `Cargo.toml`, `Cargo.lock` | Add the audited, local Syntect grammar dependency. |
| `CLAUDE.md` | Keep the UI module map discoverable for the next contributor. |
| `README.md`, `README.ja.md`, `README.ko.md` | Describe the editing, preview, and comparison capability consistently. |

## Order of work

1. Add failing tests for a multiline embedded-language layout, cache reuse, and
   a Diff invalidated by an external working-tree change.
2. Add Syntect with bundled syntax definitions; implement grammar-backed layout
   construction, semantic palette mapping, and egui frame caching.
3. Cache the line-number gutter on each open document and invalidate it only
   when the editable buffer changes.
4. While the editor Diff is visible, periodically discard and re-request that
   file's actual Git Diff. Refresh Git status for tree badges separately, but
   do not mistake an unchanged status line for unchanged file content. Fence
   background Diff results with generations and re-request after a stale
   in-flight result completes.
5. Run the three Rust gates, independent review, packaging, and installed-app
   replacement required by `AGENTS.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A syntax rule breaks normal text editing | Editor panics or text no longer lays out | Standalone layout test and the real editor draw test. |
| An extension is misclassified | Wrong tokens are coloured or a format silently loses support | Exhaustive extension classification test. |
| A long file makes the immediate-mode UI stutter | Every frame tokenizes the same 128 KiB buffer or rebuilds the gutter | Frame-cache and gutter-cache tests; manual installed-app check with a large file. |
| An agent changes a file while Diff is open | The screen presents old Git output as current | Direct-Diff refresh test, including a repeated write that leaves Git status unchanged and an older in-flight result. |
| A bundled grammar draws colours outside the design system | Syntax text ignores the active palette or becomes unreadable | Theme construction accepts only semantic palette roles; existing palette contrast tests. |
| Colours drift from the design system | Unreadable editor tokens or literals at call sites | Existing palette/document/contrast tests and semantic-role-only implementation. |
| The review flow regresses | No comparison after opening a file | Existing editor diff rendering coverage plus manual installed-app check. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- Unit tests prove common format classification, grammar layout safety, cache
  reuse, and external-update Diff invalidation; the editor draw test renders
  highlighted source, Markdown, and the Diff tab.
- `bash scripts/package-macos.sh` produces `dist/Operon.app`; the verified
  bundle atomically replaces `/Applications/Operon.app` and is manually checked
  after the previous instance has quit.

## Departures from the plan

The original lexical-only approach did not meet the declared HTML/Vue grammar
scope, egui's per-frame layouter cache requirement, or the final-Diff freshness
contract. This plan therefore adds Syntect and a generation-fenced direct-Diff
freshness refresh.
