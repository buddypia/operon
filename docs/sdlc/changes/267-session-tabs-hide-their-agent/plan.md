# Plan: session tabs show which CLI runs in them

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `assets/terminal-mark.svg` | New: the terminal mark's vector source. |
| `assets/terminal-mark-128.png` | New: the mark rendered at 128x128 (rsvg-convert at 1024, Lanczos to 128). |
| `src/glyphs.rs` | `TERMINAL_MARK`, `SESSION_MARK_SIZE`, documented beside `BRAND_MARK`. |
| `src/ui/widgets.rs` | `session_mark_texture`, `session_mark`. |
| `src/app/screens.rs` | The session tab draws `session_mark` after its state mark. |
| `src/tests.rs` | The R1–R3 guards. |
| `DESIGN.md` | One sentence on the terminal mark beside the vendor marks. |
| `docs/sdlc/changes/267-session-tabs-hide-their-agent/*` | This change's record, `verification.yaml`, `mutations.py`. |

## Order of work

1. Add the asset files and the two constants (the tree compiles; nothing draws them yet).
2. Write the R1–R3 tests against helpers declared with an empty body (`session_mark_texture` returning `None`, `session_mark` drawing nothing), and watch R1–R3 fail.
3. Implement the helpers and the tab call; watch R1–R4 pass.
4. `DESIGN.md` sentence; mutation sweep; `make q.fast`; named tests.
5. `verification.yaml`; independent empirical verifier in its own worktree; commit, push, CI, merge from `main`, clean up, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The mark lands outside the tab, or after the title | A tab reads title-then-icon, or the icon floats | R1 finds the texture between the state mark and the title on the title's row. |
| A custom command gets the terminal mark | A non-terminal reads as a shell | R1 asserts no mark in the custom tab; R2 pins the mapping. |
| The new image is not a rounded square, or is not 128x128 | The terminal mark reads as a different kind of object beside the vendor marks | R2 checks size, opaque centre, transparent corners. |
| The extra width pushes the close button or breaks drag/click | Tabs stop responding | R4's existing pointer tests. |
| Per-frame upload | Stutter, memory growth | `png_texture` caches by name; reviewed against `.claude/rules/rust.md`. |
| `src/agents.rs` gets edited after all | Launch-gate formal-proof obligation triggers | Plan lists it as untouched; the diff is checked before commit. |

## Proof of completion

- `cargo test --locked session_tabs_show_the_cli_mark_after_the_state`, `the_terminal_mark_is_a_rounded_square_for_terminal_sessions_only`, `session_tab_mark_names_its_cli_on_hover` — each `1 passed`, each seen failing first.
- `cargo test --locked workspace_session_tab_shows_its_status_glyph` and `workspace_pointer_` — pass.
- `python3 docs/sdlc/changes/267-session-tabs-hide-their-agent/mutations.py` — every mutation caught, source restored.
- `make q.fast` — passes; CI green on the branch head before the merge.
- A screenshot of the rendered tab strip compared against `screen.md`.

## Departures from the plan

Before any source edit, R3 dropped its "accessible label" clause: egui only reports a widget's name on a click or focus event, and the mark is hover-only, so no guard in this suite could watch that claim fail. The spec was re-evaluated at its new digest.

`session_254_compact_tabs` pinned the old layout ("title within 65pt of the column edge"). Its bound now grows by exactly one `SESSION_MARK_SIZE` plus one item gap, so it still catches a title pushed right by anything else.

`launch_stored_session_does_not_block_on_workspace_trust_polling` failed once while the related tests ran in parallel (it times a real tmux launch against two seconds) and passed when run alone; this change touches no launch code.

Screenshot: the worktree build ran against a throwaway HOME and its own tmux server, seeded with Antigravity, Claude Code, terminal, and custom sessions. Synthetic clicks are not permitted on this machine, so a temporary startup patch in `src/app.rs` opened the four tabs; it was reverted with `git checkout -- src/app.rs` before anything was staged. The captured strip reads `[⏸ カスタム ×] [⟳ (terminal mark) セッション3 ×] [⏸ (Claude mark) セッション1 ×] [⏸ (Antigravity mark) セッション2 ×]`, as `screen.md` describes.

Proof: R1–R4 named tests pass, mutation sweep 9 of 9 caught with the source restored, `make q.fast` passes.

The empirical verifier's isolated worktree could not reach the branch commit (the trunk and destructive-git guards refused both a detached checkout and `git worktree add`), so it ran in this worktree at 69d68bd: `scripts/run-obligations.sh` held all eight obligations in 181 s, and its reading of the document-tab, missing-session, other-surface, unknown-agent, and texture-cache cases found nothing to reproduce.

Band diagnosis: `largest_module_lines` was already 9989 (`src/app/screens.rs`) on main, over its 9400 diagnose tier; this change adds two lines.
