# Intent: a project or a session cannot be added from the session page's column, and its project names read no larger than its rows

- **Status**: approved
- **Opened**: 2026-11-25

## Problem

The person's request, verbatim (2026-11-25):

> セッションページ内でもプロジェクトを簡単に生成できるようにしたい。例えば、左サイドバーに追加ボタンや右クリックでセッション生成やプロジェクトの生成など。そしてプロジェクトというよりワークスペースという言い方がよいかもセッションページの左のサイドバーのワークスペースの文字を少し大きくし、他のUI/UXにも一貫性のあるデザインのUI/UXにして

They were asked two questions — how far to take the rename to
「ワークスペース」, and which ways to add — and did not answer within ten
minutes. The recommended answers were adopted and recorded in `state.yaml`
`resume`:

- The rename runs across the whole application, so one thing has one name,
  as a separate change (320). This change keeps the word 「プロジェクト」.
- The ways to add are the two the request names — a + (beside the column's
  heading) and a right-click (on the column's empty space) — and the
  right-click on a project's heading, which today does nothing, offers the
  same creation: a session in that project, or a project.

Today the session page's left column lists projects and their sessions, but
the only thing it can add is a session in a project that is already there,
through the small + on that project's heading. Adding a project means leaving
for the projects page, and once it is added the window stays there: the
person has to come back to the session page and find the new project, folded,
in the column. A right-click on the column does nothing except on a session
row.

The project names in the column are 12px in the muted ink, the same size as
the session titles under them and fainter than them, so the column reads as
one flat list rather than projects holding sessions. The column's own heading
「プロジェクト」 is 11.5px, smaller than everything it heads.

## Who feels it, and when

- A person working on the session page who wants to start on another folder:
  every time, they go to the projects page, add it, come back, find it and
  unfold it.
- A person scanning a column of several unfolded projects for the one they
  want: the names do not stand out from the session titles.

## Desired outcome

- From the session page's column, without leaving the session page, a person
  starts a new session or adds a project: from a + beside the column's
  heading, or by right-clicking the column's empty space. A project added this
  way appears in the column already unfolded and is the workspace's project,
  and the window is still on the session page.
- Right-clicking a project's heading offers starting a session in that
  project, and adding a project.
- 「ワークスペースの文字」 is read as both texts that name projects in the
  column: each project's name, and the column's heading 「プロジェクト」. Both
  are larger than the session titles under them, and the names are drawn in
  the plain ink rather than the faint one (session titles are already bold,
  so the names win on size and ink, not weight). The controls beside the
  heading look like the controls beside other headings in the app.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI: no file-system call in
  a draw path.
- User-facing text is Japanese, with rows in every language table.
- Sizes come from the type scale and buttons from the helpers in `DESIGN.md`.
- No persisted shape changes.

## Systems likely affected

`src/app/screens.rs` (the column), `src/app/folder_picker.rs` (where an added
folder goes), a new child of `src/app.rs` for the column's add menu,
`src/i18n_tables.rs`, and the feature documents for `session-column` and
`project-list`.

## Open questions

None left open; the two unanswered ones took the defaults above.

## Not in scope

- Renaming 「プロジェクト」 to 「ワークスペース」 anywhere (change 320).
- Removing a project from the column; it stays on the projects page.
- Making a new folder from the column beyond what the folder picker already
  does (change 317).
