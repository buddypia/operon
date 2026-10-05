# The twelve Important findings this change did not take

From the stage-5 review of changes 018–026 by `rust-reviewer` ×3,
`subprocess-safety-reviewer`, and `durability-reviewer`, 2026-09-06. All five
returned *do not approve*. The four that lose work a person did are fixed in
`./report.md`; these twelve are open.

Ordered by what they cost, not by the change they came from.

## Wrong result

| Finding | Where | What happens |
|---|---|---|
| **H** | the click handler in `src/app.rs`, against `worktree_root` in `src/git.rs` | **Change 026 does not work in a worktree session.** A worktree lives at `<project parent>/<project name>-worktrees/<branch>`, outside `project.path`, and the open path strips against `project.path`. `resolve_terminal_path` requires the file to be inside the session's worktree, so for a worktree session the two conditions are mutually exclusive: the path underlines and the click always ends at 「このプロジェクトの外にあります」. Running agents in parallel worktrees is what this application is for, so this is 026 not working in its own main case. **Needs a decision**: whether a document may be rooted at a worktree, or whether such a path simply stops underlining. |
| **G** | `resolved_paths` in `src/app.rs` | Keyed by the path string alone. Session A caches `src/main.rs` → A's worktree; in session B the same text hits the cache, underlines, and opens A's file. B's file is unreachable. Worse, the frame after a session switch resolves the previous frame's candidates against the new worktree and writes those wrong answers in permanently. **Needs a decision** only on the key: `(Uuid, String)`, or cleared on session switch. |
| **N** | `commit_message_input` in `src/app.rs` | The only field in the commit box that is not per-project, while `git_staged_files` and `upstream_cache` beside it are. Write a message in project A, switch to B, press commit: A's description is recorded against B's changes. The `CommitMessageDrafted` handler carries a `project_id` and does not use it, so a draft started in A overwrites what was typed in B. |
| **M** | `send_diff_comments` in `src/app.rs` | Staleness is read from the `(project, None)` diff cache, which the editor's review pane never fills — it parses under `(project, Some(path))`. The pane shows 「行が変わっています」 and the message sent to the agent omits the line saying so, which is what spec item 5 of change 020 exists to prevent. |
| **I** | the find bar in `src/app.rs` | Reopening ⌘F does not recompute the matches, and `capture-pane -S -5000` is anchored at the end, so every new line shifts every index. Search, close, let the agent print 2000 lines, reopen: the count is stale and the wash lands on rows that do not contain the query. |
| **D** | the statusLine slot in `src/tmux/hooks.rs` | The install marker is never cleared and is written before the settings file, with its failure swallowed. Turn the hook setting off and on again and the usage reader never returns, with nothing said; the only recovery is deleting an undocumented file. A crash between the marker write and the settings write leaves the same state. The marker is also written when the slot was already someone else's, so it records "tried once" while its comment says "installed once". |
| **Q** | the throttle in `src/tmux/hooks.rs` | `now` is the payload's `total_duration_ms`, monotonic only within one CLI session; the stamp is keyed on the tmux session, which outlives it. After `/clear` the difference is negative, the interval test is always true, and that pane never posts again. No recovery path, because the stamp is only rewritten when a post happens. |

## Work in the draw path

| Finding | Where | What happens |
|---|---|---|
| **F** | `url_end` and `terminal_row_links` in `src/ui/terminal.rs` | Two `Vec<char>` allocated per character position, before the length check, and `terminal_targets` re-tokenises every visible row every frame with no cache. Measured by the reviewer at **496µs per frame for 50 rows, 99µs with the two schemes hoisted to constants** — 5× for a change of two lines. It lands at pointer-move rate exactly while the mouse is over the pane, which is when links matter. |
| **K** | the note row in `src/ui/diff.rs` | `diff_row_offsets` reserves `5 × cell.y` for a note whose body may be 2000 characters, and `Ui::new_child` does not clip. A note of a few hundred Japanese characters overflows and the diff lines below are painted over it. While editing, the 保存 button sits under the next line's interact rect — registered later, so it wins — and pressing it opens a note on the following line instead of saving. This is the first note a person writes. |
| **L** | the note editor in `src/ui/diff.rs` | `request_focus()` runs on every frame the note is open, and egui's `request_focus` overwrites the focused widget outright. Click the editor's ファイルを絞り込む box and type: every keystroke lands in the note draft instead. The only exits are 保存/削除 or scrolling the note off screen. |
| **O** | `ensure_staged_selection` in `src/app.rs` | A `HashSet<String>` rebuilt from every changed path every frame, and `staged_selection(…).len()` builds and sorts a `Vec<String>` only to count it — the exact shape `.claude/rules/rust.md` names. `git status --porcelain` has no item ceiling, so a 2000-file change is about 4000 allocations plus a sort per frame. |

## A claim wider than what is enforced

| Finding | Where | What happens |
|---|---|---|
| **P** | change 019's commit message, against the run path in `src/app.rs` | The message says "a file swapped between being shown and being run is not the file that was approved". What is actually closed is shown-to-checked. Between the digest comparison and `bash` reading the file sit two fsyncs, a thread spawn, and three tmux processes — hundreds of milliseconds. An agent already running in that worktree can replace the script in the window, and it runs as approved. Copying the approved bytes to a private temporary file, or feeding them to `bash -s` on stdin, closes it. |

## What the reviewers checked and found correct

Worth keeping, because it says where the guards already hold: no force push in
any spelling (`no_source_file_can_force_a_push` scans all of `src/`); no new
`unwrap`/`expect` (production is still five); every new `Command::new` routed
through the budgeted wrappers, with the only raw `.output()` in `src/tests.rs`;
no byte-boundary slicing anywhere in 024–026, since every position is computed
over `chars()`; `diff_visible_rows` correct at every edge; both sidecars written
through `write_file_atomically`; change 023's polling interval honoured on every
failure path; `src/main.rs` untouched by all five commits, so the `PATH` setup
and the process lock are unchanged.
