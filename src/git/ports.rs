//! What is listening, and which worktree it belongs to.
//!
//! Running the same task in several worktrees at once is what this application
//! is for, and it is what fills a machine with dev servers on 5173, 5174, and
//! 5175 handed out in start order. Every listening socket has a process, every
//! process has a working directory, and that directory is inside exactly one
//! worktree. This module is the join.
//!
//! A child of `src/git.rs` for what it answers rather than for what it runs:
//! the question is "which worktree owns this", and a worktree is a git idea
//! that the parent already holds. The scan itself is not git, and if a better
//! parent appears this should move — said here so the next reader knows it was
//! a judgement rather than an accident. A top-level module would be a line in
//! `src/main.rs`, which `docs/sdlc/risk.yaml` holds at `paused`.

use crate::*;

/// One listening socket, already in a form a person can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListeningPort {
    pub(crate) pid: u32,
    pub(crate) process: String,
    pub(crate) port: u16,
    /// The host to connect to, not the host that was bound: a wildcard bind is
    /// `localhost` by the time it gets here, because `*` is not something a
    /// browser can open.
    pub(crate) host: String,
}

/// A folder a listener can be credited to: a session's worktree, one of a
/// project's git worktrees, or a project's root (change 261).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PortRoot {
    /// Canonical, because that is the form `lsof` reports a working directory
    /// in and `git worktree list` reports a worktree in. A stored path spelled
    /// through a symlink would otherwise contain nothing `lsof` ever names.
    pub(crate) path: PathBuf,
    /// Every stored path that resolved to `path`, and `path` itself. A session
    /// is matched against these in the frame, which may not touch the
    /// filesystem to canonicalise its `worktree_path` there.
    pub(crate) spellings: Vec<PathBuf>,
    pub(crate) project_id: Uuid,
    /// How the list names the folder: empty for the project's root, the path
    /// relative to the root when inside it, the folder's own name when not.
    pub(crate) label: String,
}

/// One listener, with where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttributedPort {
    pub(crate) port: ListeningPort,
    /// The process's working directory, when `lsof` could read it. Shown in
    /// the confirmation before the process is stopped.
    pub(crate) directory: Option<PathBuf>,
    /// Shared, because the frame clones a session's ports every time it draws
    /// them and the folder is the same for every port in it.
    pub(crate) root: Option<Arc<PortRoot>>,
}

/// Read `lsof`'s field-per-line output.
///
/// `-F` exists so that nobody has to parse columns, and a command name with a
/// space in it is why that matters. A process block opens with `p<pid>` and
/// carries `c<command>`; the files under it each open with `f<fd>` and are
/// described by `d<device>` and `n<address>`. The address forms seen on this
/// machine are `*:5173`, `127.0.0.1:5173`, and `[::1]:5173`.
///
/// A process that binds the same port on both stacks prints it twice. That is
/// one listener: the identity is the process and the port, not the socket.
pub(crate) fn parse_lsof_listeners(output: &str) -> Vec<ListeningPort> {
    let mut ports: Vec<ListeningPort> = Vec::new();
    let mut pid = None;
    let mut process = String::new();
    for line in output.lines() {
        let Some((field, value)) = line.split_at_checked(1) else {
            continue;
        };
        match field {
            "p" => {
                pid = value.trim().parse::<u32>().ok();
                process.clear();
            }
            "c" => process = value.trim().to_owned(),
            "n" => {
                let (Some(pid), Some(port)) = (pid, parse_listening_port(value.trim())) else {
                    continue;
                };
                if ports
                    .iter()
                    .any(|seen| seen.pid == pid && seen.port == port.0)
                {
                    continue;
                }
                if ports.len() >= PORT_SCAN_MAX_ENTRIES {
                    break;
                }
                ports.push(ListeningPort {
                    pid,
                    process: process.clone(),
                    port: port.0,
                    host: port.1,
                });
            }
            _ => {}
        }
    }
    ports
}

/// Split an address into its port and the host to connect to.
///
/// The port is what follows the last colon, because an IPv6 address is full of
/// them. A wildcard or an unspecified address becomes `localhost`: `*` and `::`
/// say what was bound, and this is what will be opened.
fn parse_listening_port(address: &str) -> Option<(u16, String)> {
    let (host, port) = address.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let host = match host {
        "*" | "" | "::" | "0.0.0.0" => "localhost",
        other => other,
    };
    Some((port, host.to_owned()))
}

/// Read `lsof -a -p <pids> -d cwd -Fn`: a `p<pid>` block whose one file is the
/// working directory.
pub(crate) fn parse_lsof_working_directories(output: &str) -> HashMap<u32, PathBuf> {
    let mut directories = HashMap::new();
    let mut pid = None;
    for line in output.lines() {
        let Some((field, value)) = line.split_at_checked(1) else {
            continue;
        };
        match field {
            "p" => pid = value.trim().parse::<u32>().ok(),
            "n" => {
                if let Some(pid) = pid {
                    directories.insert(pid, PathBuf::from(value.trim()));
                }
            }
            _ => {}
        }
    }
    directories
}

/// Every folder a listener could be credited to, once per canonical path.
///
/// Projects first, so that a project's root and its git worktrees are named
/// relative to that root; then sessions, whose folders are usually one of
/// those already and only add a spelling. Both lookups are parameters so that
/// a test can say what `git worktree list` and the filesystem answer.
pub(crate) fn port_roots(
    projects: &[(Uuid, PathBuf)],
    sessions: &[(Uuid, PathBuf)],
    worktrees_of: impl Fn(&Path) -> Vec<PathBuf>,
    canonical: impl Fn(&Path) -> PathBuf,
) -> Vec<Arc<PortRoot>> {
    let mut roots: Vec<PortRoot> = Vec::new();
    let mut add = |project_id: Uuid, project_root: Option<&Path>, stored: &Path| {
        let path = canonical(stored);
        if let Some(root) = roots.iter_mut().find(|root| root.path == path) {
            if !root.spellings.iter().any(|spelling| spelling == stored) {
                root.spellings.push(stored.to_path_buf());
            }
            return;
        }
        let label = port_root_label(project_root, &path);
        let mut spellings = vec![path.clone()];
        if stored != path {
            spellings.push(stored.to_path_buf());
        }
        roots.push(PortRoot {
            path,
            spellings,
            project_id,
            label,
        });
    };
    for (project_id, project) in projects {
        let root = canonical(project);
        add(*project_id, Some(&root), project);
        for worktree in worktrees_of(project) {
            add(*project_id, Some(&root), &worktree);
        }
    }
    for (project_id, worktree) in sessions {
        let root = projects
            .iter()
            .find(|(id, _)| id == project_id)
            .map(|(_, project)| canonical(project));
        add(*project_id, root.as_deref(), worktree);
    }
    roots.into_iter().map(Arc::new).collect()
}

/// Empty for the root itself, which the list names in words; relative inside
/// the root; the folder's own name for a worktree living elsewhere, which is
/// where `worktree_destination` puts the ones Operon creates.
fn port_root_label(project_root: Option<&Path>, path: &Path) -> String {
    if let Some(root) = project_root {
        if path == root {
            return String::new();
        }
        if let Ok(relative) = path.strip_prefix(root) {
            return relative.display().to_string();
        }
    }
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Give each listener the folder its process is working in.
///
/// The **deepest** containing folder wins. A worktree living inside its own
/// project would otherwise be claimed by both, and the project — which contains
/// everything — would claim every port on the machine that happened to be
/// started underneath it. Since change 261 the project's git worktrees are
/// folders too, whether or not a session is open on them, so a session at the
/// root no longer collects a worktree's servers by default.
pub(crate) fn attribute_ports(
    ports: &[ListeningPort],
    directories: &HashMap<u32, PathBuf>,
    roots: &[Arc<PortRoot>],
) -> Vec<AttributedPort> {
    ports
        .iter()
        .map(|port| {
            let directory = directories.get(&port.pid).cloned();
            let root = directory.as_ref().and_then(|directory| {
                roots
                    .iter()
                    .filter(|root| directory.starts_with(&root.path))
                    .max_by_key(|root| root.path.components().count())
                    .cloned()
            });
            AttributedPort {
                port: port.clone(),
                directory,
                root,
            }
        })
        .collect()
}

/// 443 and 8443 are the two ports a person means `https` by. Everything else is
/// guessed as `http`, which is what a dev server is, and a wrong guess costs a
/// redirect rather than a mystery.
pub(crate) fn port_url(port: &ListeningPort) -> String {
    let scheme = if matches!(port.port, 443 | 8443) {
        "https"
    } else {
        "http"
    };
    format!("{scheme}://{}:{}/", port.host, port.port)
}

/// One reading of what is listening, attributed.
///
/// Returns a reading rather than a `Result` because every way this can fail —
/// no `lsof`, no permission, a timeout — means the same thing to every caller:
/// nothing is known. A `Result` here would be a decision each caller made
/// separately, and they would not make it the same way.
pub(crate) fn scan_listening_ports(
    projects: &[(Uuid, PathBuf)],
    sessions: &[(Uuid, PathBuf)],
) -> Vec<AttributedPort> {
    scan_listening_ports_with(PORT_SCAN_PROGRAM, projects, sessions)
}

/// The reading, with the program named.
///
/// The name is a parameter so that a test can point it at something that is not
/// there and watch the whole path answer "nothing known". The alternative — a
/// test that empties `PATH` — reaches every other test running beside it, and
/// did: it took out the launch test that needs to find `tmux`.
///
/// The projects' git worktrees are listed only once something is listening:
/// a machine serving nothing costs one `lsof` and no `git`.
pub(crate) fn scan_listening_ports_with(
    program: &str,
    projects: &[(Uuid, PathBuf)],
    sessions: &[(Uuid, PathBuf)],
) -> Vec<AttributedPort> {
    let scan_timeout = Duration::from_secs(PORT_SCAN_TIMEOUT_SECONDS);
    let Some(listing) = lsof_output(
        program,
        &["-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcnd"],
        scan_timeout,
    ) else {
        return Vec::new();
    };
    let ports = parse_lsof_listeners(&listing);
    if ports.is_empty() {
        return Vec::new();
    }
    let mut pids: Vec<String> = ports.iter().map(|port| port.pid.to_string()).collect();
    pids.sort();
    pids.dedup();
    let mut arguments = vec!["-a".to_owned()];
    for pid in &pids {
        arguments.push("-p".to_owned());
        arguments.push(pid.clone());
    }
    arguments.extend(["-d".to_owned(), "cwd".to_owned(), "-Fn".to_owned()]);
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let directories = lsof_output(program, &borrowed, scan_timeout)
        .map(|output| parse_lsof_working_directories(&output))
        .unwrap_or_default();
    // A project whose listing fails still contributes its root.
    let roots = port_roots(
        projects,
        sessions,
        |project| {
            list_worktrees(project)
                .map(|worktrees| {
                    worktrees
                        .into_iter()
                        .map(|worktree| worktree.path)
                        .collect()
                })
                .unwrap_or_default()
        },
        |path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
    );
    attribute_ports(&ports, &directories, &roots)
}

fn lsof_run(program: &str, arguments: &[&str], timeout: Duration) -> Option<Output> {
    let mut command = Command::new(program);
    command.args(arguments);
    run_command_with_output_limit(
        &mut command,
        timeout,
        PORT_SCAN_MAX_BYTES,
        COMMAND_ERROR_MAX_BYTES,
    )
    .ok()
    .map(|limited| limited.output)
}

fn lsof_output(program: &str, arguments: &[&str], timeout: Duration) -> Option<String> {
    // `lsof` exits non-zero when some of what it was asked about has gone away,
    // which is normal here and not a reason to throw away what it did print.
    lsof_run(program, arguments, timeout)
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// What became of a request to stop a listener (change 261).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PortStopOutcome {
    /// Signalled, and a successful read then showed the port free.
    Stopped,
    /// Signalled, and every successful read until the deadline still saw it.
    StillListening,
    /// Signalled, and no read after the signal succeeded.
    Unconfirmed,
    /// The PID no longer listens on that port under that name. Nothing sent.
    NotListening,
    /// The listener could not be read before signalling. Nothing sent.
    Unverified,
    /// PID 0, PID 1, a number no process can have, or Operon itself.
    Refused,
    /// `kill` refused, with what it said.
    Failed(String),
}

/// Who holds `port.port` on `port.pid`, as one read of `lsof` sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListenerRead {
    /// The PID listens on the port under the name the person confirmed.
    Same,
    /// The PID listens on the port under another name. Before the signal that
    /// is not the confirmed listener; after it, the port is still not free.
    Renamed,
    /// The PID does not listen on the port.
    Free,
}

/// Read who holds `port.port` on `port.pid`.
///
/// `None` when the answer is not known: `lsof` could not run, timed out, or
/// found nothing while saying something on stderr — an absence it complained
/// about may be its failure rather than the port's, and "stopped" is never
/// reported on the strength of one.
pub(crate) fn read_listener(program: &str, port: &ListeningPort) -> Option<ListenerRead> {
    let pid = port.pid.to_string();
    let address = format!("-iTCP:{}", port.port);
    let output = lsof_run(
        program,
        &[
            "-nP",
            "-a",
            "-p",
            &pid,
            &address,
            "-sTCP:LISTEN",
            "-F",
            "pcn",
        ],
        Duration::from_secs(PORT_STOP_CHECK_TIMEOUT_SECONDS),
    )?;
    let listeners = parse_lsof_listeners(&String::from_utf8_lossy(&output.stdout));
    let mut holders = listeners
        .iter()
        .filter(|seen| seen.pid == port.pid && seen.port == port.port);
    if let Some(first) = holders.next() {
        let same =
            first.process == port.process || holders.any(|seen| seen.process == port.process);
        return Some(if same {
            ListenerRead::Same
        } else {
            ListenerRead::Renamed
        });
    }
    // `lsof` exits 1 and prints nothing when nothing matched.
    let quiet = output.stderr.iter().all(u8::is_ascii_whitespace);
    (output.status.success() || quiet).then_some(ListenerRead::Free)
}

/// Ask `pid` to exit with SIGTERM. The PID is a number formatted here, after
/// `--`, so nothing that reaches `kill` can be read as an option.
pub(crate) fn send_terminate(program: &str, pid: u32) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(["-s", "TERM", "--", &pid.to_string()]);
    let output = run_command_with_timeout(
        &mut command,
        Duration::from_secs(PORT_STOP_SIGNAL_TIMEOUT_SECONDS),
    )
    .map_err(|error| error.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(if said.is_empty() {
        output.status.to_string()
    } else {
        said
    })
}

/// Stop the listener the person confirmed, and only that one.
pub(crate) fn stop_listening_port(port: &ListeningPort) -> PortStopOutcome {
    stop_listening_port_with(
        port,
        std::process::id(),
        Duration::from_millis(PORT_STOP_WAIT_MILLIS),
        |port| read_listener(PORT_SCAN_PROGRAM, port),
        |pid| send_terminate(PORT_STOP_PROGRAM, pid),
    )
}

/// The sequence, with the read and the signal named so that a test can drive
/// it without a process.
///
/// The listener is read again immediately before the signal: the list the
/// person confirmed from may be thirty seconds old, and a PID freed since then
/// can belong to anything. Same PID, same port, same name, or nothing is sent.
/// What remains is a PID reused under the same name in the milliseconds
/// between the read and the signal. After the signal only a free port counts
/// as stopped: a PID still holding it under another name has not stopped.
pub(crate) fn stop_listening_port_with(
    port: &ListeningPort,
    operon_pid: u32,
    wait: Duration,
    mut read: impl FnMut(&ListeningPort) -> Option<ListenerRead>,
    signal: impl FnOnce(u32) -> Result<(), String>,
) -> PortStopOutcome {
    if port.pid <= 1 || port.pid > i32::MAX.unsigned_abs() || port.pid == operon_pid {
        return PortStopOutcome::Refused;
    }
    match read(port) {
        None => return PortStopOutcome::Unverified,
        Some(ListenerRead::Renamed | ListenerRead::Free) => return PortStopOutcome::NotListening,
        Some(ListenerRead::Same) => {}
    }
    if let Err(error) = signal(port.pid) {
        return PortStopOutcome::Failed(error);
    }
    let deadline = Instant::now() + wait;
    let mut seen_held = false;
    loop {
        thread::sleep(Duration::from_millis(PORT_STOP_POLL_MILLIS));
        match read(port) {
            Some(ListenerRead::Free) => return PortStopOutcome::Stopped,
            Some(ListenerRead::Same | ListenerRead::Renamed) => seen_held = true,
            None => {}
        }
        if Instant::now() >= deadline {
            break;
        }
    }
    if seen_held {
        PortStopOutcome::StillListening
    } else {
        PortStopOutcome::Unconfirmed
    }
}

/// One session's ports, split by where they run (change 261).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SessionPorts {
    /// Credited to the folder the session works in.
    pub(crate) own: Vec<AttributedPort>,
    /// Credited to another folder of the same project.
    pub(crate) elsewhere: Vec<AttributedPort>,
}

/// What the person is told when a stop finishes.
pub(crate) fn port_stop_notice(port: &ListeningPort, outcome: &PortStopOutcome) -> String {
    let process = port.process.as_str();
    let number = port.port;
    match outcome {
        PortStopOutcome::Stopped => tf!(
            "{process} を停止しました（:{port}）。",
            process = process,
            port = number
        ),
        PortStopOutcome::StillListening => tf!(
            "{process} に停止を求めましたが、まだ :{port} で動いています。",
            process = process,
            port = number
        ),
        PortStopOutcome::Unconfirmed => tf!(
            "{process} に停止を求めましたが、止まったか確かめられませんでした。",
            process = process
        ),
        PortStopOutcome::NotListening => tf!(
            ":{port} はもう {process} が使っていないため、何もしませんでした。",
            port = number,
            process = process
        ),
        PortStopOutcome::Unverified => {
            tr("ポートの持ち主を確かめられなかったため、停止しませんでした。").to_owned()
        }
        PortStopOutcome::Refused => tr("このプロセスは停止できません。").to_owned(),
        PortStopOutcome::Failed(error) => tf!(
            "{process} を停止できませんでした: {error}",
            process = process,
            error = error
        ),
    }
}
