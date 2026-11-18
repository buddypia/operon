# Spec: the session search finds what the row shows, and the history search says "none" only after it looked

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **The search reads what the row shows.** With text in 「セッションを検索」, a
   session is kept when the text, trimmed and compared without regard to letter
   case, is contained in any of: the title the list gives it (for a session
   launched without a name or a request, the first non-empty line of its first
   prompt), its name, its request, its agent's name as the screen writes it
   (「Codex CLI」, 「Claude Code」, 「Antigravity」) or its stored agent word, its
   project's name, or its branch. A session none of these contain is hidden.
   Empty or all-space text keeps every session. Test:
   `search_310_finds_a_session_by_what_its_row_shows`.
2. **The list and the grid give one answer.** 「プロジェクト別」 and 「グリッド」
   decide through the same function with the same trimmed text, so a trailing
   space hides nothing in either. Test:
   `search_310_the_list_and_the_grid_keep_the_same_sessions`, which draws both
   views.
3. **「一致する履歴はありませんでした。」 is the answer to a finished search.**
   It is drawn only when the last finished 「ローカル履歴」 search was for the
   text now in the field (trimmed), no history search is running, and there are
   no matches of either kind. With text nobody has searched for, or text edited
   since the search, the sentence is not drawn. Test:
   `search_310_history_says_none_only_after_a_finished_search`.
4. **A running history search says so in the panel.** While the scan or the
   indexed search runs and the field is not empty (trimmed), the panel draws
   「履歴を検索しています…」 in place of the "none" sentence. With the field
   emptied mid-search, nothing is drawn there. Test:
   `search_310_history_says_it_is_searching_while_it_runs`.

## Behaviour

- **「セッションを検索」**, list view: typing 「ログイン」 keeps a row titled
  「ログイン画面の修正」 whether that title is its name, its request, or the first
  line typed into an unnamed session; typing a project's name keeps that
  project's sessions; typing 「antigravity」 keeps Antigravity sessions; typing a
  branch keeps the sessions on it. Rows that do not match are hidden as today,
  and a project left with none reads 「一致するセッションはありません。」 as
  today.
- **Grid view**: the same sessions are kept as in the list. A grid card
  writes the session's own title, not the first prompt line the list uses for
  an unnamed session, so a card can be kept by words only the list shows; one
  answer in both views is the requirement.
- **「ローカル履歴」**:
  - *Empty field, or text not yet searched*: nothing under the field (before:
    「一致する履歴はありませんでした。」 from the first character).
  - *Searching, field emptied*: nothing under the field.
  - *Searching*: 「履歴を検索しています…」 in the weak text style, where the
    "none" sentence goes. The corner notice 「“…” をローカル履歴から検索して
    います…」 stays as today.
  - *Finished, nothing found*: 「一致する履歴はありませんでした。」 and, when the
    meaning search is off, the 💡 hint and its link — as today.
  - *Finished, matches*: the lists as today.
  - *Text edited after a finished search*: the "none" sentence goes away until
    the next search finishes. Matches from the earlier query stay, as today.
  - *Interrupted*: quitting mid-search loses nothing; nothing here is saved.
    *Error / too large*: a scan cut short by its limits still finishes and
    reports through the notice as today; with no matches the panel reads "none".
    An indexed search that fails reports through the notice as today, and the
    panel then reads what the scan found — "none" if that was nothing.
- No failure path is added: nothing here can fail to the person.

Strings a person reads, with their catalog rows:

| Japanese | English | Korean |
|---|---|---|
| 履歴を検索しています… | Searching history… | 기록을 검색하는 중… |

## Design

Placement verdict: **EXTEND**. Evidence: `git grep -n "session.name, session.goal, session.agent" src/`
lists the two filters in `src/app/screens.rs` (`ui_session_grid`,
`ui_terminal_session_tabs`), and `git grep -n "一致する履歴はありませんでした" src/app`
the one draw site in `ui_sessions`. The command palette's own match
(`palette_entries`) already covers title, project and branch and is not touched.

- **`src/app/screens.rs`.**
  - New `OperonApp::session_matches_search(&self, session: &Session, needle: &str) -> bool`.
    `needle` is already trimmed and lower-cased; empty returns `true`. It checks,
    short-circuiting, `session.name`, `session.goal`, `session.agent`,
    `agent_choice_copy(&session.agent).0`, `session.branch`, the name of the
    project whose id is `session.project_id`, and last
    `session_list_title(session, first prompt from self.session_prompt_turns)`.
    Each field is lower-cased only while a search is active.
  - `ui_session_grid` and `ui_terminal_session_tabs` both take
    `self.search.trim().to_lowercase()` and call it; the two `format!` filters
    go.
  - `ui_sessions`: the `else if` that draws the "none" sentence becomes two
    arms, both only with a non-empty trimmed field — searching
    (`background_tasks` holds `TranscriptSearch` or `KnowledgeSessionSearch`)
    draws 「履歴を検索しています…」; otherwise "none"
    is drawn only when `self.transcript_searched.as_deref()` equals the trimmed
    field and `knowledge.session_hits` is empty.
- **`src/app.rs`.** `OperonApp` gains `transcript_searched: Option<String>`, in
  memory only, set to the query in the `BackgroundResult::TranscriptSearch` arm
  of `process_background_results`.
- **`src/i18n_tables.rs`.** One sorted row in the English and Korean tables.
- No persisted shape, no external command, no new background task. The draw
  path reads one `HashSet::contains` pair and one string comparison for the
  history panel; the session filter allocates only while the field holds text,
  and less than the `format!` per session it replaces except for the title.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The new line uses `RichText::weak()`, as the sentence it stands in for; no colour literal (read against `.claude/rules/palette-and-glyphs.md`). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph is added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier is added; the agent's screen name is read from `agent_choice_copy`, its one home. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Store, cancellation, migration, and byte limits are untouched; the new field is in memory only. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process or launch path. |
| Documentation — user-facing docs change in all three languages together | No | `git grep -n "セッションを検索\|ローカル履歴" README*.md docs/MANUAL*.md` — the manuals describe neither which fields the search reads nor the "none" sentence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path; the remembered query is one string. |
| Draw path (`.claude/rules/rust.md`) | Yes | No I/O or command. Per-session lower-casing runs only while the field holds text and replaces a `format!` that ran there already. |
| Module size band (`largest_module_lines`, propose at 10 000) | Yes | `src/app.rs` (9 962) grows by under 10 lines and `src/app/screens.rs` (9 871) by under 50; `bash scripts/check-bands.sh` is in the contract. |

## Flagged concerns

- **Matching the project's name keeps every session of that project.**
  Answered: the list is grouped under the project's name, so the name is on
  screen for each of those rows, and the palette already behaves this way.

## Acceptance

- `cargo test --locked search_310_` — 4 passed.
- `cargo test --locked every_message_row_is_one_the_code_shows`,
  `every_message_id_has_a_row_in_every_table`,
  `every_translation_table_is_sorted_by_message_id`,
  `history_partial_results_are_disclosed_even_during_automatic_discovery` — ok.
- `make q.fast` and `bash scripts/check-bands.sh` — no new breach.
- In the packaged app: typing a word from an unnamed session's listed title
  keeps its row; typing into 「ローカル履歴」 shows nothing until Enter, then
  「履歴を検索しています…」, then matches or 「一致する履歴はありませんでした。」.

## Rejected alternatives

- **Search as you type for 「ローカル履歴」** — the scan reads up to its byte
  ceiling of transcripts; running it per keystroke is the cost the Enter was
  put there to avoid.
- **Caching a lower-cased haystack per session** — a second copy to invalidate
  on rename, first prompt, and project rename, for a filter over tens of rows
  that runs only while the field holds text.
- **Putting the match in `src/agents.rs` beside `session_list_title`** — it
  needs the project list and the prompt turns the app holds, and that file is
  a launch-gate surface.
- **Clearing old matches when the text is edited** — it would hide results the
  person may still be reading; left as today and named out of scope.
