# Bug: Shift+Enter in a session terminal sends the line instead of a newline

Reported 2026-11-09. In the person's words, after a first report that read the
other way round ("Shift + Enterで勝手に改行されて困る"):

> やっぱAntigravity CLIではダメだった。キーの設定とかの問題かな？［普段使っているターミナル］では問題なくShift + Enterができて改行も問題ない

In the terminal app the person uses day to day, Shift+Enter puts a newline in
the agent's input and Enter sends it. In
an Operon session terminal running Antigravity CLI, Shift+Enter sends the line,
the same as Enter.

## Cause

`terminal_key_binding` (`src/ui/terminal.rs`) maps `egui::Key::Enter` to the
tmux key `Enter` whatever the modifiers, so Shift+Enter reaches the pane as the
same `\r` as Enter. tmux cannot carry the Shift either way: with
`extended-keys off` (the default, and the setting on the person's server, whose
panes all report key mode `VT10x`) `send-keys S-Enter` writes `\r` as well
(od probe, tmux 3.8). An agent CLI therefore cannot tell the two keys apart and
submits.

That terminal app's config binds
`keybind = shift+enter=text:\x0a`: it sends LF (Ctrl+J) for Shift+Enter.

Evidence that the key reaches Operon as Shift+Enter: a probe built on the same
eframe 0.31 / winit 0.30.13 with Kotoeri, driven by the person, recorded
`insertNewline:` and `Key Enter pressed shift=true` for Shift+Enter after a
committed conversion; Shift+Enter while composing only commits, as Enter does.

## Fix

Send Shift+Enter as what Ctrl+J sends, tmux `C-j` (LF). Measured in a scratch
tmux server, text then `C-j` then more text: Claude Code 2.1.319, Codex 0.167
and Antigravity CLI 1.5.3 each show the two lines in their input without
sending. A shell runs the line on LF, exactly as it does in that terminal app. Enter,
Ctrl+Enter and Option+Enter keep their current keys, and an Enter that confirms
a conversion still belongs to the IME.

No persisted shape, string, command, or screen changes.

## Requirement

1. Shift+Enter in a session terminal reaches tmux as `C-j`, the key Ctrl+J
   sends, after a confirmed conversion as well as on its own; Enter stays
   `Enter`, Ctrl+Shift+Enter stays `C-Enter` and Option+Shift+Enter stays
   `M-Enter`; Shift+Enter while composing sends nothing —
   `shift_enter_puts_a_newline_in_the_agents_input_instead_of_sending`.
