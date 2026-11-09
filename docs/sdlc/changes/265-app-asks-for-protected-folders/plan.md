# Plan: macOS asks for the person's folders in Operon's name

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-09
- **Status**: approved

## Root cause

macOS asks for Desktop, Documents, Downloads and volumes on behalf of the
*responsible* process, and every process under the tmux server Operon started
is Operon's. The prompts the person saw came from an agent's `find` (bug report
1), and the dialog could not say so because the bundle has no folder usage
descriptions. That is the observed cause. Fixes 2 and 3 are precautionary, for
the same class in Operon's own binary, which the log did not show firing: the
terminal link probe stats any absolute or `~` path in agent output, and the Codex
and Antigravity scans canonicalize paths read out of those products' records, so
either can look inside a protected folder nobody pointed Operon at.

## Fix

1. **The dialog says where the request comes from.** `scripts/package-macos.sh`
   writes one reason, spelled once, into `NSDesktopFolderUsageDescription`,
   `NSDocumentsFolderUsageDescription`, `NSDownloadsFolderUsageDescription`,
   `NSRemovableVolumesUsageDescription` and `NSNetworkVolumesUsageDescription`:
   the request usually comes from an agent or command running in Operon's
   terminal, choose 「許可しない」 if it is unexpected, and Operon itself is
   unaffected unless a project lives there. The person approved this edit to the
   bundle script (approvals.log).
2. **The terminal link probe never touches a protected folder unbidden.**
   `resolve_candidate_path` normalizes the candidate lexically (`.`, `..`, `~`)
   and, before `canonicalize`, returns `None` when it lies under the home
   folder's `Desktop`, `Documents` or `Downloads`, or under `/Volumes` — the
   folders the five keys of fix 1 name — compared case-insensitively, as APFS
   resolves them, unless it is inside the folder it is being resolved against
   (that folder as given or canonicalized: the session's worktree or project).
   Every other path resolves as today, so change 087's outside-project links
   keep working. The folder names live in `src/config.rs`.
3. **Another product's record is compared as spelled.** The Codex `cwd`, the
   Antigravity history `workspace`, and the Antigravity log workspaces are
   matched against `project_and_worktree_paths` as written, never canonicalized.
   Projects are canonicalized when registered (`src/app.rs` `canonicalize` on
   add) and `project_and_worktree_paths` canonicalizes again; Operon starts
   agents with `cd` into that path; the records on this machine spell the same
   path (`"cwd":"/Users/a13973/dev/buddypia/learn_korean"`,
   `"workspace":"/Users/a13973/dev/buddypia/design-lookbook"`).
4. **The dangerous-launch warning names the reach.** 「警告: 確認もサンドボックスも
   なしにファイルを変更できます。」 becomes 「警告: 確認もサンドボックスもなしに、
   プロジェクトの外（書類・ダウンロード・デスクトップなど）のファイルも読み書き
   できます。」, with its English and Korean rows.

## Files that change

| File | Change |
|---|---|
| `scripts/package-macos.sh` | five usage-description keys from one `folder_access_reason` |
| `src/config.rs` | `PROTECTED_HOME_FOLDERS`, `PROTECTED_VOLUMES_ROOT` |
| `src/ui/terminal.rs` | lexical normalization and the protected-folder refusal inside `resolve_candidate_path` (through `resolve_candidate_path_under`, which takes the home folder so a test can supply one) |
| `src/cli.rs` | the three canonicalize calls on record paths become plain lookups |
| `src/app/screens.rs` | the warning string |
| `src/i18n_tables.rs` | the warning's English and Korean rows |
| `src/tests.rs` | the tests below; `test_terminal_resolves_tilde_paths_and_cli_tool_calls` (change 087) keeps its outside-project assertion |
| `docs/sdlc/changes/265-…/` | state, this plan, screen, approvals, verification, mutations |

## Order of work

1. Add `resolve_candidate_path_under(base_dir, path, home)` as a pass-through
   of today's body, so the tests below compile and fail on their assertions.
   Then write them and watch each fail on the unedited behaviour:
   - `protected_folders_265_terminal_link_probe_skips_protected_folders` — with
     a temporary home holding `Downloads/x.pptx`, `Documents/a.txt`,
     `Desktop/b.txt` and `other/x.txt`, the absolute, `~`, lower-case and `..`
     spellings of the first three resolve to `None` while the files exist;
     `other/x.txt` still resolves. `Downloads/link.txt`, a symlink to
     `other/x.txt`, also resolves to `None`: the refusal reads the path as
     written, before `canonicalize` — a guard moved after it would see
     `other/x.txt` and pass it.
   - `protected_folders_265_project_inside_a_protected_folder_still_resolves` —
     a project at `<home>/Documents/proj` resolves its own source file, and not
     `<home>/Documents/other.txt`.
   - `protected_folders_265_record_paths_are_compared_as_spelled` — a symlink to
     the project as Codex `cwd`, Antigravity history `workspace`, and log
     workspace is not attributed to the project; the project's own spelling is.
   - `protected_folders_265_bundle_explains_folder_prompts` — the package script
     inserts all five keys from one reason that names the terminal, 「許可しない」,
     and no other spelling of the reason.
   - `protected_folders_265_dangerous_launch_warning_names_personal_folders` —
     the footer's warning string names 書類・ダウンロード・デスクトップ and has a row
     in every table.
2. Add the constants, the normalization and the refusal; the terminal tests pass
   and the existing resolver tests stay green.
3. Replace the three canonicalize calls; the record test passes and the Codex
   and Antigravity scan tests stay green.
4. The script keys and the warning string with its rows.
5. `make q.fast`, the named tests, the existing resolver and scan tests, the
   i18n table test; `mutations.py`; `empirical-verifier`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| An Antigravity record spelled through a symlink (Go's `os.Getwd` may return `$PWD`) no longer matches | a conversation started in a symlinked folder outside Operon is not listed | accepted: Operon starts agents in the canonical project path; recorded as a departure if a scan test needs the old match |
| A Codex or Antigravity record spelled through a symlink to a registered project (the person `cd`s through a link and starts the CLI outside Operon) | that conversation is not listed for the project | accepted: projects are canonical when registered and the records on this machine spell them that way; `protected_folders_265_record_paths_are_compared_as_spelled` pins the choice |
| A project reached through a symlink inside a protected folder (`~/Documents/link` → `~/dev/proj`) | its files named through the link in output are plain text | accepted: the exemption is the session folder as given or canonical, and registered folders are canonical, so the link spelling is neither |
| A test fixture records the `/var` spelling of a `/private/var` temporary project | an existing scan test goes red | step 3 runs the Codex and Antigravity scan tests; the fix is the fixture's spelling, not a canonicalize |
| A symlink outside the protected folders that points into one | the probe follows it and macOS asks | accepted residual: only a link the person or a repository made; written in the plan rather than claimed away |
| A protected-folder path in output was clickable and now is not | `~/Downloads/x.pdf` in a terminal line is plain text | intended; the change 087 test keeps an outside-project path under the real home that is not protected |
| The usage description claims more than Operon does | review reads the string against this plan | the claim is hedged to what the TCC log shows ("多くの場合") and to what fix 2 and 3 guarantee |

## Proof of completion

- `cargo fmt --check` — no output; `cargo clippy --locked -- -D warnings` — clean.
- `cargo test --locked protected_folders_265` — 5 passed.
- `cargo test --locked refuses_a_path_that_escapes_the_session_folder`,
  `test_resolve_terminal_session_path_precedence_and_fallback`,
  `test_terminal_resolves_tilde_paths_and_cli_tool_calls` (change 087), `every_message_the_code_shows_has_a_row_in_every_table`, and the
  Codex / Antigravity scan tests — passed.
- `python3 docs/sdlc/changes/265-app-asks-for-protected-folders/mutations.py` —
  every mutation caught.
- After the swap: a folder prompt attributed to Operon shows the reason.

## Departures from the plan

- **The registered folders carry both spellings.** Fix 3 compares record paths
  as written, and the scan fixtures write the `/var/folders` spelling of a
  temporary project whose canonical form is `/private/var/folders` — the same
  logical/physical pair a real record could carry. `project_and_worktree_paths`
  now holds each project and worktree path as given (by Operon or by git) and
  canonicalized, instead of canonicalized only. Only the registered folders are
  resolved; the record's path still never is. The Codex, Antigravity and
  restore tests stay green (66, then 420 across the terminal, launch, session,
  cli, worktree, restore and path filters).
- **The data volume's spelling of the home folder** (empirical verifier, F1).
  `/System/Volumes/Data/Users/<me>/Desktop` reaches the same Desktop through
  the firmlink, and the refusal compared only `<home>/Desktop`. The predicate
  now drops a leading `SYSTEM_DATA_VOLUME_ROOT` before comparing.
  `protected_folders_265_data_volume_spelling_is_the_same_folder` failed at
  3b50349 and passes; one mutation watches it. The verifier itself could not
  check out the branch in its isolated worktree (the trunk-allowlist hook
  refused `git checkout --detach`), so the lead came from its reading and the
  reproduction was run here.
