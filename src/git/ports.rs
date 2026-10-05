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

/// Give each listener the worktree its process is working in.
///
/// The **deepest** containing worktree wins. A worktree living inside its own
/// project would otherwise be claimed by both, and the project — which contains
/// everything — would claim every port on the machine that happened to be
/// started underneath it.
pub(crate) fn attribute_ports(
    ports: &[ListeningPort],
    directories: &HashMap<u32, PathBuf>,
    worktrees: &[PathBuf],
) -> Vec<(ListeningPort, Option<PathBuf>)> {
    ports
        .iter()
        .map(|port| {
            let owner = directories.get(&port.pid).and_then(|directory| {
                worktrees
                    .iter()
                    .filter(|worktree| directory.starts_with(worktree))
                    .max_by_key(|worktree| worktree.components().count())
                    .cloned()
            });
            (port.clone(), owner)
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
pub(crate) fn scan_listening_ports(worktrees: &[PathBuf]) -> Vec<(ListeningPort, Option<PathBuf>)> {
    scan_listening_ports_with(PORT_SCAN_PROGRAM, worktrees)
}

/// The reading, with the program named.
///
/// The name is a parameter so that a test can point it at something that is not
/// there and watch the whole path answer "nothing known". The alternative — a
/// test that empties `PATH` — reaches every other test running beside it, and
/// did: it took out the launch test that needs to find `tmux`.
pub(crate) fn scan_listening_ports_with(
    program: &str,
    worktrees: &[PathBuf],
) -> Vec<(ListeningPort, Option<PathBuf>)> {
    let Some(listing) = lsof_output(program, &["-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcnd"])
    else {
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
    let directories = lsof_output(program, &borrowed)
        .map(|output| parse_lsof_working_directories(&output))
        .unwrap_or_default();
    attribute_ports(&ports, &directories, worktrees)
}

fn lsof_output(program: &str, arguments: &[&str]) -> Option<String> {
    let mut command = Command::new(program);
    command.args(arguments);
    let limited = run_command_with_output_limit(
        &mut command,
        Duration::from_secs(PORT_SCAN_TIMEOUT_SECONDS),
        PORT_SCAN_MAX_BYTES,
        COMMAND_ERROR_MAX_BYTES,
    )
    .ok()?;
    // `lsof` exits non-zero when some of what it was asked about has gone away,
    // which is normal here and not a reason to throw away what it did print.
    Some(String::from_utf8_lossy(&limited.output.stdout).into_owned())
}
