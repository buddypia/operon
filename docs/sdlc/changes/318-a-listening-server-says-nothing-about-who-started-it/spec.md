# Spec: Who started each listening server, when, and with what

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Each listener the port scan returns carries, when `ps` could read it, its
   parent PID, its elapsed running time in seconds, its start as a local wall
   clock `MM/DD HH:MM`, and its command line. A field that does not parse
   (an unknown month, a malformed `etime`) is absent rather than guessed, and
   the line `ps` was cut in by its output ceiling gives no details at all.
   Pinned by `ports_318_reads_process_details_from_ps` (including a cut last
   line and an unparsable `lstart`) and `ports_318_elapsed_time_forms`.
2. Each listener carries what its environment says about the Operon session
   that started it: the value of `HOOK_SESSION_ENV` (`Found`) or no such
   variable (`Absent`) — both only from a whole line for that PID — and
   otherwise `Unread`: `ps` failed, its output was cut at the ceiling inside or
   before this PID's line, or the PID was missing from it (gone since `lsof`).
   Pinned by `ports_318_reads_the_session_tag`.
3. A command line never reaches the screen with a secret in it: values of
   `NAME=value` arguments whose name contains `KEY`, `TOKEN`, `SECRET`,
   `PASSWORD`, `PASSWD`, `AUTH`, `CREDENTIAL` or `COOKIE` (any case); the value
   after a `--flag` or in `--flag=value` whose flag name contains one of those
   words; the word after `Bearer` or `Basic`, including inside one argument
   (`Authorization:Bearer x`); URL user information (`scheme://user:pass@`);
   and arguments starting with a known token prefix (`sk-`, `ghp_`, `gho_`,
   `ghs_`, `github_pat_`, `glpat-`, `xoxb-`, `xoxp-`, `AKIA`, `AIza`) are each
   shown as `***`. The command is cut to `PORT_COMMAND_MAX_CHARS` characters
   after redaction, ending in `…` when cut. Pinned by
   `ports_318_command_line_hides_secrets`.
4. A listener's origin, resolved against the store when it is shown: the session
   whose `tmux_name` equals the tag (`Session`, with its name and whether its
   status is `Starting` or `Active`), a tag naming no session in the store
   (`RemovedSession`), `Absent` (`Unknown`), or `Unread` (`Unread`). A listener
   is **leftover** — 「終了忘れ?」 — exactly when its origin is a session that is
   not `Starting`/`Active`, or `RemovedSession`. `Unknown` and `Unread` are never
   leftover. Pinned by `ports_318_origin_names_the_session_and_marks_leftovers`.
5. The ports menu shows, under each row, the origin, 「親は終了済み」 when the
   parent PID is 1, the start and how long ago, and the command line; a leftover
   row shows 「終了忘れ?」 with its reason on hover. Pinned by
   `ports_318_menu_rows_show_origin_time_and_command`.
6. The session header's badge gains 「⚠ 終了忘れ? {n}」 in `palette.warning` when
   `n` > 0 leftovers are among the ports its menu lists (own folder and the rest
   of the project). Pinned by `ports_318_badge_counts_leftovers`.
7. The stop confirmation lists 起動したセッション, 起動時刻, コマンド, フォルダ,
   PID and 親プロセス; a fact that could not be read says 「不明」. Pinned by
   `ports_318_confirmation_lists_every_detail`.
8. When `ps` cannot run, the scan still returns every listener as today, with no
   details and origin `Unread`; when it can, it runs `ps` exactly twice per
   scan whatever the number of listeners. Pinned by
   `ports_318_scan_without_ps_still_lists_ports`.

## Behaviour

Strings (all through `tr` / `tf!`, rows in every table of `src/i18n_tables.rs`):

| Where | Text |
|---|---|
| origin, session running | 「{name}」 |
| origin, session not running | 「{name}」(停止中) |
| origin, tag names a removed session | 削除済みのセッション |
| origin, no tag | 起動元不明 |
| origin, tag not read | 起動元を読み取れませんでした |
| parent PID 1 | 親は終了済み |
| start | {start} · {relative} (e.g. `11/24 19:30 · 14時間前`, `relative_time`) |
| leftover mark | 終了忘れ? |
| leftover hover, stopped | 起動したセッションは停止しています |
| leftover hover, removed | 起動したセッションは削除されています |
| badge segment | 終了忘れ? {n} |
| modal labels | 起動したセッション · 起動時刻 · コマンド · フォルダ · PID · 親プロセス |
| modal parent | 終了済み (PID {ppid}) when 1, otherwise PID {ppid} |
| unread fact | 不明 |

- Detail line 1 joins origin, parent-gone, and start with ` · `; a part that was
  not read is left out. Detail line 2 is the redacted command, truncated to the
  row width; hover shows the full redacted command and the folder.
- The modal's start line is `{start} ({relative})`. The start is shown as
  `MM/DD HH:MM` always — screen.md's `10:02` for today is not drawn, because the
  crate has no local date for "today" (see Rejected alternatives).
- Empty: no listeners — nothing drawn, as today. Error: `ps` failing or timing
  out — rows as today with origin 起動元を読み取れませんでした and no time or
  command. Unparsable: a field that does not parse is left out of the line.
  Too large: `ps -E` output over `PORT_ENVIRONMENT_MAX_BYTES` — the PIDs whose
  lines were not read whole are `Unread`; `ps` details over
  `PORT_SCAN_MAX_BYTES` — the cut line's PID has no details. Gone: a PID that
  exited between `lsof` and `ps` has no details and is `Unread`. Interrupted: a scan that
  outlives a stop is discarded by the existing generation check.

## Design

Placement verdict: **EXTEND** — `src/git/ports.rs` (change 262, feature
`worktrees`, its spec items 12–14) owns the scan and attribution; the drawing is
`ui_session_ports` / `session_ports_menu` / `ui_port_stop_modal` in
`src/app/screens.rs`. Route `modify`.

- `src/git/ports.rs`
  - `ListenerProcess` with optional `parent_pid`, `elapsed_seconds`, `started`
    and `command`, and a `session_tag: SessionTag`;
    `enum SessionTag { Found(String), Absent, Unread }`.
  - `AttributedPort` gains an optional, `Arc`-shared `process`.
  - `parse_ps_details` (details keyed by PID), `parse_elapsed`
    (`[[dd-]hh:]mm:ss` to seconds), `parse_ps_environment` (the output and
    whether it was cut, to a `SessionTag` keyed by PID), and
    `redact_command_line`.
  - `scan_listening_ports_with` gains the `ps` program as a parameter; after
    attribution it runs, once each:
    `ps -ww -o pid=,ppid=,etime=,lstart=,command= -p PID,PID,…` with
    `LC_ALL=C` (so `lstart` is `Tue Nov 24 19:30:42 2026`), ceiling
    `PORT_SCAN_MAX_BYTES`; and `ps -E -ww -o pid=,command= -p PID,PID,…`, ceiling
    `PORT_ENVIRONMENT_MAX_BYTES`, timeout `PORT_SCAN_TIMEOUT_SECONDS` for both,
    through `run_command_with_output_limit`. The environment output is searched
    for the **last** whitespace token starting `{HOOK_SESSION_ENV}=` on each
    PID's line and nothing else from it is kept.
  Each read runs once per scan with every listening PID in one `-p` list, never
  once per PID.
- `src/config.rs`: `PORT_PROCESS_PROGRAM = "/bin/ps"`,
  `PORT_ENVIRONMENT_MAX_BYTES = 4 MiB`, `PORT_COMMAND_MAX_CHARS = 1000`.
- `src/app.rs`: `enum PortOrigin { Session { name, running }, RemovedSession,
  Unknown, Unread }` with `is_leftover()`; `OperonApp::port_origin(&self,
  &AttributedPort) -> PortOrigin` (a linear search of `store.sessions`);
  `SessionPorts` gains `origins`, a `PortOrigin` per PID, filled by
  `session_ports`, and `leftovers() -> usize`.
- `src/app/screens.rs`: the badge segment, two detail lines per row in
  `port_menu_rows`, and the modal grid rows. The modal works from
  `pending_port_stop`, which is not a `SessionPorts` entry, so it calls
  `port_origin` itself when drawn. No child process or filesystem call is added
  to a draw path. What is per frame, and accepted: `session_ports` already runs
  each frame the header draws (change 262); it now also resolves one origin per
  listed PID by a linear search of `store.sessions` and clones that session's
  name; the menu, only while open, formats one detail line and one
  `relative_time` per row. Redaction, parsing and the start string are done in
  the scan, so the frame never holds a raw command line.
- Background: both `ps` runs happen inside the existing `PortScan` background
  task, every `PORT_SCAN_INTERVAL_SECONDS`, and only when something listens.
  Scans never overlap: `request_port_scan` returns while a `PortScan` task is in
  flight, so a scan that takes up to four 8 s timeouts delays the next one
  rather than racing it.
- No persisted shape changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes | `palette.warning` and `palette.text_muted`, existing roles; no new role. |
| Icons | yes | `ICON_ATTENTION`, existing; no new constant. |
| Identifier SSOT | yes | The tag name is `HOOK_SESSION_ENV`, built into the search, never retyped; the test builds the fixture from the constant and from `managed_tmux_name`'s output (round trip). |
| Durability | yes | No store change. New budget `PORT_ENVIRONMENT_MAX_BYTES` in `src/config.rs`, enforced while reading by `run_command_with_output_limit`; a cut line is reported as `Unread`, never as `Absent` (rule 2); README states the ceilings. |
| Subprocess safety | yes | Both `ps` runs go through `run_command_with_output_limit` with a timeout; absolute `/bin/ps`; PIDs are integers formatted by Operon; no launch path changes. |
| Documentation | yes | `README.md`, `README.ja.md`, `README.ko.md` port paragraph together; `docs/features/worktrees/spec.md` and `brief.md`; `docs/features/glossary.yaml` adds 終了忘れ. |
| Local-first | yes | No network, no new `unsafe`, no dependency; the environment of other processes is read in memory, only the tag kept. |
| Budgets | yes | `ps` details: `PORT_SCAN_MAX_BYTES` (512 KiB); environments: 4 MiB; at most `PORT_SCAN_MAX_ENTRIES` (200) PIDs; command 1000 characters; timeouts 8 s each, sequential, never overlapping with the next scan (Design, Background). |

## Flagged concerns

- **The tag is not on every child.** Measured: MCP servers launched by Codex and
  Antigravity did not inherit it. Answered: those show 起動元不明 and are never
  marked leftover; the mark is only claimed from a positive tag.
- **Parent PID 1 is also every launchd service.** Answered: only listeners inside
  a project folder are listed, where PID 1 means an orphan in practice; the
  text says 親は終了済み, and the modal shows the PID.
- **Redaction is a pattern list, not a guarantee.** Answered: the patterns cover
  what was measured on this machine and common token shapes; the requirement
  names each, and the test pins each.

## Acceptance

- `cargo test --locked ports_318` and `cargo test --locked ports_262` pass;
  `make q.fast` passes.
- In the built app, with a dev server started by an agent session and then that
  session stopped, the menu row names the session with (停止中), shows its start
  time and command, and is marked 終了忘れ?; the header shows 終了忘れ? 1.

## Rejected alternatives

- Walking parent PIDs to a tmux pane: loses the answer the moment the parent
  exits, which is the forgotten-server case.
- `sysctl(KERN_PROCARGS2)` through `libc`: exact argv/env, but new `unsafe` — a
  paused surface — for what `ps` already answers.
- Recording origins in the store: the person chose not to record.
- A wall clock "today" (`10:02`): needs the local UTC offset, which the crate
  does not have without a dependency or `unsafe` (`time_until`'s note).
- Notifying on a leftover: the person chose the mark only.
