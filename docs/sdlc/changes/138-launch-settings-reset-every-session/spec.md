# Spec: pin the launch settings with 「この設定を毎回使う」

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Checking 「この設定を毎回使う」 stores the sheet's current agent, model, mode,
   effort, flags, custom command, account, and whether the danger
   acknowledgement is given, in `recent-agent-settings.json`, and it survives a
   save/load round trip. — `pinned_launch_round_trips_through_the_settings_file`
2. A file written before this change (no `pinned` key) loads with no pin.
   — `pinned_launch_round_trips_through_the_settings_file`
3. Opening the launch sheet while pinned puts every pinned value into the form
   and, when the pin was acknowledged, the acknowledgement for exactly that
   combination, so `launch_ready` is true without touching anything.
   — `pinned_launch_fills_the_sheet_each_time_it_opens`
4. Changing the form after opening does not change the pin; the next opening
   is the pin again. — `pinned_launch_fills_the_sheet_each_time_it_opens`
5. A pinned account that no longer exists opens as 「このマシン」; a pinned
   setting that sanitising changes on load loses its acknowledgement.
   — `pinned_launch_drops_what_no_longer_holds`
6. Unchecking clears the pin; the sheet then behaves as before this change.
   — `pinned_launch_fills_the_sheet_each_time_it_opens`

## Behaviour

- Footer, below the acknowledgement checkbox when shown: checkbox
  「この設定を毎回使う」. Checked = a pin exists.
- Pinned and the form differs from the pin: small weak note
  「固定した設定から変更しています。次回は固定した設定で開きます。」 and quiet
  button 「今の設定で固定し直す」, which re-pins the current form.
- Save failure: `eprintln!` like the existing recent-settings save; the pin still
  holds for this run. The file is local and small (existing
  `RECENT_AGENT_SETTINGS_FILE_MAX_BYTES` ceiling); too large or unparsable loads
  as no pin, as today.
- The pinned agent unavailable on this Mac: the sheet opens on it anyway and the
  existing 「選んだ AI が見つかりません」 reason shows.

## Design

Placement: **EXTEND** `src/agents/settings.rs` — `git grep RecentAgentSettings`
shows the remembered launch options live there; the pin is one more of them.

- `src/agents/settings.rs`: `PinnedLaunch { agent, settings: AgentLaunchSettings,
  account: Option<Uuid>, acknowledged: bool }`; `RecentAgentSettings` gains
  `#[serde(default)] pinned: Option<PinnedLaunch>`. `load_recent_agent_settings`
  sanitises it: unknown agent → no pin; sanitising that changes the settings →
  `acknowledged = false`.
- `src/app.rs`: `pin_launch_settings`, `unpin_launch_settings`,
  `apply_pinned_launch` (called from `open_project_session_setup`), and
  `launch_matches_pin` — a field-by-field borrow compare, no allocation, since
  the footer calls it each frame.
- `src/app/screens.rs`: the checkbox, note and button in `ui_launch_footer`.
- `src/i18n_tables.rs`: three message ids in every table.
- Persisted shape: `recent-agent-settings.json` only, an additive optional key.
  Not the store; no `STORE_SCHEMA_VERSION` bump. No external command, no
  background work.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | existing checkbox/weak/quiet_button styling only (`.claude/rules/palette-and-glyphs.md` read) |
| Icons | no | no glyph |
| Identifier SSOT | no | no string shared across two places; the file name constant already exists |
| Durability | yes, light | the settings file is written by the existing `write_file_atomically`; old files load via `serde(default)`; not the store |
| Subprocess safety | no | no child; the launch still goes through `build_agent_launch_command` and its gates |
| Documentation | no | no user doc describes the launch sheet footer |
| Local-first | yes | local file only, no `unsafe` |
| Budgets | yes | existing `RECENT_AGENT_SETTINGS_FILE_MAX_BYTES` and the 1024/4096 field caps apply to the pin |

## Flagged concerns

- **The pin carries the danger acknowledgement** — weakens a per-launch protection.
  Answered by the person on 2026-10-09 (chose 「了承も固定する」). Bounded: the
  warning text still shows, and the acknowledgement covers only the exact pinned
  combination (`pending_launch` equality).

## Acceptance

- `cargo test --locked pinned_launch` passes the three tests above.
- In the app: pin a combination, close and reopen the sheet (and restart the
  app); it opens with that combination and ⌘↩ launches.

## Rejected alternatives

- Skip the sheet and launch directly — the person chose pinning.
- Pin per project — nothing in the request asks for it; one pin is simpler.
