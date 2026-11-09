# Spec: A session's ports, split by folder, and stoppable

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A listener is credited to the **deepest** known folder containing its
   process's working directory, where the known folders are every session's
   worktree, every git worktree of every project, and every project's root;
   the credit carries the folder's project and the process's working
   directory, and a listener whose directory is unknown or outside every known
   folder is credited to none. Test: `ports_262_deepest_known_folder_is_credited`.
2. The known folders are compared by their canonical path — the form `lsof`
   reports working directories in and `git worktree list` reports worktrees
   in — and listed once per canonical path, remembering every stored spelling
   that resolved to it. Each is named for the list: empty for a project's
   root, the path relative to the project root when the folder is inside it,
   the folder's own name when it is not. Test:
   `ports_262_known_folders_are_named_once`.
3. A session's ports split in two: **own** — credited to the folder the
   session works in, which is the folder its stored worktree path is a
   spelling of, or the project root when it has no worktree (it then runs in
   the project checkout) — and **elsewhere** — credited to another folder of the same
   project. A session at a project's root does not count a server in one of
   that project's git worktrees as own, whether or not a session is open on it;
   another project's servers appear in neither. Own is in port order,
   elsewhere in folder-name then port order. Tests:
   `ports_262_session_splits_own_from_elsewhere`,
   `ports_262_session_without_worktree_owns_the_project_root`.
4. The header badge names the own ports as today (`:5173 vite` for one,
   「{n} ポート稼働中」 for more); with no own ports but some elsewhere it reads
   「プロジェクトで {n} ポート稼働中」 in `text-muted`; with neither nothing is
   drawn. Tests: `ports_262_badge_counts_own_folder_or_names_the_project`,
   `ui_session_ports_renders_compact_popover_button_when_active`.
5. The menu body draws 「この作業フォルダ」 over the own rows and
   「プロジェクト内のほかの場所」 over the elsewhere rows, omits a section that is
   empty, gives each elsewhere row its folder name (「プロジェクトのルート」 for
   the root), and gives every row an open, a copy and a stop control; pressing
   stop asks for confirmation and sends nothing. Test:
   `ports_262_menu_sections_name_folders_and_offer_stop`.
6. Stopping refuses PID 0, PID 1, a PID above `i32::MAX` and Operon's own PID
   without running any program. Test: `ports_262_stop_refuses_operon_and_init`.
7. Stopping signals only the listener the person confirmed: when the PID no
   longer listens on that port under that process name, nothing is sent and
   the process keeps running; when it does, SIGTERM is sent and the outcome is
   "stopped" once a successful read shows the port no longer held by that PID
   under any name — a PID that keeps the port under a new name has not
   stopped (`ports_262_stop_does_not_take_a_renamed_holder_for_gone`).
   Tests: `ports_262_stop_signals_only_the_confirmed_listener` (a real
   `python3` listener, real `lsof` and `/bin/kill`) and
   `ports_262_stop_sequence_checks_before_signalling` (injected check and
   signal).
8. "Stopped" is never reported without a successful read showing the port
   free: when the listener cannot be verified before signalling (no `lsof`,
   a timeout), nothing is sent and the outcome says so; when `kill` exits
   non-zero, the outcome carries its error; when every read after the signal
   fails, the outcome is "asked, but could not confirm". The whole stop is
   bounded by a deadline. Test:
   `ports_262_stop_never_reports_stopped_without_seeing_it_gone`.
9. The confirmation window names the port, process, PID and the process's
   working directory (「不明」 when unknown); 「キャンセル」 and the close box
   clear it without starting a stop. Test:
   `ports_262_confirmation_names_pid_and_folder`.
10. A stop's outcome reaches the person as the notice for that outcome, and the
    ports are read again on the next frame rather than up to thirty seconds
    later. A scan that was already running when the outcome arrived is
    discarded when it lands and read again, so it cannot put a stopped server
    back in the list. Test: `ports_262_stop_outcome_is_told_and_rescanned`.

## Behaviour

Header badge (unsplit session header only, as today):

- Own ports: `↗ :5173 vite` or `↗ 3 ポート稼働中`, `info` ink.
- No own, some elsewhere: `↗ プロジェクトで 2 ポート稼働中`, `text-muted` ink.
- Neither, or the machine could not be read: no badge, no separator.
- Hover on the badge: 「稼働中のローカルポート一覧を表示」 (unchanged).

Menu (opens under the badge, scrolls past 240px):

- Section headings, small and weak with a separator under each:
  「この作業フォルダ」, 「プロジェクト内のほかの場所」. An empty section is not drawn.
- Row: `[:5173 vite]` opens the URL (hover 「ブラウザで開く: {url}」); elsewhere
  rows then show the folder name, muted and truncated, full path on hover;
  `⧉` copies (hover 「URL をコピー」, notice 「URL をコピーしました。」); `■`
  (`ICON_STOP`, hover 「このプロセスを停止…」) closes the menu and opens the
  confirmation. While a stop is in flight every `■` is disabled with hover
  「停止しています…」.
- A URL failing the existing http/https and control-character check is not
  listed (unchanged).

Confirmation window 「ポートを停止」, centred, 440px wide, not resizable:

- 「:{port} {process} を停止しますか？」 (strong)
- 「PID」 `48213`; 「フォルダ」 the process's working directory, or 「不明」.
- 「終了シグナル (SIGTERM) を送ります。Operon の外で起動したプロセスも含まれます。」
- 「キャンセル」 and the close box dismiss it; 「停止する」 (`danger` ink) starts the
  stop in the background and closes the window.

Outcome notices (`notice`):

- Stopped: 「{process} を停止しました（:{port}）。」
- Still held after the wait: 「{process} に停止を求めましたが、まだ :{port} で動いています。」
- Signalled, but no read after it succeeded: 「{process} に停止を求めましたが、止まったか確かめられませんでした。」
- No longer that listener: 「:{port} はもう {process} が使っていないため、何もしませんでした。」
- Could not verify: 「ポートの持ち主を確かめられなかったため、停止しませんでした。」
- Refused: 「このプロセスは停止できません。」
- Signal failed: 「{process} を停止できませんでした: {error}」

Interrupted: quitting during a stop leaves the signal sent or not sent; nothing
is persisted. A listener whose process was replaced between the scan and the
confirmation is caught by requirement 7.

## Design

Placement: `EXTEND`. `src/git/ports.rs` already joins listeners to folders
(`attribute_ports`, tests from `ports_` in change 077) and `src/app.rs` already
draws the badge (`ui_session_ports`, test
`ui_session_ports_renders_compact_popover_button_when_active`). Both grow; no
module is added. The badge's drawing moves into `src/app/screens.rs` with the
other modals, to keep `src/app.rs` under its size band.

`src/git/ports.rs`:

- `PortRoot { path, spellings: Vec<PathBuf>, project_id: Uuid, label: String }`
  — a known folder: `path` is canonical, `spellings` every stored path that
  resolved to it (and `path` itself).
- `AttributedPort { port: ListeningPort, directory: Option<PathBuf>, root: Option<Arc<PortRoot>> }`
  replaces `(ListeningPort, Option<PathBuf>)`.
- `port_roots(projects: &[(Uuid, PathBuf)], sessions: &[(Uuid, PathBuf)], worktrees_of: impl Fn(&Path) -> Vec<PathBuf>, canonical: impl Fn(&Path) -> PathBuf) -> Vec<Arc<PortRoot>>`
  — requirement 2; pure, so a test injects both the git listing and the
  canonicalisation. The scan passes `list_worktrees` and
  `Path::canonicalize` (falling back to the path itself), both on the
  background thread. Labels are computed between canonical paths.
- `attribute_ports(ports, directories, roots: &[Arc<PortRoot>]) -> Vec<AttributedPort>`
  — requirement 1, same deepest-wins rule as today.
- `scan_listening_ports(projects, sessions)` / `scan_listening_ports_with(program, projects, sessions)`:
  the two existing `lsof` reads, then — only when a listener was found —
  `list_worktrees` for each project (existing; `git worktree list --porcelain`
  through `run_command_with_timeout`, 20 s, `COMMAND_OUTPUT_MAX_BYTES`), then
  attribution. A project whose listing fails contributes only its root.
- `PortStopOutcome { Stopped, StillListening, Unconfirmed, NotListening, Unverified, Refused, Failed(String) }`.
- `ListenerRead { Same, Renamed, Free }` and
  `read_listener(program, port) -> Option<ListenerRead>`:
  `lsof -nP -a -p <pid> -iTCP:<port> -sTCP:LISTEN -F pcn`, parsed by the
  existing `parse_lsof_listeners`; `Same` when the PID holds the port under
  the confirmed name, `Renamed` when it holds it under another, `Free` when
  `lsof` ran and the PID does not hold it (exit 0, or exit 1 with nothing on
  stderr), `None` when it could not run, timed out, or found nothing while
  writing to stderr. `lsof_output` gains a timeout parameter: the scan
  keeps `PORT_SCAN_TIMEOUT_SECONDS`, this check uses
  `PORT_STOP_CHECK_TIMEOUT_SECONDS` = 2; both keep `PORT_SCAN_MAX_BYTES`.
- `send_terminate(program, pid) -> Result<(), String>`:
  `/bin/kill -s TERM -- <pid>` through `run_command_with_timeout`
  (`PORT_STOP_SIGNAL_TIMEOUT_SECONDS` = 2); the error is `kill`'s stderr.
  `PORT_STOP_PROGRAM` = `"/bin/kill"`, an absolute path, not a `PATH` lookup.
- `stop_listening_port(port)` calls
  `stop_listening_port_with(port, std::process::id(), wait, check, signal)`
  with the two functions above; the `_with` form takes the check and the signal
  as closures so the sequence is tested without processes. Order: refuse PID
  ≤ 1, PID > `i32::MAX` or Operon's PID; read — `None` → `Unverified`,
  `Renamed` or `Free` → `NotListening`; signal — `Err` → `Failed`; then read
  every `PORT_STOP_POLL_MILLIS` = 250 until `PORT_STOP_WAIT_MILLIS` = 3000 has
  passed: the first `Free` → `Stopped`; at the deadline `StillListening` if
  any read saw `Same` or `Renamed`, else `Unconfirmed`. Worst case: 2 s check + 2 s
  signal + 3 s wait + one 2 s read overrunning it ≈ 9 s, with `PortStop` held.
  A PID reused between the check and the signal, under the same name, in that
  window of milliseconds, is the residual risk; the name match is what narrows
  it. No `unsafe`, no `libc::kill`.

`src/app.rs`:

- `listening_ports: Vec<AttributedPort>` and `ports_generation: u64` (new
  fields on `OperonApp`, not persisted); `BackgroundResult::ListeningPorts`
  becomes `{ generation, ports }`. `request_port_scan` passes
  `(project.id, project.path)` for every project and
  `(session.project_id, worktree)` for every session with a worktree; with
  neither it clears as today.
- `session_ports(&self, &Session) -> SessionPorts` — requirement 3, own when
  the root's `spellings` contain the session's stored worktree path, or the
  stored project path when the session has none (a lookup in
  `store.projects`, no filesystem call in the frame); the same small
  clone-and-sort per frame as today, over at most `PORT_SCAN_MAX_ENTRIES`.
  `stop_port` spawns the stop.

`src/app/screens.rs`, beside the other modals, which keeps `src/app.rs` under
the `largest_module_lines` band's propose tier (10 000):

- `ui_session_ports` draws the badge (requirement 4) and calls
  `session_ports_menu(ui, palette, &SessionPorts, stopping) -> Option<PortMenuAction>`
  (requirement 5), which returns what was pressed instead of mutating state, so
  it can be drawn and read in a test. The stop control is the existing
  `small_icon_button` inside `ui.add_enabled_ui(!stopping, …)`, with
  `on_disabled_hover_text` — no new button helper and no raw button.
- `ui_port_stop_modal(ctx, palette)`, called beside `ui_run_script_modal`
  (requirement 9), reading `pending_port_stop: Option<AttributedPort>` on
  `OperonApp`.

`src/app.rs` again:

- `BackgroundKey::PortStop` (one stop at a time) and
  `BackgroundResult::PortStopped { port, outcome }`; its handler removes the
  key, sets the notice, bumps `ports_generation`, clears `ports_scanned_at`,
  and on `Stopped` drops that one (pid, port) from the list at once
  (requirement 10). `BackgroundResult::ListeningPorts`
  carries the generation its scan started at; a result from an older
  generation is dropped and `ports_scanned_at` cleared, so the next frame
  reads again.

Nothing new runs in the draw path: the scan, the git listing, both `lsof`
checks and `kill` run on the background thread; the frame reads
`listening_ports` and `pending_port_stop`. No persisted shape changes.

`src/git/ports.rs` also holds `SessionPorts { own, elsewhere }` and
`port_stop_notice(port, outcome)`, the words for each outcome.

`src/config.rs`: the five `PORT_STOP_*` constants. `src/i18n_tables.rs`: every
new string in `EN_TABLE` and `KO_TABLE`; 「稼働中のポート」 removed with its last
use. `README.md`, `README.ja.md`, `README.ko.md`: the ports paragraph.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | Existing roles only: `info`, `text-muted`, `danger`, read off `Palette`. No literal, no new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Existing `ICON_STOP`, `ICON_COPY`, `ICON_OPEN_EXTERNAL`; no new glyph. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | `PORT_STOP_PROGRAM` and the timing constants live in `src/config.rs` beside `PORT_SCAN_PROGRAM`; the tests pass program names through the `_with` functions rather than retyping them. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Nothing persisted, no store, cancellation path, tmux stop or migration touched; the scan keeps its existing byte and entry ceilings, and the new reads state theirs below. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | `lsof`, `kill` and `git` all through `run_command_with_output_limit` / `run_command_with_timeout` with timeouts; arguments are numbers formatted from `u32`/`u16`, never shell text. No launch path, so the agent gates do not apply. The person confirms every signal, and the PID, port and name are re-verified immediately before it. |
| Documentation — user-facing docs change in all three languages together | yes | The ports paragraph in `README.md`, `README.ja.md`, `README.ko.md` changes together. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Local processes only; SIGTERM through the `kill` program, not `libc::kill`. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | Verification `lsof`: `PORT_SCAN_MAX_BYTES`, `PORT_SCAN_MAX_ENTRIES`, 2 s each, the whole stop under a ≈9 s deadline. `kill`: `COMMAND_OUTPUT_MAX_BYTES`, 2 s. `git worktree list`: existing 20 s and `COMMAND_OUTPUT_MAX_BYTES`, once per project per scan and only when something listens. |

## Flagged concerns

- **A `git worktree list` per project every 30 s.** Answered: it runs only when
  at least one listener exists, in the background thread that already runs two
  `lsof` reads, and costs milliseconds per project; caching it on the existing
  `worktree_cache` would make attribution depend on whether the person had
  opened the worktree tab.
- **SIGTERM may be ignored.** Answered: the outcome says the server is still
  running; forcing is out of scope (intent).
- **A scan in flight when a stop finishes.** A periodic scan that started
  before the signal lands afterwards with the old listing and would put the
  stopped server back for up to thirty seconds. Answered: each scan carries
  the generation it started at, a stop outcome bumps the generation, and a
  result from an older generation is dropped and read again on the next frame
  (requirement 10).
- **Symlinked or otherwise uncanonical stored paths.** Answered: roots are
  compared canonically and remember their stored spellings (requirements 2
  and 3).

## Acceptance

- `cargo test --locked ports_262_` and
  `cargo test --locked ui_session_ports_renders_compact_popover_button_when_active`
  pass; the existing `ports_` tests pass against the new signatures.
- `make q.fast` passes.
- In the running app, with a dev server in a worktree that has no session and
  one in the root: the root session's badge names only the root's server; its
  menu lists the worktree's server under 「プロジェクト内のほかの場所」 with its
  folder; stopping it after confirmation removes it from the list.

## Rejected alternatives

- A persisted scope toggle: offered to the person, who chose the two sections;
  it would also change the store's shape.
- `libc::kill`: a new `unsafe` block, banded at propose, for no gain over `kill`.
- Killing the process group or escalating to SIGKILL: can take down processes
  the person did not see in the list.
- Reusing `worktree_cache`: filled only when the worktree tab is opened, so the
  same server would move between sections depending on what the person viewed.
