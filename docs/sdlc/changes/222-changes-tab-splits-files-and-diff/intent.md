# Intent: clicking a changed file shows its diff beside the file list and commit box

- **Date**: 2026-10-24
- **Route**: `modify`
- **Risk tier**: `low` (`autonomy: full`, `screen: yes`)
- **Status**: approved

## Problem

Under a project's 「変更」 tab (`src/app/screens.rs:3036`), the changed file list, the stage checkboxes, and the commit box live on `GitView::Changes` while the working-tree diff lives on `GitView::Diff`. Clicking a file in `GitView::Changes` sets `self.git_view = GitView::Diff` (`src/app/screens.rs:3122`), replacing the file list and the commit box with the diff pane. To inspect a second file, tick a stage checkbox after reading its diff, or write the commit message while looking at the diff, a person has to switch back and forth between the two sub-tabs on every file.

## Desired outcome

- Opening a project's 「変更」 tab on `GitView::Changes` shows the changed file list, stage checkboxes, commit box, and push row in a left column and the working-tree diff pane (with its file filter, AI review button, inline comment gutter, and comment footer) in a right column beside it.
- Clicking a file row in `GitView::Changes` selects that file for the diff pane (`self.git_selected_file = Some(path)`) while keeping `self.git_view` on `GitView::Changes`, so the file list, stage checkboxes, and commit box stay on screen beside the selected file's diff.
- Clicking the refresh button on `GitView::Changes` refreshes both the changed file list and the diff shown beside it.

## Non-goals

- Removing `GitView::Diff` or `GitView::History`: `GitView::Diff` remains available as a full-width diff view, and `GitView::History` remains the commit history view.
- Changing how `diff_pane`, inline diff comments, AI diff review, or `ui_commit_box` work internally.
