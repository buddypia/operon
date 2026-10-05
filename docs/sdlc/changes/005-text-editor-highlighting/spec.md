# Spec: Syntax-highlighted project text editor and review flow

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The editable file view SHALL apply grammar-backed syntax highlighting for the common project
   source, script, markup, data, and configuration extensions already presented
   as code in the project tree: Rust, TypeScript/JavaScript, Python, Go, Ruby,
   shell, C/C++, Swift, Kotlin/Java, PHP, SQL, HTML, CSS, Vue, Lua, JSON, TOML,
   YAML, and Markdown/MDX.
2. Unsupported extensions SHALL remain editable as plaintext with no misleading
   language-specific treatment.
3. Highlighted editing SHALL retain the current editor guarantees: monospaced
   unwrapped text, line-number gutter, keyboard editing (including Tab), save,
   reload, read-only explanations, and the 128 KiB input ceiling.
4. The existing Markdown Preview and per-file Git Diff views SHALL remain
available, so agent-authored text can be compared to the last commit with
added and removed lines visibly marked. A Diff view that is already open SHALL
fetch a new working-tree comparison after an external file update is detected;
it must never present a cached comparison as the current result.
5. Repeated editor frames with unchanged text, language, and palette SHALL
reuse the syntax layout and line-number gutter rather than re-tokenizing the
entire document or rebuilding every number.
5. All new user-visible wording SHALL be Japanese and translated through the
   existing tables.

## Behaviour

Opening any supported text file shows its content as editable, monospaced text
with comments, strings, numbers, keywords, structural punctuation, and relevant
format delimiters distinguished by semantic palette roles. The gutter and text
continue to scroll together horizontally and vertically without wrapping.

An unknown extension opens as normal editable plaintext. Binary or oversized
files retain the present explanation in place of an editor. Markdown continues
to offer 編集, プレビュー, and 差分; every other text file offers 編集 and 差分. The
差分 view remains the working-tree comparison against the last commit, including
the clear unchanged and error states already provided by the editor.

## Design

`src/files.rs` owns a pure extension-to-language classification that reuses the
same recognized extension set as the tree icon. A focused `src/ui/syntax.rs`
module turns one classified text buffer into an `egui::text::LayoutJob` using
Syntect's bundled, resilient syntax definitions. Its theme is constructed from
the existing semantic `Palette` roles, not raw drawing colours. An egui frame
cache keys layouts by text, language, and palette so unchanged frames reuse
their result. Plaintext remains a direct semantic-colour layout.

`ui_editor_text` supplies that layout job through egui's `TextEdit::layouter`
hook, preserving the existing horizontal no-wrap layout. The line-number text
is cached on the open document and is recomputed only when its buffer changes.
The diff renderer stays the single owner of diff line colours and parsing.
While an editor Diff is visible, a throttled background request re-fetches that
file's Git Diff directly. Before each request its cached text and parse are
dropped, so the UI says that it is loading rather than presenting an old
comparison as current. Git status continues to refresh the file-tree badges but
is not treated as a content fingerprint. Each request carries an in-memory
generation; a result from an earlier generation is discarded and causes the
latest generation to be requested after its older Git process completes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | Reuses documented semantic palette roles; no new colour role or literal. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The existing editor view icons remain unchanged. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | Format recognition is local pure classification, not a cross-system identifier. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No persisted shape or write path changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No subprocess is added. |
| Documentation — user-facing docs change in all three languages together | Yes | The editor capability is documented consistently in English, Japanese, and Korean. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Syntect runs in-process against bundled syntax definitions; no telemetry, account, cloud call, or `unsafe` is introduced. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The existing `EDITOR_FILE_MAX_BYTES` 128 KiB ceiling bounds lexing; no additional scan or output path exists. |

## Flagged concerns

- **Grammar behaviour on malformed agent output** — Syntect must preserve the
  editable bytes and return a safe layout when a document is incomplete. Its
  fallback is the normal semantic plaintext layout; grammar errors never block
  editing or saving.
- **External change freshness** — the file tree does not own all agent writes,
  and a repeated write leaves `git status` unchanged. While Diff is open, the
  application periodically discards and re-requests that file's actual Diff;
  a generation fence prevents an older in-flight Git process from restoring
  cached output after the refresh.

## Acceptance

- `cargo test --locked` passes, including format classification, grammar-backed
  syntax layout, cached-layout, Diff freshness, and editor rendering coverage.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass.
- In the installed app, opening Rust, JSON, YAML, Markdown, and a `.txt` file
  shows editable source with appropriate highlighting (or plaintext fallback),
  and the Diff tab shows Git's added and removed lines for an agent change.

## Rejected alternatives

- Keep the custom lexical highlighter — rejected because it cannot faithfully
  cover the already-declared HTML/Vue and multiline grammar cases, and it
  re-tokenizes every frame despite the immediate-mode API's cache contract.
- Replace the whole GUI framework — rejected because the pinned egui release
  already provides the cache primitives needed here; framework migration is a
  separate compatibility project.
- Treat every extension as a programming language — rejected because invented
  highlighting is worse than an honest plaintext fallback.
