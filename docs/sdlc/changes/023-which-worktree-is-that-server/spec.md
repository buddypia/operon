# Spec: a session says what its worktree is listening on

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Listening TCP sockets are read with `lsof -nP -iTCP -sTCP:LISTEN -F pcnd`,
   which is the field-per-line form and needs no column guessing.
2. Each listener is attributed to a worktree by the **working directory of the
   process that owns it**, read with `lsof -a -p <pids> -d cwd -Fn`. The
   deepest worktree whose path contains that directory wins, so a worktree
   inside a project is not also claimed by the project.
3. A process that binds the same port on IPv4 and IPv6 is one entry, not two.
   The identity of a listener is its process and its port.
4. A wildcard bind is shown as something a person can click:
   `*` becomes `localhost`. `443` and `8443` are `https`; everything else is
   `http`.
5. The scan runs in the background at most once every
   `PORT_SCAN_INTERVAL_SECONDS`, never on a draw path, and reads at most
   `PORT_SCAN_MAX_ENTRIES` listeners and `PORT_SCAN_MAX_BYTES` of output.
6. The session's terminal header shows one line: the ports of that session's
   worktree, each as `:<port> <process>`. Clicking one opens it in a browser.
7. A worktree with nothing listening draws no line. A machine where `lsof`
   cannot be run draws no line, and says nothing anywhere: this is information,
   not a task, and a permanent complaint about a missing reading is worse than
   the missing reading.
8. Nothing is killed, signalled, or connected to. The only outbound act is
   handing a URL to the system browser when a person clicks it.

## Behaviour

The screen was agreed before this was written:

```
┌─ セッション ────────────────────── 1180px ─┐
│ fix-login ・ operon-worktrees/fix-login       ▶ ⏹ ⋯│
│ :5173 vite   :3001 node                        │
├────────────────────────────────────────┤
│ $ pnpm dev                                      │
│   VITE ready in 412 ms                          │
│   → Local:   http://localhost:5173/             │
└────────────────────────────────────────┘
```

The states that are not the happy one:

- **Nothing listening.** No line. Most sessions, most of the time.
- **`lsof` missing or refusing.** No line, no notice, nothing in the log a
  person has to dismiss.
- **A session with no worktree path.** No line: there is nothing to attribute
  against.
- **More ports than fit the row.** The row wraps; a header two lines tall is
  better than a truncation that hides the port somebody wanted.
- **The scan is between runs.** The previous reading stays on screen. A port
  list that blanked every thirty seconds would be unreadable.

## Design

`src/git/ports.rs`, a child of `src/git.rs`. The parent is chosen for what the
module answers rather than for what it runs: the question is "which worktree
owns this", and a worktree is a git idea that `src/git.rs` already holds
(`list_worktrees`, `is_allowed_session_path`). The scan itself is not git, and
if a better parent appears this should move — said here so the next reader knows
it was a judgement and not an accident. A top-level module is a line in
`src/main.rs`, which `docs/sdlc/risk.yaml` holds at `paused`.

```rust
pub(crate) struct ListeningPort {
    pub(crate) pid: u32,
    pub(crate) process: String,
    pub(crate) port: u16,
    /// Already usable: a wildcard bind is `localhost` here.
    pub(crate) host: String,
}
pub(crate) fn parse_lsof_listeners(output: &str) -> Vec<ListeningPort>
pub(crate) fn parse_lsof_working_directories(output: &str) -> HashMap<u32, PathBuf>
pub(crate) fn attribute_ports(
    ports: &[ListeningPort],
    directories: &HashMap<u32, PathBuf>,
    worktrees: &[PathBuf],
) -> Vec<(ListeningPort, Option<PathBuf>)>
pub(crate) fn port_url(port: &ListeningPort) -> String
pub(crate) fn scan_listening_ports(worktrees: &[PathBuf]) -> Vec<(ListeningPort, Option<PathBuf>)>
```

The two parsers are pure and take the text `lsof` actually prints, which was
captured from this machine before either was written: a process block opens with
`p<pid>`, carries `c<command>`, and then a set of files each opened by `f<fd>`
and described by `d<device>` and `n<address>`. The address forms are `*:5173`,
`127.0.0.1:5173`, and `[::1]:5173`.

`scan_listening_ports` returns a reading rather than a `Result`: every failure
mode here — no `lsof`, no permission, a timeout — means "nothing known", and a
caller that has to decide what an error means would decide it differently in two
places.

`src/app.rs` gains `listening_ports: Vec<(ListeningPort, Option<PathBuf>)>`,
`ports_scanned_at: Option<Instant>`, and a `BackgroundKey::PortScan`. The scan is
requested from the frame loop when the interval has elapsed, beside the session
poll that already works that way.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The port chips use `palette.text_muted` and the existing link treatment. No new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The row is `:port process`, which is the whole label. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The interval, the entry ceiling, the byte ceiling, and the scan timeout are `src/config.rs` constants. The `lsof` argument lists exist once each. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Nothing is persisted. A port reading is true for thirty seconds and is worth nothing after a restart. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Two `lsof` invocations through `run_command_with_output_limit`, with a timeout and a byte ceiling. Arguments are vectors. Nothing is signalled or killed. The pid list passed to the second call is built from the first call's own output, and every entry is a `u32` that was parsed as one. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Reading the local process table. The only outbound act is opening a URL a person clicked. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `PORT_SCAN_MAX_ENTRIES`, `PORT_SCAN_MAX_BYTES`, `PORT_SCAN_TIMEOUT_SECONDS`, `PORT_SCAN_INTERVAL_SECONDS`. |

## Flagged concerns

- **`lsof` is slow on a busy machine.** Bounded by a timeout and run in the
  background at an interval; a scan that times out is a reading of nothing and
  the previous one stays on screen.
- **A port could be attributed to the wrong worktree.** Only when one worktree's
  path contains another's, which is why the deepest match wins and why that rule
  has a test of its own.

## Acceptance

- `cargo test --locked` passes, including:
  - `reads_the_pid_command_and_port_out_of_what_lsof_prints`
  - `counts_one_listener_when_a_process_binds_both_stacks`
  - `turns_a_wildcard_bind_into_something_clickable`
  - `attributes_a_port_to_the_deepest_worktree_that_contains_it`
  - `leaves_a_port_unattributed_when_no_worktree_contains_it`
  - `reads_a_working_directory_for_every_process_lsof_named`
  - `a_missing_lsof_is_a_reading_of_nothing_rather_than_an_error`
- In the running app: start a server in a worktree, and the session's header
  names its port within the scan interval; clicking it opens the browser.

## Rejected alternatives

- **A project-wide Ports tab.** The other option on the agreed screen. It shows
  more and is somewhere you have to go; the question "what is this session
  serving" is asked while looking at the session.
- **Parse `lsof`'s default columns.** `-F` exists precisely so that nobody has
  to, and a command name with a space in it breaks column parsing.
- **`netstat`.** It does not give the process's working directory, which is the
  entire join.
- **Kill a process from the row.** A different kind of action, needing a
  confirmation and an owner check. Its own change.
- **Scan on every frame, or on demand only.** The first is work proportional to
  the process table on a draw path; the second means the answer is stale exactly
  when it is looked at.
