# Intent: what a project learned in one agent conversation is lost to every later one

- **Status**: approved
- **Opened**: 2026-10-16

## Problem

The person asked (2026-10-16):

> 各Claude Code/Codex CLI/Antigravity CLIなどのセッションの会話から重要なものを判定AIなどしてEmbeddingGemma2を利用してナレッジ基盤としてプロジェクト別に利用するようにオプトイン機能を作りたい。セッション検索も素早くできるし、プロジェクト内でナレッジ鮮度や参照回数などで自動的に消えたり増えたりするようにしたい。そして2Dなどでナレッジ同士が可視化されたのをみたい。可視化に関してはこのプロジェクトを参照(~/dev/buddypia/personal-knowledge)

and then decided (2026-10-16): Operon is to be distributed as open source, the
embeddings must come from EmbeddingGemma 2, importance is judged in two steps
(a local score first, then an agent CLI for the few that pass), the knowledge
is usable both inside Operon and by the agents working in the project, it is
stored privately on the machine rather than in the repository, and the work is
split in two — this change is the knowledge itself, a later one draws it in 2D.

Today a decision, a root cause, or a working procedure reached in a Claude Code,
Codex, or Antigravity conversation lives only inside that conversation's
transcript. The next conversation in the same project starts without it, and
the person finds it again only by remembering which conversation it was in and
scrolling. The local history search reads raw transcript files on every query,
matches words rather than meaning, and caps how much it reads, so a conversation
from a few weeks ago is slow to find or not found.

The Settings screen already offers 「内蔵 EmbeddingGemma 2」, but nothing behind
it is real: the download button writes a marker file claiming 248MB without
fetching anything, and the "semantic" ranking is word overlap with a hard-coded
synonym list (change 142). A person who turned it on believes they have semantic
search and does not.

## Who feels it, and when

- Someone opening a new agent session in a project where, last week, another
  session found why a test was flaky; the new agent rediscovers it from scratch,
  or repeats the mistake.
- Someone who remembers "we decided how retries work" but not which of forty
  conversations, across three CLIs, said so.
- Someone who enabled 「内蔵 EmbeddingGemma 2」 and gets the same results as
  keyword search, with a 「Metal GPU」 label claiming otherwise.

## Desired outcome

- A person turns knowledge collection on for one project; nothing is collected
  for a project they did not turn it on for, and nothing is downloaded until
  they ask.
- After their next conversations in that project, the important parts — a
  decision and its reason, a cause and its fix, a procedure, a pitfall — appear
  as short knowledge entries, each linked back to the conversation it came from.
  Small talk and dead ends do not.
- Searching by meaning ("why does restore drop the first turn") finds the entry
  and its conversation in well under a second, whichever CLI held it and however
  old it is.
- An agent working in that project can ask Operon for the project's knowledge on
  a topic and gets the relevant entries.
- Entries nobody uses fade and are eventually removed; entries that are looked
  up, or that come up again in new conversations, grow stronger and stay. The
  person can see each entry's freshness and how often it was used, and can keep
  or delete one by hand.
- The embedding model is EmbeddingGemma 2, running on the person's Mac, with
  results that match the reference implementation.
- 「内蔵 EmbeddingGemma 2」 no longer claims something that is not happening.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Open source: a person who builds or installs Operon must not need a separate
  tool (such as the `eg2` server) for the feature to work.
- Local-first: no telemetry, no accounts. The embedding model runs locally; its
  weights are fetched only when the person opts in. The second judging step uses
  the agent CLI the person already uses for that project, which already sees the
  conversation it is judging.
- User-facing text is Japanese; code, comments, and docs are English.
- Conversations can contain secrets, so the knowledge stays in the app's own data
  directory, never in the project's working tree.
- The persisted store is on real machines — a shape change needs a migration and
  a `STORE_SCHEMA_VERSION` bump.

## Systems likely affected

`src/cli.rs` and `src/history.rs` (reading conversations), `src/agents.rs`
(the judging command), `src/store.rs` / `src/models.rs` (the opt-in flag),
`src/app.rs` / `src/app/screens.rs` (settings, search), `src/main.rs` (a
command-line entry for agents), `Cargo.toml` (an inference runtime), and a new
module for the knowledge base itself.

## Open questions

- Which inference runtime runs EmbeddingGemma 2 in-process — answered by
  measurement against the reference server and by the person, because it is a
  new dependency.

## Not in scope

- The 2D map of knowledge entries (the next change, modelled on
  `~/dev/buddypia/personal-knowledge`).
- Sharing knowledge between machines or with a team.
- Image or audio embeddings.
- Injecting knowledge into an agent's context automatically at session start;
  the agent asks for it.
