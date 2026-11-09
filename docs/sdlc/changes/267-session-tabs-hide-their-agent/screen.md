# Session tab CLI marks

- **Status**: approved

The existing horizontal tab strip at the top of a workspace pane. Only the inside of a session tab changes.

Before: [⟳ セッション2 ×] [✓ セッション1 ×] [⏸ セッション3 ×]
After:  [⟳ (Antigravity mark) セッション2 ×] [✓ (Antigravity mark) セッション1 ×] [⏸ (terminal mark) セッション3 ×]

The lines describe adjacent tabs in one row. The CLI mark is a 16pt rounded square — the macOS app icon for Antigravity, Claude Code, and Codex; Operon's own terminal mark (light plate, dark screen, green `>` and light `_`) for a plain or recovered terminal — between the 18pt state mark and the 12pt title, centred on the row. The title keeps its 40–132pt truncating slot, the close button stays last, and the active tab keeps its accent underline.

Hover on the mark: `Antigravity` / `Claude` / `Codex` / `ターミナル` / `復旧ターミナル`. Hover on the state mark: unchanged.

Empty: no tabs. Custom command or unknown binary: no mark, the title follows the state mark as before. Missing session: blank title, no mark. Long title: truncates after the mark. Many tabs: horizontal scrolling, unchanged. Error or interrupted states: only the state mark changes; the CLI mark stays.
