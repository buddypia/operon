# Intent: session tabs do not say which CLI runs in them

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

The person showed the workspace tab strip — 「セッション2」「セッション1」「セッション3」, each with only a state mark — and asked: 「セッションタブに良い感じに実行中のAIエージェントCLIのアイコンを表示したい。ターミナルだとターミナルのアイコン（生成して使ってOK）」. Since change 253 a session name no longer repeats its agent, so a tab says nothing about which CLI is behind it, while the session row in the left column already shows the vendor mark.

## Who feels it, and when

Someone running Antigravity, Claude Code, Codex, and plain terminals side by side in one project must open each tab, or find its row in the left column, to learn which tool it is before typing into it.

## Desired outcome

Reading a tab tells the person which CLI it runs: the vendor's mark for Antigravity, Claude Code, and Codex, and a terminal mark for a plain or recovered terminal — at a glance, beside the state mark the tab already shows.

## Constraints this change inherits

Local-first, no new dependency, Japanese text, palette roles for every painted colour, the vendor marks as shipped in `assets/agent-icons/`, and the launch gates in `src/agents.rs` left untouched.

## Systems likely affected

The workspace tab drawing, the shared mark helpers, a new bundled image for the terminal mark, the design document, and UI tests.

## Open questions

None. The person authorised generating the terminal icon; it is Operon's own artwork, not a copy of any vendor's terminal app icon.

## Not in scope

The left column's session rows (they keep their current agent slot), the CLI filter, session naming, and status detection.
