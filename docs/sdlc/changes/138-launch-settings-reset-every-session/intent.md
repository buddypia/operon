# Intent: a session cannot be started the same way every time without re-choosing

- **Status**: approved
- **Opened**: 2026-10-09

## Problem

The person, quoted: 「毎回新しい設定をするのはしんどいからセッション作成時に
チェックボックスを入れて毎回同じ設定でセッション開始するとかチェックしたら
そのようにするようにして。」

Today the launch sheet starts from whatever was last touched, not from a setting
the person chose to keep. The account falls back to 「このマシン」 after a
restart, the danger acknowledgement is cleared on every project switch, and a
one-off change for one launch becomes the next launch's starting point.

## Who feels it, and when

Someone who starts most sessions with one combination — one agent, one model,
one mode, one login, often a mode that needs the acknowledgement — and has to
re-select it on most launches.

## Desired outcome

A checkbox on the launch sheet. With it checked, every new sheet opens already
holding the combination it was checked with — agent, model, mode, effort, flags,
custom command, account, and the danger acknowledgement — so ⌘↩ alone starts the
session. Changing something for one launch does not change what the next sheet
opens with. Unchecking returns to today's behaviour.

## Constraints this change inherits

- User-facing text is Japanese, through `tr` / `tf!`.
- The acknowledgement still only covers the exact combination it was given for.

## Systems likely affected

`src/agents/settings.rs` (where the remembered options live), the launch sheet
in `src/app/screens.rs`, `src/app.rs`.

## Open questions

- Pin the settings, or skip the sheet entirely? — answered by the person: pin.
- Does the pin carry the danger acknowledgement? — answered by the person: yes.

## Not in scope

Skipping the sheet and launching straight away; the request text, session name,
work location and dependency, which are per-launch.
