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
6. [Cross-CLI handoff](#6-cross-cli-handoff)
7. [Knowledge](#7-knowledge)
8. [Keyboard shortcuts](#8-keyboard-shortcuts)
9. [Settings and language](#9-settings-and-language)
10. [Data locations and privacy](#10-data-locations-and-privacy)
11. [Troubleshooting](#11-troubleshooting)
12. [Uninstalling and removing data](#12-uninstalling-and-removing-data)

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
round (see [Single-instance lock](#11-troubleshooting)).

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
- **Project** (⌘2) — your projects. Opening one shows its tabs: **Overview**,
  **Changes**, **worktree**, **PR**, **Files**, **Agent settings**, and
  **Knowledge**.
- **Session** (⌘3) — three columns. On the left, the projects with their live
  sessions, the status tiles, and **History**. In the middle, tab groups holding
  session terminals and files; drag a tab to an edge to split, up to four
  groups. On the right, the side panel (**Files**, **Conversation**,
  **Changes**). **History** swaps the middle for past sessions and
  conversations.
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
   accounts (see [Settings](#9-settings-and-language)). The default is
   **This machine**.
3. **Request (optional)** — what you want the agent to do. It is typed into the
   agent as the first message. **Insert template** adds a built-in request
   (review this diff, fix the CI error, resolve merge conflicts, add tests) or
   one of the repository's `.operon/prompts/*.md`. **Fetch from GitHub Issue…**
   lists open issues through `gh`; choosing one fills the request and the
   session name. Typing `@` offers `@diff`, `@knowledge:<title>`, and project
   files; they go in as text for the agent to read, and Operon does not expand
   them.
4. **Where to work** — **Project root**, or a worktree (shown as
   **Worktree: branch**). Create worktrees in the project's **worktree** tab.
5. **More settings (optional)** — collapsed by default. It holds the session
   name, the model, the CLI's own permission mode, **Reasoning effort**, the
   switches that CLI accepts, and **ほかのセッションの後に開始** (Start after
   another session).

**Run multiple agents in parallel (compare in separate worktrees)** gives the
same request to two or three agents at once, each in a new worktree and branch
of its own, so you can compare what they did.

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
- Closing a session's tab (its ✕, or ⌘W) only takes it off the screen; the
  agent keeps running and the session stays in the list. ⇧⌘T reopens it.
- The `···` menu in the terminal header offers **Terminal.app で開く**
  (open the `tmux` session in Terminal.app, for mouse use and `tmux` commands),
  **表示中のログをすべてコピー** (copy the whole buffer without colour codes), and
  **今すぐ同期** (refresh now). It also holds **Open in external editor**,
  **Send recent error to agent**, and — where they apply —
  **Run script (.operon/run.sh)…**, **Checkpoints**, **Create PR…**, and
  **Merge worktree into main…** ([section 5](#5-git-diffs-files-and-the-editor)).

### The terminal pane

Each session tab starts with its state mark; the **+** right after the last tab
asks what to start — a terminal, Claude Code, Codex CLI or Antigravity — and
⌘T opens a plain terminal directly. ⇧⌘] and ⇧⌘[, or ⌃Tab and ⌃⇧Tab, move to the next
and previous tab. A
group can be split by dragging a tab to an edge of the workspace — up to four
groups, each at least 320 × 200 points — and a divider resizes them. Each
project keeps its own tabs and layout when you switch.

Click the pane to type into the agent; input methods such as the Japanese IME
work. Scrollback is up to 5,000 lines in the app (`tmux` itself keeps 50,000).
**Latest** jumps to the newest line. The side panel opens to the right of the
terminal (fold it with its arrow or ⌥⌘B, reopen with **Show side panel** or
⌥⌘B) and has three tabs: **Files**, **Conversation**
(the requests you have sent), and **Changes**. Each tab, and the session
list's heading, has a grip (⠿): drag it to the left, right, top, or bottom edge
of the workspace to dock that panel there on its own, or right-click the grip
to choose an edge; **Change panel placement** does the same from a button.
Releasing in the middle or on the panel's own edge, or pressing Escape, cancels.
An edge the window is too small to show the panel on is outlined in amber and
refuses it; widen the window to use it.

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
| **Idle** | Finished its turn; ready for your next request |
| **Finished** | The terminal has ended (done, failed, stopped, or lost) |

The **Session** page lists each project's live sessions in the order you act on
them: waiting on you first, then running, then idle. Finished sessions move to
**History** once you have seen them.

The session sidebar's header has four toggle chips — **Needs you**, **Running**,
**Idle**, **Finished** — each with a live count, and a segment below them that
narrows by CLI (Codex, Claude Code, Antigravity, others). Turn some on to show
only those sessions; the column says how many it shows, and **Clear** turns the
filter off. The same chips filter the list in **History**, where **Clear the
filter** does the same. The filter is not remembered after a restart.

Sessions that need you are pinned under **要対応** (Needs you) at the top of the
list, longest wait first, each with how long it has waited and — when its agent
has finished a turn — what the agent last said, or 許可または回答を待っています
(waiting for a permission or an answer). Click one, or press ⌘J, to open it with
the keyboard already in its terminal; the toolbar's **要対応 N** button does the
same while any session waits. Once answered, it returns to its project group.
The Dock icon carries the number of waiting sessions as a badge, so it is in
view while another application is in front; the badge goes when nobody waits.

A waiting row whose terminal is running also has a one-line reply field. Type
there and press Enter: the words, then Enter, are typed into that session's
terminal, and the screen stays where it was — 「はい」 answers a question, 「2」
picks a prompt's second option. Open the session instead when the question
needs reading. A reply left half typed is dropped when the session stops
waiting.

A session whose agent is working says for how long once a minute has passed:
実行中 · 12分 in the heading of its open pane and on its row in the per-project
session list, and the same time when you hover its state on a tab or in the
project column. The time is the current turn's — it starts again when the agent
stops and works again — and it is not kept across a restart.

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

**Settings → Notifications → Notify when a turn finishes, input is needed, or a session exits**
shows a macOS notification when an agent finishes a turn, stops on a prompt, or
exits. macOS asks for permission the first time you turn it on.

### Other indicators

- A session header shows what its own folder is listening on, for example
  `:5173 vite`, or **{n} ports active** when there are several. Each listening
  process belongs to the deepest folder containing its working directory: a
  session's worktree, any git worktree of the project, or the project root.
  When only other places in the project are listening, the header says
  **{n} ports active in this project** in a muted colour. Click it for a menu
  with **This working folder** and **Elsewhere in this project**; each row
  opens or copies its URL or stops the server. Stopping asks first
  (**Stop port**, showing the process, PID, and folder), checks that the same
  process still owns the port, sends SIGTERM, and says whether it stopped.
  Ports are read with `lsof` every 30 seconds.
- The toolbar shows how much of your Claude usage window is spent. Operon makes
  no request for it; it reads what Claude Code already reports. The number turns
  amber past 80% and fades, with its age, after 30 minutes without an update.

---

## 5. Git, diffs, files, and the editor

Open a project and use its tabs.

### Changes

The **Changes** tab has three views, **Changed files**, **Diff**, and
**Commit history**.

- **Changed files** lists changed files on the left and the selected file's diff
  on the right. Tick the files that belong in the next
  commit; tracked modifications start ticked and untracked files start unticked.
  Write a message or press **Have the AI write it** (a local agent CLI drafts it
  from the staged patch into the field — it never commits by itself), then
  **Commit N files**. The `push` row shows how the branch compares with its
  upstream; push refuses when the remote has moved on. Operon has no force push.
  Committing re-stages exactly the ticked files; if a hunk-level selection made
  with `git add -p` is in the index, it refuses rather than flatten it.
- **Diff** draws each file folded, numbered on both sides, with the changed words
  marked inside a modified line. Every hunk header carries **Stage** and
  **Discard**, which act on that hunk of the project's checkout and nothing else
  in the file. **Discard does not ask first.** **AI review** asks a local agent
  CLI to review the diff (45-second limit); **Copy** copies the result.
- **Commit history** shows recent commits.

**Create PR…**, beside push (and in a worktree session's terminal menu), opens
a title, a description, and a draft switch, then runs `gh pr create`. **Draft
with AI** pre-fills the description from the start of the diff. The branch must
already be pushed; Operon never pushes behind your back.

### Checkpoints

In a worktree session with the status hooks on, every request you send records a
snapshot of the worktree under `refs/operon/checkpoints/`, without touching the
index. **Create manual checkpoint** in the terminal menu's **Checkpoints** takes
one by hand. The same menu lists the latest five: copy the diff since one, or
**Roll back to checkpoint**. A rollback restores the tracked files at once and
leaves files created since then in place.

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

**Merge into main…** on a worktree (or **Merge worktree into main…** in a
session's terminal menu) merges its branch into the mainline locally with
`git merge --no-ff`. It refuses while either tree has uncommitted changes, while
a session is running or queued in that worktree or the main checkout, or when the
branch has nothing to merge. A conflict rolls the merge back and names the files.
Afterwards the worktree and the merged branch are removed (`git branch -d`)
unless you untick that.

If the repository contains `.operon/setup.sh`, Operon shows its first lines after
creating a worktree and asks **Run this repository's setup?** **Run** starts it
in a session of its own, which you can watch and stop. **Do not ask again until
the contents change** remembers your approval for that exact script; editing it
asks again.

A repository that checks in `.operon/run.sh` (a dev server or a build) gets
**Run script (.operon/run.sh)…** in the terminal menu. Each run shows the script
and asks (**Review run script**); the file is read again on approval and must
still match what you saw. The approved copy runs in a terminal of its own, and
**Stop the run script** in the same menu stops it.

### Pull requests

The **PR** tab lists open GitHub pull requests with review and CI-check summaries.
It needs the GitHub CLI (`gh`) installed and signed in.

### Files and the editor

The **Files** tab has a folding project tree with a Git badge on every changed
file, editor tabs, line numbers, and syntax highlighting. **⌘S** saves. Markdown
renders as Markdown. Files up to 128 KiB can be edited; larger ones up to 8 MiB
open read-only in a viewer that draws only the visible lines, and larger files
are not opened. Images up to 10 MiB preview. The tree skips `.git`,
`node_modules`, `target`, and `.next`.

Operon never silently overwrites an agent's work: saving a file that changed on
disk after you opened it is refused. The open file is compared with the disk
every two seconds; with no unsaved edits it reloads and says so, and with unsaved
edits a bar offers the reload or the diff. The **Agent settings** tab lists the
project's skills and instruction files.

From the editor:

- **Open in external editor** opens the file, a worktree, or the project folder
  in VS Code, Cursor, Zed, Windsurf, Antigravity, Void, or Finder, picked from a
  menu.
- **Send to agent** sends an excerpt of the open file to the newest agent running
  in the project, or puts it in a new session's request when none is.
- **Send recent error to agent**, in a terminal's menu, finds the last error or
  stack trace in that pane's recent output and types it into the same pane.
- ⌘K also lists the selected project's files and the symbols (`fn`, `struct`,
  `enum`, `trait`, `interface`, `class`, `type`) of files open in the editor,
  and opens them at that line.

---

## 6. Cross-CLI handoff

A conversation from Claude Code, Codex CLI, or Antigravity can be handed off to
any of the three — or to the same CLI again, which starts a second conversation
there without disturbing the original. Operon reads the transcript files the
CLIs already saved and builds a compact **Handoff bundle** from them by rule.
**No model runs and no tokens are spent** building it, and the destination CLI's
own store is never modified; the destination starts a fresh session of its own
with the bundle as its first message.

The bundle holds the first request, up to 8 recent User / Assistant turns (each
at most 1,200 characters), and the last 3 tool results — a test run, a build log
— each shortened by rule to at most 1,000 characters, with errors and failures
kept. The whole bundle stays under 6,000 bytes. A tool result is never presented
as something the user or the assistant said, and reasoning is never carried.

**From a session**: open the session's `···` menu in the list and choose
**Hand off to <Agent>**. When a model or reasoning effort is set for that
destination, the entry names it.

**From a past conversation**: on **Session**, press **History**, expand **CLI
sessions (find and restore past conversations)**, pick the project, and press
**ローカル CLI セッションを検出** (Detect local CLI sessions). Each card shows the
conversation's ID, branch, and last request. Then either:

- **元の CLI で再開** (Resume in the original CLI), or
- choose **復元先** (destination) — the same CLI is listed as
  **<Agent>（同じ CLI に複製）** (copy into the same CLI) — optionally pick a
  model and reasoning effort for it (remembered per CLI), write
  **Handoff note (optional)**, and press **Hand off with bundle**.

A handoff shows a window with the source, the destination, and elapsed time; you
can send it to the background. The same **History** page also has **ローカル履歴**
(local history) — a full-text search of recent Codex and Claude transcripts.

### The shared-session archive

Each handoff keeps the original transcript verbatim in
`shared-sessions/<id>/source.jsonl`, with `handoff-bundle.md` (what the
destination received) and `manifest.json` beside it. Handing the same unchanged
conversation off again reuses that snapshot and its note; one that has moved on
is snapshotted afresh. An archive no handoff has used for seven days is deleted
at the next startup or handoff; archives from the earlier full-history restore
are left alone. Folders are created with mode `0700` and snapshots `0600`, so
only your account can read them. They may contain sensitive prompts, tool
output, and source code.

---

## 7. Knowledge

Knowledge turns your conversations with Claude Code, Codex, and Antigravity in a
project into short notes on decisions, causes and fixes, procedures, and
cautions, and makes whole conversations searchable. Notes nobody uses fade and
disappear; notes that are referenced or keep coming up stay.

**Turn it on** per project: open the project's **Knowledge** tab and press
**Collect knowledge in this project**. The first time, it needs the
EmbeddingGemma 2 model — **Download the model and start (310MB)** fetches it
once from a pinned Hugging Face revision and checks its SHA-256. The model runs
on this Mac's GPU; **Settings → Data and recovery → Semantic search
(EmbeddingGemma 2)** downloads or removes it. **Stop collecting (keep
knowledge)** pauses a project without deleting anything.

**How notes are made**: exchanges are ranked locally first. Only the few worth
keeping are sent to an available CLI (Claude Code, Codex, or Antigravity) for a
read-only, one-off summary — at most 30 calls a day, run outside the project's
folder.

**The tab**: switch between **List** and **Map** (notes grouped by meaning on a
2D map). **Sort:** and **Kind:** narrow the list; each note shows whether it is
**Verified** or **Unconfirmed**, and **Pin** keeps one from fading.

**What agents see**: when a Claude Code or Codex session starts, that CLI is
given the titles of up to 10 notes — pinned, verified, and recently used ones
first, including titles made from conversations with other CLIs — and how to
search the knowledge. The search is Operon's own executable run from inside the
project's folder; it opens no window and works while the app is open:

```sh
Operon knowledge search "query" [--limit N] [--json]
```

Knowledge is stored in Operon's data folder on this Mac
(`knowledge/<project id>.sqlite3`), never in the repository.

---

## 8. Keyboard shortcuts

| Keys | Action | Notes |
| --- | --- | --- |
| ⌘K | **Search & actions** | Arrow keys move, Enter opens. Finds sessions by title, request, agent, branch, project, or what the agent last said; projects by name or path; the selected project's files and the symbols of files open in the editor. Among equal matches a waiting session comes first |
| ⌘N | **New session** | |
| ⌘J | Next session that needs you | Longest wait first; the keyboard goes to its terminal. The toolbar's **要対応 N** (Needs you N) does the same |
| ⌥⌘B | Show or hide the side panel | |
| ⌘1 / ⌘2 / ⌘3 | **Home** / **Project** / **Session** | ⌘2 returns to the project list |
| ⌘, | **Settings** | |
| ⌘O | Choose a project folder | |
| ⌘F | Find in the terminal | Enter = next, ⇧Enter = previous, Esc closes. Nothing typed reaches the agent |
| ⌘↩ | Start the session | In the new-session sheet |
| Esc | Close the new-session sheet | |
| ⌘S | Save the open file | In **Files** |
| ⌘T | Open an empty terminal | In the focused pane; the tab strip's **+** → Terminal without the menu. From any page |
| ⌘W | Close the active tab | A session's agent keeps running |
| ⇧⌘T | Reopen closed tab | Newest first; a removed session's tab is skipped |
| ⇧⌘] / ⇧⌘[ | Next / previous tab | Wraps round within the focused pane |
| ⌃Tab / ⌃⇧Tab | Next / previous tab | |
| ⌥ + click / ⌘ + click | Open a path or URL in terminal output | In the editor / externally |

⌘↩, Esc, ⌘S, ⌃Tab, ⌃⇧Tab, and the clicks are fixed; the others can be changed.
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
key. The actions are `palette.open`, `session.new`, `attention.next`,
`page.home`, `page.projects`, `page.sessions`, `page.settings`, `project.add`,
`terminal.find`, and `panel.toggle`. A file that
gives one chord to two actions, or names an unknown action, is refused whole and
Operon starts with the defaults and tells you why.

---

## 9. Settings and language

**Settings** (⌘,) has five sections:

| Section | What it holds |
| --- | --- |
| **Appearance** | **Theme** (Dark by default, Light, and High Contrast (Dark) — applied immediately, including open terminals), **Terminal and code font** (four Sarasa faces), and **Language** |
| **Agent** | Tool status, the hook switch, folder-trust auto-approval, and **Agent accounts** |
| **Notifications** | The notification switch |
| **Data and recovery** | The data folder path, **データフォルダを表示** (show it in Finder), **tmux をスキャン** (scan `tmux`) to adopt sessions the index lost, and **Semantic search (EmbeddingGemma 2)** to download or remove the knowledge model |
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

## 10. Data locations and privacy

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
| `session-order.json` | The order you dragged projects and sessions into |
| `agent-accounts.json` | Registered account folders |
| `keybindings.json` | Your shortcuts, if you created it |
| `diff-comments.json` | Notes written on diffs |
| `setup-trust.json` | Approved `.operon/setup.sh` digests |
| `approved-scripts/` | The exact bytes of each approved `.operon/setup.sh` or `.operon/run.sh`, by digest: what runs is this copy, with `$0` still the script's path in the repository. `${BASH_SOURCE[0]}` names the copy, so a script that finds its folder from it, or runs only when `${BASH_SOURCE[0]}` equals `$0`, should use `$0` or the worktree, which is the current directory. A script that runs `bash "$0"` again runs the repository's file as it is then, not the approved copy |
| `agent-hooks/` | Managed hook scripts, endpoint file, and the local `hook.sock` socket (mode `0700`) |
| `shared-sessions/` | The handoff archive ([section 6](#6-cross-cli-handoff)) |
| `knowledge/` | One knowledge database per project ([section 7](#7-knowledge)) |
| `models/embeddinggemma-2-Q8_0.gguf` | The knowledge model, once downloaded |
| `.operon-instance.lock` | Single-instance lock |

Operon also reads (never uploads) the agents' own transcript folders such as
`~/.claude/projects` and `~/.codex` to find and restore conversations. Builds
from before the rename to Operon kept data in a `xirp-copy` folder; it is
imported on first launch.

---

## 11. Troubleshooting

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

## 12. Uninstalling and removing data

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
   your project list, session records, notes, knowledge, the knowledge model,
   and the handoff archive.
   Transcripts that Claude Code, Codex, or Antigravity wrote are theirs and are
   not touched.

Removing a project or deleting a session inside Operon never deletes files,
branches, or worktrees.
