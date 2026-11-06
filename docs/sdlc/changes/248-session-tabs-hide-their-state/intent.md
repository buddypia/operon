# Intent: identical session tabs hide which session needs attention

- **Status**: draft
- **Opened**: 2026-11-07

## Problem

The person showed two identically named Antigravity tabs and asked: 「指示待ち、作業中、完了みたいな感じがわかるアイコンにしたい。タブの冒頭に」. A plain ring does not explain what to do next.

## Who feels it, and when

Someone comparing several sessions in one project must open each tab to discover whether it needs another instruction.

## Desired outcome

Reading the beginning of a tab tells the person whether the session is waiting, busy, or finished, even when names repeat.

## Constraints this change inherits

Existing local status detection, Japanese text, bundled Phosphor glyphs and semantic palette roles.

## Systems likely affected

The workspace tab drawing, shared status glyph vocabulary, and UI tests.

## Open questions

None; the request authorizes state cues within the existing tabs.

## Not in scope

Changing session names, inferring completion from terminal text, or changing session lifecycle detection.
