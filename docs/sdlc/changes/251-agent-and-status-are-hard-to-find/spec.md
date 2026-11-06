# Spec: Recognisable agents and sidebar session filters

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The left project sidebar exposes CLI and status filters, including an explicit approval/answer waiting choice; selecting them changes its actual rows and reset restores the normal view (`sidebar_251_filters_drive_real_rows`).
2. CLI choices combine by OR; status choices combine by OR; the two dimensions combine by AND. Filtering includes finished sessions when requested, respects fresh status resolution, and does not change selected session, workspace tabs, or running processes (`sidebar_251_filter_combinations`).
3. Official bundled vendor marks identify known CLIs in the sidebar and session rows without repeated CLI words; tooltips and accessibility retain names, and terminals/custom CLIs retain a readable fallback (`sidebar_251_marks_and_accessible_filters`).

## Behaviour

The sidebar shows 「CLI」 followed by 「すべて」, the three official icon buttons, and 「その他」. Below it 「ステータス」 offers 「すべて」「承認・回答待ち」「要対応」「実行中」「指示待ち」「終了」, wrapping within the existing column width. Multiple choices toggle independently. No selection means unrestricted in that dimension. 「承認・回答待ち」 means Waiting, which includes both permissions and questions; hooks do not distinguish the two reliably. 「すべて」 clears its own dimension. Existing history filters share the same state. A sidebar with active filters shows only matching projects and rows, including ended sessions; no matches says 「一致するセッションはありません。」. Clearing preserves the normal live-only sidebar. Finished sessions remain in history by default. No disk reads, process calls, asynchronous work, loading state or new failure path is introduced. Existing empty-project and status-checking states remain. Filter changes cancel pending row removal confirmation.

## Design

Placement: EXTEND the existing `OperonApp` filters and `ui_workspace_projects` in `src/app/screens.rs`; no new module. Add memory-only agent selection and waiting selection alongside `status_filter` in `src/app.rs`. Reuse status resolution, agent textures and palette roles. One combined predicate serves sidebar, pinned rows, history list and grid. Sidebar exposes reusable filter controls, with full official names as accessible labels. Agent slot groups codex/claude/gemini/other, including legacy/custom strings in other. Existing marks and SOURCES.md are sufficient; no network or dependency change. `show_status_group` clears unrelated CLI/waiting filters so Home shortcuts show the group they count. Update all three translations for new strings.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | yes | Existing text, selected, status and control roles only; no new colour. |
| Icons — src/glyphs.rs | yes | Reuse bundled vendor PNGs and existing terminal glyph; no new glyph. |
| Identifier SSOT — src/config.rs | no | No persisted identifier or external command contract changes. |
| Durability | no | No store shape, cancellation, migration or byte limit changes. |
| Subprocess safety — src/exec.rs and src/agents.rs | no | No commands, launch changes, or edits to agent gates. |
| Documentation — CONTRIBUTING.md | yes | English development records; no public documentation change. |
| Local-first — CONTRIBUTING.md | yes | Bundled assets, local memory; no network, accounts, telemetry or unsafe. |
| Budgets | no | No scanning or output path introduced. |

## Flagged concerns

None. The user's request explicitly authorises left-sidebar placement; no further layout approval is needed. Waiting covers both permissions and answers and the label says so.

## Acceptance

- `cargo test --locked sidebar_251_` and relevant existing status/sidebar guards pass.
- `make q.fast` passes; mutation sweep catches each changed requirement.
- CI passes before merge. Main is packaged after landing; installation follows the ship procedure.
- Observe actual filter control selection, official marks, wrapped sidebar at its normal width, no-match and finished-session states.

## Rejected alternatives

- Only filtering history: the person explicitly requested the left sidebar.
- Inferring permission-only waiting: the existing status signal also includes questions.
- Persisting filters: transient view preference does not justify changing a stored shape.
