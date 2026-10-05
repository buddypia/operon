# Spec: a document carries the root it is relative to

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. An open document knows which directory its path is relative to. For a file
   browsed from the tree that is the project; for a file followed from a
   terminal link it is that session's worktree.
2. Opening, saving, and the external-change check all use the document's own
   root. No call site pairs a document's path with a project's path by hand.
3. Two documents with the same relative path under different roots are two
   documents, not one. Today `active_document` is keyed by
   `(project, path)`, so the project's `src/app.rs` and a worktree's would be
   the same tab.
4. A path that resolves outside both the project and the session's worktree is
   still refused, with the message it has today. There is no root to give it.
5. A tab whose document is not rooted at the project says which branch it is
   from.
6. Nothing about a project-rooted document changes: same tab, same label, same
   paths.
7. The badge and "is this the project's own file" are decided once, when the
   document is opened, from a canonical pair. The tab row draws every frame, and
   a root comparison re-derived there against whatever spelling the project is
   stored under would label every one of the project's own documents as coming
   from somewhere else.
8. A document that is not rooted at the project is not offered the diff view.
   The diff is read from the project's working tree and cached under
   `(project, relative path)`, so drawing one for a worktree document would show
   the project's copy of the same path — and the two would share one cache
   entry, along with the notes change 020 keys the same way.

## Behaviour

Clicking a path with a line number in a worktree session opens that worktree's
file at that line, as following the link has always claimed it would.

The tab row gains a branch name after the file name, in the muted text colour,
only for a document that is not rooted at the project:

```
┌─────────┐┌─────────────────────┐
│ ◦ app.rs ││ ◦ app.rs  fix/login │
└─────────┘└─────────────────────┘
```

The badge is the branch, not the path: the path is long, changes nothing a
person is deciding, and would push the file name out of view. Where the branch
cannot be read, the worktree folder's own name is used, which is derived from
the branch anyway.

Not-happy states:

- **The worktree is gone when the file is saved.** The save fails and says so,
  the same way it does for any file that has moved. The document stays open with
  its buffer intact.
- **The same file followed twice from two sessions on the same worktree.** One
  document; the root and the path are equal, so it is the same file.
- **A path outside every root.** Refused, unchanged from today.

## Design

`OpenDocument` gains `root: PathBuf`, and the join of a root with a path goes
through one accessor rather than being written at the call site:

```rust
impl OpenDocument {
    pub(crate) fn absolute(&self) -> PathBuf { self.root.join(&self.path) }
}
```

The four sites that pair them today all already take a root as an argument and
are always handed `project.path`:

| Site | Becomes |
|---|---|
| the editor open | reads the document's root |
| the editor save | reads the document's root |
| the external-change check | reads the document's root |
| the review pane's full path | reads the document's root |

`active_document` and `document_index` key on `(project, root, path)`, which is
requirement 3.

`open_document` takes the root as an argument. `open_terminal_target` chooses
it: the project when the resolved file is inside it, the session's worktree when
it is inside that, and a refusal otherwise — which is requirement 4 and keeps
the existing message for the case that has no answer.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The badge uses `palette.text_muted`, an existing role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The badge is a branch name, not a mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Not a string but the same rule: the root and the path must agree everywhere, so they are joined in one method and a test forbids any other site from `join`ing them, on one line or split across several by rustfmt. What that test cannot see is stated with it under **Flagged concerns**, because the review of this change found a real pairing it misses. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No persisted shape changes; `OpenDocument` is in-memory. The durability question here is the save path, and it is requirement 2: a save into the wrong copy is the failure this change must not introduce. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child. |
| Documentation — user-facing docs change in all three languages together | Yes | The 026 bullet in all three READMEs currently claims something that is false for a worktree session; it is corrected rather than extended. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None of the above. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan. The editor's existing `EDITOR_FILE_MAX_BYTES` applies unchanged, since the read is the same call with a different root. |

## Flagged concerns

- **A save into the wrong copy.** The one way this change could be worse than
  the defect it fixes. Requirement 2 is structural rather than careful:
  `absolute()` is where a root meets a path, and a test scans the source for any
  other site `join`ing a project path to a document path, whitespace-collapsed
  so a chain rustfmt split across four lines is still one join.

  **What that test does not cover, stated because the review of this change
  found it the hard way.** A root taken into a variable first, and a root and a
  path handed to the same function without a `join` at all. The second is not
  hypothetical: `request_git_diff` takes the project's path as the git root and
  the document's path as the file, which is how the diff view came to show the
  project's copy of a worktree document. The test did not see it and could not
  have; a reviewer did. Requirement 8 is the answer to that one, and the honest
  position is that this guard narrows the failure rather than closing it.


## Acceptance

- `cargo test --locked` passes, including:
  - `a_document_opens_and_saves_under_the_root_it_was_opened_from`
  - `the_same_relative_path_under_two_roots_is_two_documents`
  - `a_path_outside_every_root_is_still_refused`
  - `no_source_file_joins_a_project_path_to_a_document_path`
  - `a_tab_says_which_branch_a_document_that_is_not_the_projects_came_from`
  - `a_document_rooted_outside_the_project_is_not_offered_a_diff`
- In the running app: start a session in a worktree, make it print a path, click
  it, confirm the editor opens that worktree's file with the branch on the tab,
  edit and save, and confirm the project's copy is untouched.

## Rejected alternatives

- **A boolean "force" flag on the refusal.** The refusal is not a permission
  decision. `strip_prefix` failing means there is no relative path to open with,
  so forcing past it has nothing to force — it would open a name the editor
  resolves somewhere else, which is the save-into-the-wrong-file failure.
- **Stop underlining what the editor cannot open.** Keeps 026's promise true and
  removes the feature from exactly the sessions it was built for.
- **Add each worktree as its own project.** Works today, and multiplies the
  project list by the number of branches.
- **Show the full path on the tab instead of the branch.** Long, and pushes the
  file name out of view for the answer to a question nobody is asking.
