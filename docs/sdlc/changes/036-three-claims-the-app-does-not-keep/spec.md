# Spec: the launch screen says what it will run

- **Intent**: `./intent.md`
- **Status**: draft

## Requirements

1. The account row is drawn in step 1, under the agent cards, whenever the
   chosen agent has a login variable. Nothing has to be unfolded to see it.
2. The folded header of step 3 names every choice made inside it, so a folded
   section never hides a decision a person took. At most three are named; past
   that it says how many more there are.
3. The permission mode, the reasoning effort, and the launch switches for the
   chosen agent are pickable on the launch screen. Every value offered comes
   from `agent_mode_options`, `agent_effort_options`, and `agent_flag_options`;
   the drawing code writes no flag of its own.
4. A switch that runs the agent without asking permission is marked as such
   wherever it is offered, and cannot be turned on without the written
   acknowledgement that already exists.
5. The acknowledgement is decided by the command the launch will run, not by
   which picker was used. A dangerous switch typed into a custom command is the
   same answer as one ticked in a box.
6. Exclusivity the CLI declares is enforced before the launch: ticking a switch
   in a group turns the others in that group off, and a switch that refuses a
   mode is not offered while a mode is set.
7. The `native resume:` line on a session card is the command that a restart of
   that session will run, character for character.
8. The two places that show a resume command for a session Operon did not launch
   — the copy button on a discovered CLI session, and the notice when one is
   adopted — keep showing the bare resume command, because there is no launch
   command for them to carry.
## Behaviour

### The launch screen

Step 1 gains one row under the three agent cards, separated from them, drawn
only for `codex` and `claude`:

```
1. AI を選ぶ                      必須
  ( ) Codex CLI   (•) Claude Code   ( ) Antigravity
  ────────────────────────────────────────────────
  アカウント  [ 仕事 ▾ ]        ← 未登録なら「「設定」で追加できます」
```

Step 3 gains three controls under モデル, and its header carries what is set:

```
▸ 3. 詳しい設定（任意）· モデル opus · 権限モード acceptEdits · ほか 1 件

  （開いたとき）
  セッション名（任意）  [                    ]
  モデル（任意）        [ opus               ]
  権限モード            [ acceptEdits      ▾ ]
  推論の深さ            [ 既定（CLI に任せる） ▾ ]
  [ ] 権限確認をすべてスキップする（危険）
  [ ] 後から「権限確認をすべてスキップ」を選べるようにする
  [ ] IDE に自動接続する
  [ ] 詳細ログを出す
  作業フォルダ …
```

- The mode combo's first entry is `既定（CLI に任せる）`, which is today's
  behaviour: no mode flag is passed at all. Its label is the CLI's own word for
  the setting — `agent_mode_label` already answers this, and says
  サンドボックス for Codex, 権限モード for Claude Code, 実行モード for
  Antigravity.
- The effort combo is the same shape with the same first entry, labelled
  `推論の深さ`. Not in the screen that was agreed, and included because it is the
  third input of the same dead trio: `agent_effort_input` is cleared, read at
  launch, and written by nothing. Restoring two of the three and leaving the
  third would be a gap with no reason a reader could reconstruct.
- Each switch is a checkbox whose label is the flag's own `label` and whose
  hover is its `detail`. A `dangerous` flag draws as
  `{label}（危険）` in `palette.accent_soft`, the colour the existing danger
  warning already uses.
- Ticking a switch that names a `group` turns off every other switch in that
  group, because the CLI refuses the pair and a refusal at launch is a window
  that opens, prints usage, and dies.
- A switch whose `conflicts_with_mode` is true is disabled while a mode is
  chosen, with the hover saying so:
  `「{label}」は{mode}の指定と同時に使えません。` — the sentence
  `resolve_agent_flags` already produces, moved to where the choice is made
  instead of after it.
- The danger warning and the `理解したうえで起動する` checkbox are unchanged in
  wording and now reachable: they appear whenever the command that would run
  carries a dangerous switch. That includes a custom command — typing
  `claude --dangerously-skip-permissions` into 起動コマンド now raises the same
  warning it would raise from a checkbox.
- Empty: no accounts registered draws the account combo with only
  `このマシン` and the existing 「設定」で追加できます note. An agent with no
  modes, no efforts, and no flags (`custom`) draws none of the three controls.

### The session card

```
before   native resume: claude --resume abc-123
after    native resume: claude --dangerously-skip-permissions --model opus --resume abc-123
```

Unchanged when the session was launched with no options: the two strings are the
same string. The line stays small and weak; it is evidence, not a control.

## Design

**`src/agents.rs`**

- `dangerous_launch_arguments() -> Vec<&'static str>` — every `args` entry of
  every `AgentFlag` whose `dangerous` is true, across all three tables, plus the
  two dangerous mode values. Derived from the tables rather than retyped, so a
  table edit cannot leave this behind (lesson 004).
- `command_needs_acknowledgement(command: &str) -> bool` — whether the command
  that will run carries one of them, matched on whitespace-separated tokens so
  that a path containing the text is not a match.
- `launch_needs_acknowledgement` keeps its name and signature-plus-one: it gains
  the custom command, and answers true when the mode is dangerous, a chosen flag
  is dangerous, or — for the custom agent, which has neither — the command
  itself carries one.

**`src/app.rs`**

- `LaunchAcknowledgement` gains `custom_command`, so that consent given for one
  typed command is not consent for another.
- `launch_detail_summary(&self) -> Vec<String>` — the choices made inside step 3,
  as already-translated `name value` fragments, in the order they are drawn.
  Named because the header and its guard must be the same answer; a summary the
  test recomputes is a summary that can drift from the one on screen
  (lesson 005).

**`src/app/screens.rs`**

- The account row moves out of the `CollapsingHeader` into the step-1 frame.
- `CollapsingHeader::new` takes the summarised title.
- Three controls are added inside step 3, between モデル and 作業フォルダ.
- The `native resume:` line calls `resume_command_with_launch_options`, the
  function the resume itself calls, instead of `native_resume_display`.

**`src/i18n_tables.rs`** — three new ids: `ほか {p0} 件`, `{p0}（危険）`,
`推論の深さ`. Everything else composes strings that are already translated, the
way `session_title` and its status label already do.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The dangerous switch uses `palette.accent_soft`, the role the existing danger warning on this same screen already uses. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. Checkboxes and combo boxes are egui's own. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The dangerous arguments are read out of the flag tables rather than retyped, and the resume line calls the same function the resume calls. Neither value is written twice. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No persisted shape changes. `LaunchAcknowledgement` is app state that lives for one launch. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child. The launch command is still built by `build_agent_launch_command` and still passes `is_safe_agent_command` and `is_safe_agent_option`; this change only lets a person reach inputs that were already validated. |
| Documentation — user-facing docs change in all three languages together | Yes | The launch options become visible behaviour, so `README.md`, `README.ja.md`, and `README.ko.md` gain the same sentence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Nothing leaves the machine and no dependency is added. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The folded header names at most three choices and then a count; the flag list is a `&'static` table of at most five entries per agent. No scan. |

## Flagged concerns

- **Work in the draw path.** `launch_needs_acknowledgement` is called twice per
  frame on this screen and now also walks the dangerous-argument list over the
  custom command. That list is at most four short strings and the command is one
  short validated line, which is the same order of work as the `tr()` calls
  beside it — the argument change 027 made for `resume_verb` and the reviewer
  accepted. **Resolved**: kept, with the same note at the call site, and the
  list is built from `&'static` tables so it allocates one `Vec` of borrowed
  strings rather than copying any text.
- **The reasoning-effort control was not in the agreed screen.** **Resolved**:
  included, stated above and in the report, because it is the same defect as the
  two that were agreed and the person's answer was to restore the picker rather
  than to restore two thirds of it.

## Acceptance

- `cargo test --locked` passes, including:
  - `the_account_row_is_on_the_screen_without_unfolding_anything`
  - `a_folded_section_names_the_choices_made_inside_it`
  - `the_launch_screen_offers_every_mode_and_switch_the_agent_has`
  - `a_dangerous_switch_cannot_be_launched_without_the_acknowledgement`
  - `a_dangerous_switch_typed_into_a_custom_command_still_asks`
  - `switches_that_the_cli_refuses_together_cannot_both_be_on`
  - `the_card_shows_the_command_the_restart_will_run`
  - `a_resume_command_for_a_session_operon_did_not_launch_is_unchanged`
- Every one of those is watched failing before it is watched passing.
- In the running app: choose Claude Code, tick 権限確認をすべてスキップする, and
  the warning and the checkbox appear; the launch button stays disabled until the
  checkbox is ticked. Clear it, type `claude --dangerously-skip-permissions` into
  別のコマンドを使う, and the same warning appears.
- In the running app: register a second account, open a project, and the
  アカウント row is visible without touching 詳しい設定.

## Rejected alternatives

- **Leave the account row where it is and only summarise the folded header.**
  The summary tells a person what they chose; it does not let them choose. The
  first-time case is the one that matters here.
- **Delete the dangerous-mode warning and the acknowledgement checkbox as dead
  UI.** That is what change 012 would have done, and it is the wrong direction:
  the gate is not unreachable because it is unwanted, it is unreachable because
  the pickers that fed it were removed and nothing replaced them.
- **Judge danger by scanning the command for the literal text
  `dangerously`.** A substring rule over an arbitrary command is a rule that
  fires on a path and misses a rename. The flag tables already name the exact
  arguments.
