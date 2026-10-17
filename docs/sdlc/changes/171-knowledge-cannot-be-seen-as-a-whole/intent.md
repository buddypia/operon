# Intent: a project's knowledge can be read one entry at a time, never seen as a whole

- **Status**: draft
- **Opened**: 2026-10-17

## Problem

The person asked (2026-10-16), in the request change 157 answered in part:

> 各Claude Code/Codex CLI/Antigravity CLIなどのセッションの会話から重要なものを判定AIなどしてEmbeddingGemma2を利用してナレッジ基盤としてプロジェクト別に利用するようにオプトイン機能を作りたい。セッション検索も素早くできるし、プロジェクト内でナレッジ鮮度や参照回数などで自動的に消えたり増えたりするようにしたい。そして2Dなどでナレッジ同士が可視化されたのをみたい。可視化に関してはこのプロジェクトを参照(~/dev/buddypia/personal-knowledge)

and decided that the work is split in two: change 157 collects the knowledge,
and a later change draws it in 2D. 157 has landed. This is the second half; the
person asked for it to proceed on 2026-10-17 (「残タスクを進めて」).

Today a project's knowledge is a list. It can be sorted by freshness, by use, or
by age, and searched, but nothing shows which entries are about the same thing,
which topics the project has accumulated most knowledge on, where a topic has
only one stale entry left, or how a newly added entry relates to what was
already there. With hundreds of entries the list answers "find X" and nothing
else.

## Who feels it, and when

- Someone returning to a project after weeks, wanting to see what it has learned
  — the main topics and how alive each still is — before opening a conversation.
- Someone about to start work on one area ("restore", "tmux"), wanting every
  related decision, cause, and pitfall together, including entries whose words
  do not match a search.
- Someone deciding what to pin or delete, who can see that ten entries repeat
  one point, or that a topic is fading as a whole.

## Desired outcome

- In a project with knowledge, the person can switch from the list to a 2D map
  in which every entry is a point, and entries with similar meaning sit close
  together.
- Groups of related entries are visible as groups, each with a short name the
  person can read.
- Each point shows, at a glance, its kind, how fresh it is, and how often it was
  used; a pinned entry is recognisable.
- Pointing at a point shows its title; choosing one shows the same detail the
  list shows (body, sources, open the conversation, pin, delete), and the list
  and the map stay in agreement about what was chosen.
- A search highlights the matching points on the map.
- The map stays readable and responsive for the largest knowledge base a project
  can hold, and opening it does not freeze the window.
- Placement does not jump around between visits: the same knowledge appears in
  the same place, and new entries appear near their neighbours.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls. The layout is computed
  on the Mac from the vectors 157 already stores; nothing new leaves the machine.
- User-facing text is Japanese; code, comments, and docs are English.
- Knowledge stays in the app's own data directory, never in the working tree.
- Change 157's behaviour — collection, freshness, search, the list — is unchanged.

## Systems likely affected

`src/knowledge.rs` (reading entries and vectors, computing a layout),
`src/app/knowledge.rs` (the tab, a background task), `src/ui/` (drawing the map),
`src/config.rs` (layout constants), `src/i18n_tables.rs`, and possibly
`Cargo.toml` if a projection crate is chosen over writing one.

## Open questions

- How the reference project lays out and groups its knowledge, and which of that
  fits ≤2 000 entries of 768 dimensions in-process — answered by reading
  `~/dev/buddypia/personal-knowledge` and measuring.
- How the map is arranged on the tab — answered by the person at the screen.

## Not in scope

- A 3D view, or a map spanning several projects.
- Editing an entry's text, or merging entries, from the map.
- Links an agent or the person draws by hand; relations come from meaning only.
- Exporting the map as an image.
