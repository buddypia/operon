# Regression: project selection lost its controls

The user reports: 「セッションで左サイドバーの選択が丸ごと劣化しているじゃん。セッションページの起動中のセッションやファイル内容でVSCodeやChromeタブのようにしてって言ったよね。」

## Named cause

The central-tab change replaced the production sorted/action-bearing project
navigator with unsorted name buttons. Project clicks open every historical
session regardless of status and choose the last one. Initial workspace setup
opens only the selected session. select_session and document-tab activation do
not synchronize the selected project, so the left highlight becomes stale.
This was a scope error: centralizing running sessions/files did not authorize
removing project navigation affordances.

## Requirements and acceptance

1. Restore sorted/arranged project cards, counts, per-project launch control and
   drag ordering in the left project-only navigator, with bounded long names.
   Prove actual pointer launch/reordering and count/control visibility.
2. Initial entry and project selection expose Starting/Active/Unknown sessions
   as central tabs. Do not automatically add Exited/Completed/Queued history;
   explicit history selection may still open any record. Repeated project
   selection preserves the focused tab rather than jumping to the last record.
3. Focused session and document tabs keep left project selection synchronized.
   Empty projects provide their own empty workspace, with launch/history still
   reachable. Project switching preserves open files, dirty buffers and split
   arrangements. Tab close never stops a session or reopens it every frame.
4. Keep mixed central tab switching, reorder, both split axes, divider resizing,
   targeted file navigation, dirty-close protection, and history operations.
5. Use project-scoped tabs, preserving each project's workspace in memory.
   The user explicitly chose 「選んだプロジェクトごとにタブを切り替える（開いたタブ・分割は保持）」.
   No new durable workspace format is introduced.

No new persistence, cancellation, restore, command, dependency or file scan.
The existing arrangement writer remains unchanged; only call its established
project-reorder path.
