# Intent: buttons and clickable actions lack visual affordance at rest

- **Status**: approved
- **Opened**: 2026-10-11

## Problem

Multiple interactive buttons and dismiss/cancel actions in the application are drawn using `quiet_button` or plain text labels without visible borders or resting cues. Because `quiet_button` explicitly sets `.stroke(Stroke::NONE)` with `palette.text_muted` and transparent fill, controls like 「バックグラウンドで続ける」 (Continue in background) in progress modals and 「ターミナルだけ開く」 (Open terminal only) in the launch sheet look like static explanatory text rather than clickable buttons until hovered. Users cannot discover that these actions can be clicked without moving the cursor over them.

## Who feels it, and when

Anyone using the application:
1. When starting an agent CLI or terminal, or restoring session history, a progress modal appears. The only dismiss action at the bottom right is 「バックグラウンドで続ける」, but it looks like a footnote note, leaving users confused about how to dismiss or background the task.
2. In the session launch sheet, 「ターミナルだけ開く」 sits beside the primary launch button but lacks a button frame.
3. In project deletion confirmation cards, 「キャンセル」 uses `quiet_button` while the dangerous delete button is a filled red button.
4. Inline actions such as 「別のコマンドを使う」, 「プロジェクト直下に戻す」, 「今の設定で固定し直す」, and 「手順」 appear as unadorned text without click affordances.
5. Beside section headings and on the toolbar, actions like 「すべて表示」, 「履歴」, and 「要対応」 blend in with adjacent static counter labels.

## Desired outcome

1. **Modal and sheet action buttons**: Replace `quiet_button` with `secondary_button` for dialog dismiss, cancel, and alternative launch actions (e.g. 「バックグラウンドで続ける」, 「ターミナルだけ開く」, and inline project removal 「キャンセル」). At rest, these display `border_subtle` hairline outlines and standard text color, making them unmistakably buttons.
2. **Inline interactive choices**: Convert unadorned inline action texts (such as 「別のコマンドを使う」, 「プロジェクト直下に戻す」, and tool 「手順」) into clear `secondary_button` controls.
3. **Heading actions & quiet button affordance**: Give `quiet_button` clear resting interactive cues (such as `palette.link` color or navigation indicators like `›`) and provide the toolbar attention control with a visible chip/border treatment so it is immediately identifiable as an actionable navigation jump.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Conformance to `DESIGN.md` button hierarchy: primary action is filled accent; ordinary/modal actions are `secondary_button` (hairline at rest); quiet heading actions are subtle but distinguishable from passive text.
- No new external dependencies.
- Standard i18n support preserved (Japanese, English, Korean).

## Systems likely affected

- `src/ui/widgets.rs` (progress footers and quiet_button styling)
- `src/app/screens.rs` (modal footers, inline options, breadcrumbs, attention toolbar button)
- `src/tests.rs` (widget layout and button focus/styling tests)
- `DESIGN.md` (ensuring documentation matches button roles)

## Not in scope

- Modifying primary button styles (`primary_button`).
- Changing table or session list card interactions (`clickable_card`).
- Altering the business logic or execution of any button action.
