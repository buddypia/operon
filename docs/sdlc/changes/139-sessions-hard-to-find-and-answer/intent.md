# Intent: the agent waiting for an answer has to be hunted for

- **Status**: approved
- **Opened**: 2026-10-09

## Problem

The person runs several agent CLIs in parallel, and they finish at uneven
times. To answer one, they first have to find which session stopped, then open
it to learn what it is asking. Nothing on the session screen puts "who is
waiting for me, and what did they ask" in front of them, and no key takes them
there.

The screen around that work also reads as cramped and doubled: the session list
and the file / conversation / changes panel share one left column and each gets
a few rows; the page navigation, the session header, and an empty band above
the terminal take three rows before the terminal starts.

In the person's words:

> UI配置にすごく違和感がある。VSCODEやTMUXやCMUXなど業界標準を考えて利便性の高いUI/UXを実現したい。

> セッションやプロジェクトが多くても素早く探せるように工夫したUI/UXが必要。並列で作業させているからターミナル上で動いているCLIの終了にばらつきがあるから面倒臭いから瞬時にプロジェクトを探したり瞬時にAIに会話に回答できる内容が見つかると良い

Three layouts were drawn as clickable HTML mockups; the person chose the first:

> A 返事待ちを先頭に置くで進めて

## Who feels it, and when

Anyone with more than a handful of sessions running at once — the normal case
here — every time an agent stops to ask something while the person is reading
another one's output.

## Desired outcome

1. The sessions waiting for an answer are always at the top of the session list,
   longest-waiting first, each showing the gist of what it asked.
2. One key moves to the next waiting session and leaves the person ready to type
   the answer.
3. The one search (⌘K) finds a session by its name, project, branch, or the
   words of what its agent said, and puts a waiting one first.
4. The session list, the terminal, and the file / conversation / changes panel
   each have their own column, and the panel can be folded away with one key.
5. The window spends one row above the terminal on navigation and one row on
   the session, not three.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- Agent CLIs own their keys while the terminal has focus.

## Systems likely affected

The session workspace drawing in `src/app.rs` and `src/app/screens.rs`, the
keymap, the status hooks in `src/tmux/hooks.rs` if they are where an agent's
question can be read, `src/i18n_tables.rs`, and the suite in `src/tests.rs`.

## Open questions

- Where the words of an agent's question can be read for each of the three CLIs
  without guessing — answered by the spec from the hooks and transcripts code.

## Not in scope

- Answer buttons that type a reply for the person (mockup C): they depend on
  reading each CLI's prompt correctly, and a misread sends the wrong keys.
- An activity bar or session tabs above the terminal (mockup B).
- macOS notifications and a Dock badge; a later change.
- Persisting the new column widths across launches.
