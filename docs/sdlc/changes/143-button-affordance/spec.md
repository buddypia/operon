# Spec: clear visual affordance on buttons and interactive controls

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `restore_progress_footer` and `launch_progress_footer` in `src/ui/widgets.rs` render the 「バックグラウンドで続ける」 dismiss action using `secondary_button`, giving it a visible `border_subtle` hairline at rest and `CONTROL_HEIGHT` (28px) height.
2. In `src/app/screens.rs`, dialog and sheet secondary actions—specifically 「ターミナルだけ開く」 in `ui_launch_footer` and 「キャンセル」 in both project removal cards—use `secondary_button` instead of `quiet_button`.
3. In `src/app/screens.rs`, inline interactive action choices—specifically 「別のコマンドを使う」 (AI choice), 「プロジェクト直下に戻す」 (working directory reset), 「今の設定で固定し直す」 (launch pin update), and tool 「手順」 (tool status setup)—render as visible controls via `secondary_button`.
4. `quiet_button` in `src/ui/widgets.rs` renders with `palette.link` color instead of `palette.text_muted`, so that heading actions like 「すべて表示」 and 「履歴」 are clearly identifiable as interactive link-style actions rather than static muted text labels.
5. In `src/app/screens.rs`, the toolbar attention button (「要対応」) renders with button framing via `secondary_button` or framed chip so it reads as an actionable jump button rather than a plain text label.
6. All updated buttons maintain the documented control height (`CONTROL_HEIGHT` = 28px) and retain proper keyboard focus rings.

## Behaviour

- In launch and restore progress modals, the bottom-right action 「バックグラウンドで続ける」 displays a distinct hairline rectangular border and standard text color at rest. Hovering fills the background with control hover color.
- In the new session launch sheet footer, 「ターミナルだけ開く」 appears with a visible secondary button frame at rest, standing out clearly beside the primary launch button.
- In project deletion confirmation cards, 「キャンセル」 appears as a framed secondary button next to the dangerous red delete button.
- In the new session launch sheet, 「別のコマンドを使う」, 「プロジェクト直下に戻す」, and 「今の設定で固定し直す」 appear with secondary button styling.
- In settings agent tool status rows, the 「手順」 action appears as a secondary button.
- Beside section headings (e.g. 「最近のセッション」 -> 「すべて表示」, 「プロジェクト別セッション」 -> 「履歴」), `quiet_button` displays link-colored text (`palette.link`), visually separating it from adjacent non-clickable count labels.
- In the window toolbar, the 「要対応」 button displays with secondary button styling at rest.

## Design

### Placement verdict: `EXTEND`

Existing button helpers in `src/ui/widgets.rs` and their invocation sites in `src/app/screens.rs` are updated. No new module or persisted shape is introduced.

1. `src/ui/widgets.rs`:
   - In `restore_progress_footer` and `launch_progress_footer`, replace `quiet_button(ui, palette, tr("バックグラウンドで続ける"))` with `secondary_button(ui, tr("バックグラウンドで続ける"))`.
   - In `quiet_button`, change the text color from `palette.text_muted` to `palette.link`. This gives frameless heading actions an unambiguous interactive link appearance at rest without adding noisy borders beside section titles.
2. `src/app/screens.rs`:
   - Replace `quiet_button` calls in modal/sheet footers and inline actions with `secondary_button`:
     - Line 167: `attention_res = secondary_button(ui, attention_label)`
     - Line 1129, 1353: `secondary_button(ui, tr("キャンセル"))`
     - Line 1736: `secondary_button(ui, tr("別のコマンドを使う"))`
     - Line 1889: `secondary_button(ui, tr("プロジェクト直下に戻す"))`
     - Line 2050: `secondary_button(ui, tr("今の設定で固定し直す"))`
     - Line 2100: `secondary_button(ui, tr("ターミナルだけ開く"))`
     - Line 4539: `secondary_button(ui, tr("手順"))`
   - Heading actions at line 698 (「すべて表示」) and line 5081 (「履歴」) continue using `quiet_button`, which now benefits from `palette.link` styling.
   - Breadcrumb navigation at line 1282 continues using `quiet_button` with link styling.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Uses existing semantic palette roles (`border_subtle`, `text`, `link`); no new role added. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No new icons added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No shared or persisted config identifiers modified. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Pure UI drawing change; persistence layer untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No subprocess execution modified. |
| Documentation — user-facing docs change in all three languages together | no | No new user-facing copy introduced; uses existing localized translation keys. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | No network calls, no telemetry, no unsafe code. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new filesystem scans or output buffers. |

## Flagged concerns

- **Visual hierarchy in footers**: Replacing `quiet_button` with `secondary_button` gives footers two framed buttons (e.g. primary launch + secondary terminal only). This aligns with `DESIGN.md` which specifies `secondary_button` for ordinary actions beside primary buttons.
- **`quiet_button` link color contrast**: `palette.link` is defined in all three themes (Dark `#5AAAFF`, Light `#1F5FBF`, High Contrast `#78B4FF`), all meeting WCAG AA contrast against their respective panel backgrounds.

## Acceptance

- `cargo fmt --check` succeeds.
- `cargo clippy --locked -- -D warnings` succeeds.
- `cargo test --locked` passes, including button height and focus ring regression tests.
- In progress modal, 「バックグラウンドで続ける」 displays a hairline border and standard button appearance at rest.
- In launch sheet, 「ターミナルだけ開く」, 「別のコマンドを使う」, 「プロジェクト直下に戻す」, and 「今の設定で固定し直す」 display proper secondary button borders.
- In settings agent tools list, 「手順」 displays as a secondary button.
- Beside headings, 「すべて表示」 and 「履歴」 render with link color rather than muted text.

## Rejected alternatives

- **Adding a border to `quiet_button` globally**: `quiet_button` is intended for quiet actions beside headings. Giving it a heavy border would clutter heading rows where cards and titles already sit closely together. Using `palette.link` preserves calmness while establishing unambiguous click affordance.
- **Leaving modal actions as quiet buttons**: Violates the principle that primary/dismiss dialog actions must be readily discoverable without pointer exploration.
