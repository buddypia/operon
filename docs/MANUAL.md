English | [日本語](MANUAL.ja.md) | [한국어](MANUAL.ko.md)

# Operon User Manual

Operon is a local-first macOS app for running and supervising AI coding agents
(Claude Code, Codex CLI, Antigravity CLI, or a custom command). Each agent runs
in its own detached `tmux` session, so the work keeps going if you quit the app.

This manual covers day-to-day use. For the feature overview see
[README.md](../README.md); for contributing see [CONTRIBUTING.md](../CONTRIBUTING.md).

> **About label names.** Operon's interface is available in English, 日本語, and
> 한국어, but not every label has been translated yet. A label that is still
> untranslated appears in Japanese in every language. In this manual, a label
> shown in bold such as **New session** is what you will see in the English UI;
> an untranslated one is written as **停止** (Stop) — the Japanese text on screen
> followed by its meaning.

## Contents

1. [Installation](#1-installation)
2. [Core concepts](#2-core-concepts)
3. [Starting and managing agent sessions](#3-starting-and-managing-agent-sessions)
4. [Supervision, status, and notifications](#4-supervision-status-and-notifications)
5. [Git, diffs, files, and the editor](#5-git-diffs-files-and-the-editor)
6. [Cross-CLI conversation restore](#6-cross-cli-conversation-restore)
7. [Keyboard shortcuts](#7-keyboard-shortcuts)
8. [Settings and language](#8-settings-and-language)
9. [Data locations and privacy](#9-data-locations-and-privacy)
10. [Troubleshooting](#10-troubleshooting)
11. [Uninstalling and removing data](#11-uninstalling-and-removing-data)

---

## 1. Installation

### Requirements

| Requirement | Notes |
| --- | --- |
| macOS 12 or later | Apple Silicon or Intel (the packaged bundle declares 12.0 as its minimum) |
| Rust toolchain | Only to build from source; install with [rustup](https://rustup.rs/) |
| `tmux` | Required to run sessions. `brew install tmux` |
| At least one agent CLI | `claude`, `codex`, or `agy` (Antigravity) on `PATH` — or any custom command |
| `gh` (optional) | Enables the open pull request list in the **PR** tab |

### Build and run from source

```sh
git clone https://github.com/buddypia/operon.git
cd operon
cargo run --release
```

Quit any installed copy of Operon before running from source, and the other way
round (see [Single-instance lock](#10-troubleshooting)).

### Package a macOS app

```sh
bash scripts/package-macos.sh
```

The script first runs `cargo fmt --check`, `cargo test --locked`, and
`cargo clippy --locked -- -D warnings`, then builds with the locked dependency
set and writes `dist/Operon.app`. Copy that bundle to `/Applications` yourself
if you want it there; the script never touches `/Applications`.

The bundle is **ad-hoc signed**: it runs on the Mac that built it, but it is not
signed with a Developer ID or notarized. On another Mac, Gatekeeper will refuse
to open it at first; use *right-click → Open*, or *System Settings → Privacy &
Security → Open Anyway*. Distributing to other people properly requires an
Apple Developer ID certificate and notarization.

### First run

With no projects added, the **Project** page shows **Start by opening a project**
and a checklist, **Before starting your first session**, with two rows: `tmux`
and a coding agent (one or more of Codex, Claude Code, Antigravity). If either is
missing, **Open the setup steps** takes you to **Settings → Agent**, where each
tool is listed with its status (the header reads `n/6` available), a
**Recheck tools** button, and a **Setup** button that opens the vendor's
instructions.

Apps launched from Finder have a short `PATH`. Operon adds `/opt/homebrew/bin`,
`/usr/local/bin`, `~/.local/bin`, and `~/.cargo/bin` for itself, so tools
installed in those places are found.

---

## 2. Core concepts

- **Project** — any local folder. Git features appear when it is a Git
  checkout; starting agents and browsing files work anywhere. Adding a project
  never modifies it, and removing one deletes only Operon's record — files,
  branches, and worktrees stay.
- **Session** — one agent running in one `tmux` session, named `operon-` plus
  eight characters. A session belongs to a project and may run in a **worktree**.
- **Worktree** — an extra checkout of the same repository on its own branch, so
  several sessions can work in parallel without touching each other's files.
- **Agent** — the tool a session runs: **Claude Code**, **Codex CLI**,
  **Antigravity CLI (agy)**, or a custom command. (Internally Antigravity is
  stored under the older id `gemini`; the app and these docs call it Antigravity.)

The window has a toolbar and four pages:

- **Home** (⌘1) — what is running now, what is waiting for you, recent
  sessions, and a start button per project.
- **Project** (⌘2) — your projects. Opening one shows its tabs: **概要**
  (Overview), **Changes**, **worktree**, **PR**, **Files**, **Agent settings**.
- **Session** (⌘3) — the session list on the left and the selected session's
  terminal on the right.
- **Settings** (⌘,).

The toolbar holds **New session** (⌘N), **Search & actions** (⌘K), the Settings
button, and — when available — the Claude usage indicator.

---

## 3. Starting and managing agent sessions

### Add a project

On the **Project** page choose **Choose a project folder…** (⌘O), or drag a
folder from Finder into the window. **Scan a workspace…** imports every Git
repository found up to three levels below a folder. **パスを手入力** (Enter a
path manually) lets you type an absolute path.

### Start a session

Press **New session** (⌘N) from any page. A sheet opens over the current page:

1. **Agent** — Codex, Claude Code, Antigravity, or **別のコマンドを使う** (Use a
   different command). A green mark means the tool is available on this Mac.
   For a custom command, fill in **起動コマンド（必須）** (Launch command, required).
2. **Account** — only for Claude Code and Codex, and only if you have registered
   accounts (see [Settings](#8-settings-and-language)). The default is
   **This machine**.
3. **Request (optional)** — what you want the agent to do. It is typed into the
   agent as the first message.
4. **Where to work** — **Project root**, or a worktree (shown as
   **Worktree: branch**). Create worktrees in the project's **worktree** tab.
5. **More settings (optional)** — collapsed by default. It holds the session
   name, the model, the CLI's own permission mode, **Reasoning effort**, the
   switches that CLI accepts, and **ほかのセッションの後に開始** (Start after
   another session).

Press **<Agent> を起動 ⌘↩** (Start <Agent>, ⌘↩) to launch. **ターミナルだけ開く**
(Open a terminal only) starts a plain `tmux` shell with no agent. **Esc** or ✕
closes the sheet without launching.

Things the sheet enforces:

- Every mode, effort level, and switch comes from that CLI's own list; Operon
  invents no flags. Two switches the CLI refuses together cannot both be on.
- A switch or mode that lets the agent change files without asking is marked
  (危険, "dangerous"). The launch button stays disabled until you tick
  **理解したうえで起動する** (I understand — start anyway). This also applies to
  a custom command that contains such a flag.
- If `tmux` or the agent is missing, the sheet says so and offers a shortcut to
  **Settings → Agent**.
- Your last-used model, mode, effort, switches, and custom command are
  remembered per agent in `recent-agent-settings.json`.

### Queue sessions

Under **More settings**, **Start after another session** makes a session wait for
an earlier one. A queued session shows **今すぐ開始** (Start now) to run it
immediately. If the prerequisite fails, the queued session is marked blocked and
does not start by itself.

### Stop, retry, resume, close

The button in the corner of a session row and of the terminal header always
shows the one action that fits the state:

| State | Button |
| --- | --- |
| Running | **停止** (Stop). The record is kept. While a stop is still landing the button reads **停止中…**; pressing it again retries. |
| Queued | **今すぐ開始** (Start now) |
| Failed or lost, no saved conversation | **Retry** — runs the session again after checking and cleaning up the old terminal |
| Failed, lost, exited, or stopped, with a saved conversation | **会話 ID から再開** (Resume from conversation ID). Hover to see which model, mode, and switches carry over. |

Other actions:

- **名前を変更** (Rename): double-click a session's title, or use its `···` menu.
- **Mark unread**: in the `···` menu.
- **セッションを削除** (Delete session) — the ✕ on a row. A live terminal is
  stopped first; files and worktrees are never deleted.
- **ターミナルを閉じる** (Close terminal) removes the leftover terminal of a
  finished or failed session.
- The `···` menu in the terminal header offers **Terminal.app で開く**
  (open the `tmux` session in Terminal.app, for mouse use and `tmux` commands),
  **表示中のログをすべてコピー** (copy the whole buffer without colour codes), and
  **今すぐ同期** (refresh now).

### The terminal pane

Click the pane to type into the agent; input methods such as the Japanese IME
work. Scrollback is up to 5,000 lines in the app (`tmux` itself keeps 50,000).
**Latest** jumps to the newest line. The side panel (fold it with its arrow,
reopen with **Show side panel**) has three tabs: **Files**, **Conversation**
(the requests you have sent), and **Changes**.

Paths in terminal output are links: hold **⌥** and click to open the file in the
built-in editor at that line, or **⌘** and click to open it externally. Only
paths that resolve to a file inside the session's own folder are offered.
Addresses printed by a dev server open in the browser the same way. A plain
click only focuses the pane, so the agent never loses the keyboard by accident.

---

## 4. Supervision, status, and notifications

### Status

Every session carries one of four labels, with a hover that gives the detail:

| Label | Meaning |
| --- | --- |
| **Needs you** | Waiting for an answer, or stopped because a prerequisite failed |
| **Running** | Starting, working, stopping, or queued |
| **Idle** | Running and waiting for your next request |
| **Finished** | The terminal has ended (done, failed, stopped, or lost) |

The session list header has four toggle chips — **Needs you**, **Running**,
**Idle**, **Finished** — each with a live count. Turn some on to show only those
sessions; the list says how many it is hiding, and **フィルタを解除** (Clear
filter) turns the filter off. The filter is not remembered after a restart.

On **Home**, a banner counts the sessions waiting for you (waiting for an answer,
blocked, or finished unread) and opens the list already narrowed to them. A turn
that finishes while you are looking at another session leaves that session
marked until you open it; use **Mark unread** to mark one again. These marks do
not survive a restart.

### Where the status comes from

- **Hooks (recommended).** In **Settings → Agent**, **Detect state from the agent
  CLIs' hooks** registers a small managed entry in each CLI's own settings file —
  `~/.claude/settings.json`, `~/.codex/hooks.json` (and its trust entry in
  `~/.codex/config.toml`), and `~/.gemini/config/hooks.json` for Antigravity —
  plus in every account folder you registered. The CLI then reports working,
  waiting, and finished itself. Your existing entries are preserved, and turning
  the switch off removes exactly what Operon added. Each tool row shows
  **Hook installed**, **Hook not installed**, or **Hook failed**; hover for the
  reason.
- **Screen reading.** Without hooks, or when a hook report is older than 30
  minutes, Operon reads the bottom lines of the terminal to tell a working
  agent from one blocked on a question.

**Auto-approve the folder-trust prompt on first launch** (same section) answers
only the CLI's "do you trust this folder" question; it approves nothing else.

### Notifications

**Settings → Notifications → Notify on turn completion, input waits, and exits**
shows a macOS notification when an agent finishes a turn, stops on a prompt, or
exits. macOS asks for permission the first time you turn it on.

### Other indicators

- A session header shows what its own worktree is listening on, for example
  `:5173 vite`. Click to open it; right-click to copy the URL. Ports are read
  with `lsof` every 30 seconds.
- The toolbar shows how much of your Claude usage window is spent. Operon makes
  no request for it; it reads what Claude Code already reports. The number turns
  amber past 80% and fades, with its age, after 30 minutes without an update.

---

## 5. Git, diffs, files, and the editor

Open a project and use its tabs.

### Changes

The **Changes** tab has three views, **Changes**, **Diff**, and **History**.

- **Changes** lists changed files. Tick the files that belong in the next
  commit; tracked modifications start ticked and untracked files start unticked.
  Write a message or press **Have the AI write it** (a local agent CLI drafts it
  from the staged patch into the field — it never commits by itself), then
  **Commit N files**. The `push` row shows how the branch compares with its
  upstream; push refuses when the remote has moved on. Operon has no force push.
  Committing re-stages exactly the ticked files; if a hunk-level selection made
  with `git add -p` is in the index, it refuses rather than flatten it.
- **Diff** draws each file folded, numbered on both sides, with the changed words
  marked inside a modified line. **AI Review** asks a local agent CLI to review
  the diff (45-second limit); **Copy** copies the result.
- **History** shows recent commits.

### Notes on a diff

In **Diff**, click a line that exists after the change and write a note under
it (**この行について書く**, "Write about this line"; **保存**, Save). Notes are saved on
disk and survive a restart. The footer counts them (unresolved, sent, resolved)
and offers **Send them all to the session** (one message naming every file and
line, sent to the agent running in the project — refused while that agent is
mid-turn), **Copy Markdown**, **Clear resolved**, and **Clear all**. A note whose
line has since changed says **行が変わっています** (the line has changed) instead of
silently moving.

### Worktrees

The **worktree** tab creates a worktree from a new branch name. It starts from the
project's mainline (`origin/HEAD`, else `origin/main`, `origin/master`, `main`,
`master`) and says which one it used. If the name is taken, the next free
number is used (`name-2`, `name-3`, …). Each worktree offers **この場所で新規セッション**
(New session here) and **削除…** (Delete…); only clean, inactive worktrees are
removed, after a confirmation, and the branch is kept.

If the repository contains `.operon/setup.sh`, Operon shows its first lines after
creating a worktree and asks **Run this repository's setup?** **Run** starts it
in a session of its own, which you can watch and stop. **Do not ask again until
the contents change** remembers your approval for that exact script; editing it
asks again.

### Pull requests

The **PR** tab lists open GitHub pull requests with review and CI-check summaries.
It needs the GitHub CLI (`gh`) installed and signed in.

### Files and the editor

The **Files** tab has a folding project tree with a Git badge on every changed
file, editor tabs, line numbers, and syntax highlighting. **⌘S** saves. Markdown
renders as Markdown. Files over 128 KiB open read-only; images up to 10 MiB
preview. The tree skips `.git`, `node_modules`, `target`, and `.next`.

Operon never silently overwrites an agent's work: saving a file that changed on
disk after you opened it is refused. The open file is compared with the disk
every two seconds; with no unsaved edits it reloads and says so, and with unsaved
edits a bar offers the reload or the diff. The **Agent settings** tab lists the
project's skills and instruction files.

---

## 6. Cross-CLI conversation restore

A conversation from Claude Code, Codex CLI, or Antigravity can be restored into
any of the three — or into the same CLI again, which forks it without disturbing
the original. A restore reads the transcript files the CLIs already saved. **No
model runs and no tokens are spent**, and nothing is summarized.

**From a session**: open the session's `···` menu in the list and choose
**<Agent> へ履歴を復元** (Restore history to <Agent>), where the other CLIs are listed.

**From a past conversation**: on **Session**, press **History**, expand **CLI
sessions (find and restore past conversations)**, pick the project, and press
**ローカル CLI セッションを検出** (Detect local CLI sessions). Each card shows the
conversation's ID, branch, and last request. Then either:

- **元の CLI で再開** (Resume in the original CLI), or
- choose **復元先** (Restore to), optionally write **復元メモ（任意）** (Restore
  note, optional), and press **会話全履歴を復元** (Restore the full history).

A restore shows a window with the source, the destination, and elapsed time; you
can send it to the background. The same **History** page also has **ローカル履歴**
(local history) — a full-text search of recent Codex and Claude transcripts.

### The shared-session archive

Each restore keeps the original transcript verbatim in
`shared-sessions/<id>/source.jsonl`, with a `manifest.json` and a readable
`restored-conversation.md` beside it. Folders are created with mode `0700` and
snapshots `0600`, so only your account can read them. They may contain sensitive
prompts, tool output, and source code.

---

## 7. Keyboard shortcuts

| Keys | Action | Notes |
| --- | --- | --- |
| ⌘K | **Search & actions** | Arrow keys move, Enter opens. Finds sessions by title, request, agent, branch, or project; projects by name or path |
| ⌘N | **New session** | |
| ⌘1 / ⌘2 / ⌘3 | **Home** / **Project** / **Session** | ⌘2 returns to the project list |
| ⌘, | **Settings** | |
| ⌘O | Choose a project folder | |
| ⌘F | Find in the terminal | Enter = next, ⇧Enter = previous, Esc closes. Nothing typed reaches the agent |
| ⌘↩ | Start the session | In the new-session sheet |
| Esc | Close the new-session sheet | |
| ⌘S | Save the open file | In **Files** |
| ⌥ + click / ⌘ + click | Open a path or URL in terminal output | In the editor / externally |

To change a shortcut, open **Settings → Key bindings** and press
**Open the keymap file**. The first time, Operon writes `keybindings.json` from
the current keys and reveals it; it never overwrites an existing file. The format:

```json
{
  "version": 1,
  "keybindings": {
    "palette.open": "Mod+K",
    "terminal.find": "Mod+Shift+F",
    "page.settings": null
  }
}
```

`Mod` means ⌘; other modifiers are `Shift`, `Alt`, and `Ctrl`. `null` removes the
key. The actions are `palette.open`, `session.new`, `page.home`, `page.projects`,
`page.sessions`, `page.settings`, `project.add`, and `terminal.find`. A file that
gives one chord to two actions, or names an unknown action, is refused whole and
Operon starts with the defaults and tells you why.

---

## 8. Settings and language

**Settings** (⌘,) has five sections:

| Section | What it holds |
| --- | --- |
| **Appearance** | **Theme** (Dark by default, Light, and High Contrast (Dark) — applied immediately, including open terminals), **Terminal and code font** (four Sarasa faces), and **Language** |
| **Agent** | Tool status, the hook switch, folder-trust auto-approval, and **Agent accounts** |
| **Notifications** | The notification switch |
| **Data and recovery** | The data folder path, **データフォルダを表示** (show it in Finder), and **tmux をスキャン** (scan `tmux`) to adopt sessions the index lost |
| **Key bindings** | Every shortcut and its current key |

**Language**: choose English, 日本語, or 한국어. The change applies on the next
frame and is saved. On first launch Operon picks your system language
(`LC_ALL`, `LC_MESSAGES`, `LANG`, then macOS's language list), falling back to
Japanese.

**Agent accounts**: an account is the folder where Codex or Claude Code keeps its
login (`CODEX_HOME` / `CLAUDE_CONFIG_DIR`). Pick the agent, give it a name (for
example "Work"), and press **フォルダを選んで追加** (Choose a folder and add). It
then appears under **Account** when you start a session. Operon only points the
CLI at the folder — it never reads inside it. Managed hooks are installed there
too. Antigravity has no accounts.

---

## 9. Data locations and privacy

Everything is local. Operon has no telemetry, no accounts, and no cloud service.
Agents run as separate tools and may send project content to their own provider;
check each agent's permission and sandbox mode before launching.

Operon's data lives in `~/Library/Application Support/com.local.operon/`
(**Settings → Data and recovery** shows the exact path and opens it in Finder):

| Item | Purpose |
| --- | --- |
| `store-v2.json` | Projects, sessions, and preferences (schema-versioned; written atomically) |
| `store-v2.json.pending-cancellations.json` | Durable record of stops in progress |
| `recent-agent-settings.json` | Remembered launch options |
| `agent-accounts.json` | Registered account folders |
| `keybindings.json` | Your shortcuts, if you created it |
| `diff-comments.json` | Notes written on diffs |
| `setup-trust.json` | Approved `.operon/setup.sh` digests |
| `agent-hooks/` | Managed hook scripts, endpoint file, and the local `hook.sock` socket (mode `0700`) |
| `shared-sessions/` | The restore archive ([section 6](#6-cross-cli-conversation-restore)) |
| `.operon-instance.lock` | Single-instance lock |

Operon also reads (never uploads) the agents' own transcript folders such as
`~/.claude/projects` and `~/.codex` to find and restore conversations. Builds
from before the rename to Operon kept data in a `xirp-copy` folder; it is
imported on first launch.

---

## 10. Troubleshooting

**"`tmux` is required"** — Install it with `brew install tmux`, then press
**Recheck tools** in **Settings → Agent**.

**Agent not found / "選んだ AI が見つかりません"** — The CLI must be on `PATH`.
Operon searches your `PATH` plus `/opt/homebrew/bin`, `/usr/local/bin`,
`~/.local/bin`, and `~/.cargo/bin`. A CLI installed elsewhere (for example under
`nvm`) may be invisible to an app opened from Finder: start Operon from a shell
(`open -a Operon` from a terminal where the command works, or
`cargo run --release`), or symlink the CLI into `~/.local/bin`. Then press
**Recheck tools**.

**Single-instance lock** — Only one Operon may use the data folder. A second
launch shows **Operon is already open**; close the other copy and try again. This
includes an installed build and a `cargo run` build. If you see **Operon could
not open its local data**, check the folder's permissions and free disk space.

**Status never leaves "Running" or stays "Idle"** — Check that the hook shows
**Hook installed** for that CLI. If it shows **Hook failed**, hover for the reason.
Without hooks Operon falls back to reading the screen.

**A session shows Finished, and its hover says its terminal is gone** — Its
`tmux` session disappeared outside Operon, so the result cannot be confirmed
(Operon records this as "lost", not "failed"). Use **Retry** or **会話 ID から再開**
(Resume from conversation ID).

**Sessions exist in `tmux` but not in Operon** — **Settings → Data and
recovery → tmux をスキャン** lists them; **セッションを引き継ぐ** (Adopt the
session) re-attaches one to a registered project.

**Attach from a real terminal** — Sessions are on your default `tmux` server:
`tmux ls`, then `tmux attach -t operon-xxxxxxxx`.

**A keybindings file is ignored** — The notice names the problem (invalid JSON,
unknown action, duplicate chord). Fix it, or delete the file to return to the
defaults.

---

## 11. Uninstalling and removing data

1. **Turn the hook switch off first** (**Settings → Agent**). This removes the
   entries Operon added to each CLI's settings file. Also remove any account
   folders you registered if you want those entries gone.
2. Stop your sessions, then quit Operon. Sessions are plain `tmux` sessions; ones
   you leave running continue until you stop them (`tmux kill-session -t
   operon-xxxxxxxx`).
3. Delete the app: move `Operon.app` (for example from `/Applications` or this
   repository's `dist/`) to the Trash.
4. To delete Operon's data, move
   `~/Library/Application Support/com.local.operon/` to the Trash. This removes
   your project list, session records, notes, and the restore archive.
   Transcripts that Claude Code, Codex, or Antigravity wrote are theirs and are
   not touched.

Removing a project or deleting a session inside Operon never deletes files,
branches, or worktrees.
