# Spec: Short numbered session names

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Newly launched sessions without an explicit name use セッション1, セッション2, and so on within their project, independently of CLI or initial request. Choose the first available number above the project's current session count; avoid collisions with numbered names in every UI language (`session_253_numbered_defaults`). Explicit names and restored conversation titles remain intact.
2. In the workspace sidebar and session tabs, existing empty-request launch fallback names such as design-lookbook · Antigravity are shown as numbered sessions, ordered by creation time then UUID within their project. Existing conversation-topic and custom names keep their title; a first prompt already discovered for a legacy fallback still supplies its topic (`session_253_compact_titles`). No saved record is rewritten. The hero and crumb use the same title as the tab.
3. Double-clicking a session name or choosing 名前を変更 from its right-click menu in the workspace sidebar or tab opens an inline editor at that location. Enter or blur saves through the existing rename operation; Escape cancels; empty input keeps the old name and its existing notice. A rename updates both sidebar and tab and survives reloading (`session_253_rename_from_sidebar_and_tab`, `renaming_a_session_is_durable`). Existing history rename controls remain available.

## Behaviour

Within each project the unnamed launch starts at セッション1. Numbering is shared across CLI kinds, not across projects. Deleting or renaming sessions can release numbers; no permanent counter is promised. Long custom names still truncate and reveal their full name on hover. Agent marks and status marks remain. Empty projects and ended sessions keep their existing states. No loading or external command is added. Save failures use the existing persistence notice and rollback. Interrupted editing does not change a saved name. Empty names show the existing セッション名は空にできません。前の名前のままにしました。 notice.

## Design

Placement: EXTEND the launch-name calculation and rename state in src/app.rs, and the sidebar/tab drawing in src/app/screens.rs. A pure numbered-name helper uses tf! with セッション{p0}; translation rows in src/i18n_tables.rs. App helpers choose the next free default and project legacy fallback titles for all workspace callers. Recognise a legacy launch fallback using empty goal, the workspace basename, and its matching provider label; custom and unknown command names are retained. Reuse session_list_title for conversation topics. New memory-only rename-location flag prevents two editors sharing focus. Existing persisted shape, persistence operation, commands, and background tasks are unchanged. No IO in drawing; title projection scans only the in-memory sessions.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | yes | Existing palette inks and controls, no new role. |
| Icons — src/glyphs.rs | yes | Existing vendor marks and status marks remain. |
| Identifier SSOT — src/config.rs | no | No command, path, or external identifier added. |
| Durability — durability-invariants skill | yes | Reuse the existing rename persistence and rollback; no schema or write-order change. |
| Subprocess safety — src/exec.rs and src/agents.rs | no | No launch command or child lifecycle changed. |
| Documentation — CONTRIBUTING.md | no | No README changes required for a label and direct rename affordance. |
| Local-first — CONTRIBUTING.md | yes | Only in-memory title projection and existing local writes. |
| Budgets | no | No scan of disk or subprocess output. |

## Flagged concerns

None. The person explicitly requested numbering and both direct rename gestures. Per-project numbering follows the project grouping they referenced.

## Acceptance

- cargo test --locked session_253_ and the existing rename tests pass; make q.fast passes.
- Mutation sweep catches missing numbering, repeated launch context, and disabled rename gestures.
- Rendered sidebar and tabs show numbered names and their marks; inline editor saves and cancels.
- CI passes before landing; package from main. Installation is pending until the running app can be quit with permission.

## Rejected alternatives

- Persistently rewriting existing names: unnecessary migration of user data.
- Global numbering: the project already groups each sequence.
- A new dialog: the existing inline rename model is sufficient.
