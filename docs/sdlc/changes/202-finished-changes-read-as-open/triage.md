# Thirty records that said a change was still open

Read on 2026-10-22 against main at 3d5d07e. Every record below arrived in the
squashed import `6ebed6a` (2026-10-05) with a status that was not `done`, and
none was touched again: the merge gate (change 149) reads only the change
directories a branch touches, so a record that reached main open by any other
path was never read twice. Each one was read against the code by a reader that
did not trust the record's own prose: the function, the file and line, and the
test in `src/tests.rs` that pins it.

## Verdicts

| Record | Was | Verdict | Becomes |
|---|---|---|---|
| 008 the pipeline has no state between sessions | reviewing | landed; requirement 4 is weaker than written (the readiness script checks that Acceptance is filled, not that it covers every requirement) and requirement 13 was missed, as its plan says | done |
| 010 the editor does not keep up | reviewing | landed, all seven acceptance tests present | done |
| 011 the brand mark reads as a letter | reviewing | landed; the acceptance test is named `the_brand_is_drawn_as_artwork_…`, and the themes are now Dark, Light, HighContrast (041) | done |
| 012 dead launch preset UI | reviewing | landed; every removed symbol is gone from `src/` | done |
| 015 agent state read off the screen | reviewing | landed; the envelope is one JSON body on `/hook` rather than a path per CLI, and 060 moved Codex's flag under `[features]` | done |
| 016 a finished turn leaves no mark | reviewing | landed; unread is drawn in stronger ink rather than bold, as its plan records | done |
| 017 the palette cannot find a session | reviewing | landed, all five tests present | done |
| 018 a worktree branches from whatever is checked out | reviewing | landed, all eight tests present | done |
| 019 a worktree cannot set itself up | reviewing | landed; the run-time recheck has the gap in A15 | done |
| 020 a review cannot be handed back | reviewing | landed; "a send clears the notes" was replaced by 047's sent and resolved states | done |
| 021 the largest module is two modules | reviewing | landed; `largest_module_lines` has since grown back to 8856, in the warn tier, where `docs/sdlc/bands.yaml` says to record and not act | done |
| 022 the loop ends in the terminal | reviewing | landed, all eight tests present | done |
| 023 which worktree is that server | reviewing | landed; 077 reshaped the line into one badge with a popover | done |
| 024 the answer is somewhere above | reviewing | landed, with the defects in A7 | done |
| 025 how much of the window is left | reviewing | landed, with the defects in A1 and A2 | done |
| 026 the path in the trace is not a link | reviewing | landed; 087 changed the gesture to ⌥ and ⌘ and let ⌘ open a path outside the project | done |
| 027 a restart forgets how the agent was launched | reviewing | landed, all six tests present | done |
| 028 the keys are written into the code that reads them | reviewing | landed, with the leftover in A8 | done |
| 029 Enter goes missing when a conversion lands in the same frame | reviewing | landed, pinned, lesson 016 | done |
| 031 one machine, one account | reviewing | landed, with the gap in A3 | done |
| 032 the release gate asks for what a machine can check | reviewing | landed; 128 later let it take CI's verdict for a clean tree | done |
| 033 four ways the app can lose work | reviewing | landed for A, B, C and E; of the twelve findings in its `remaining.md`, H and G are fixed (034, 087), the other ten are A4–A6, A9–A12, A15, A1, A2 | done |
| 035 quality waits for a person to read the diff | blocked | landed: the lexer, the trampoline, `scripts/check-review.sh`, and about 150 changes judged by it since; its own last verdict was never recorded | done |
| 038 the harness is built but not measured | awaiting-user | partly landed (`scripts/pipeline-indicators.sh`); not built: a gate decision log, a `failed` contract verdict, a band-breach history, a spec | archived |
| 040 nothing builds the app and nothing gates another CLI | awaiting-user | superseded: CI builds the release bundle and runs fmt, clippy, the suite and the bands on every pushed branch, and the merge gate waits for it (128, 191); Codex alone has no merge gate (A18) | archived |
| 041 visibility and accessibility gaps | blocked | landed: the inks, `focus_stroke`, `every_theme_stays_readable`; 136 extended the ring to every button | done |
| 049 the session list cannot be narrowed | reviewing | landed; 088 removed the chip icons; the two remarks it carried are A13 | done |
| 077 streamline the session view | reviewing | landed; its two tests assert nothing (A13) | done |
| 110 AI IDE competitive features | in-progress | landed (115's intent records it shipped), with A10, A11, A16, A17 | done |
| 111 adversarial security and resilience hardening | in-progress | landed, with A14 and A15 | done |

002 was already `archived` and is not in the table.

## Why 038 and 040 are archived rather than finished

038 set out to measure the harness. What it built — leading indicators from git
and the change directories — is in use. What it did not build has no reader
three weeks later: a decision log waits on a question about what it may record
that nobody needed answered, a `failed` verdict changes nothing a gate does (a
check that fails stays pending and blocks), and first-pass CI success now comes
from the CI history this repository did not have when 038 was written. Starting
any of it again is a new change with a reader named first.

040 asked for the release build and the three gates to stand at a boundary
every CLI crosses. CI is that boundary now. The one gap left is A18.

## Defects found, and where each is fixed

Each was confirmed on the code, not taken from the record.

| | Defect | From |
|---|---|---|
| A1 | The usage reader's install marker is one file for every account folder, written whatever the slot held, before the settings file, and never cleared: a second Claude account never gets the reader, and turning hooks off and on loses it for good | 025, 033 D |
| A2 | The usage throttle compares the CLI's elapsed time against a stamp kept per pane; a new CLI in the same pane posts nothing for as long as the last one ran | 033 Q |
| A3 | An account whose folder is gone is skipped at hook install without being named anywhere | 031 |
| A4 | The commit message box is one string for every project; a draft finishing in A overwrites what was typed in B | 033 N |
| A5 | A note whose line changed is sent to the agent without the line saying so, when the diff was parsed per file | 033 M |
| A6 | Reopening the find bar shows matches from before the buffer moved | 033 I |
| A7 | In the find bar Enter never steps (the field gives up focus on Enter), a conversion's Enter would step once it did, Esc leaves focus on a field that is gone, and the close icon is a raw button | 024 |
| A8 | The empty projects page types `⌘O` into its hint, which goes wrong when `project.add` is rebound | 028 |
| A9 | A long diff note paints over the lines below it, and 保存 sits under the next line's click area | 033 K |
| A10 | The note editor takes focus back every frame | 033 L |
| A11 | Opening in an external editor and copying a checkpoint's diff run on the frame, up to ten seconds, with every error dropped | 110 |
| A12 | Terminal links are re-tokenised for every visible row every frame, and the staged-file selection is rebuilt every frame | 033 F, O |
| A13 | Two tests of 077 assert nothing; 049's hidden count has no test, and one of its tests builds a session it never reads | 049, 077 |
| A14 | `is_safe_ref_name` accepts names `git check-ref-format` refuses (`a~1`, `a@{1}`, control characters), which are read as revisions by the land; the checkpoint SHA check accepts an empty string | 111 |
| A15 | An approved setup or run script is run by path, so the file can change between the check and bash reading it | 019, 111, 033 P |
| A16 | Running the run script again starts a second copy instead of stopping the first | 110 |
| A17 | The templates and run-script reads happen on the frame a menu opens, and every two seconds while it stays open | 110, 111 |
| A18 | Codex has no merge gate: a Codex session can merge into main before CI has passed | 040 |
| A19 | The test build carries two warnings (a loop name no assertion uses, a `mut` nothing mutates), and nothing compiles the tests with warnings denied | found while building 202 |

Each later change names the rows it closes.
