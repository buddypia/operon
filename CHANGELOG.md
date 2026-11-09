# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Knowledge: per-project notes distilled from conversations with Claude Code, Codex, and Antigravity, ranked on-device with EmbeddingGemma 2 (at most 30 summary calls a day), shown as a list or a map, marked verified or unconfirmed, replaced when a later conversation proves one wrong, briefed to Claude Code and Codex at session start, and searchable with `Operon knowledge search`.
- Cross-CLI Handoff bundle: a conversation moves to another CLI (or forks within the same one) as a compact bundle of the first request, recent turns, and the last 3 tool results, with a model and reasoning effort per destination; an unchanged conversation reuses its snapshot, and unused archives are deleted after seven days.
- Ports in the session header are split by folder (this working folder, elsewhere in the project), and a server can be stopped from its row after a confirmation.
- Agent IDE actions: land a worktree into main, `.operon/run.sh`, create a pull request and fetch a GitHub issue through `gh`, checkpoints with rollback, external editors, per-hunk stage and discard, `@` context in the request, parallel runs across worktrees, send code or the latest terminal error to an agent, `⌘K` file and symbol search, and prompt templates.
- Session sidebar filters: status tiles with live counts and a CLI segment.
- Session tabs show their state; the new-terminal **+** follows the last tab; `⌘W` closes a tab and `⌃Tab` / `⌃⇧Tab` move between tabs.
- Session names are numbered and can be renamed from the sidebar or a tab.
- Text files between 128 KiB and 8 MiB open read-only in a virtualised, highlighted viewer.
- Standardized open-source repository documentation and metadata (`CHANGELOG.md`).
- Extended `.gitignore` to cover macOS metadata (`._*`, `.Spotlight-V100`, `.Trashes`), editor configurations (`.idea/`, `.vscode/`, `*.swp`), and Rust tooling artifacts (`*.rs.bk`, `*.profraw`, `*.profdata`).

### Changed
- Restore project selection controls around central tabs, keep live sessions out of archived-history flooding, and preserve each project's open files, focused tab and split layout when switching.
- Sessions and files share central workspace tabs with draggable horizontal and vertical groups; the session sidebar lists projects. Internal file opens keep the session page, and closing a session tab leaves its agent running.
- The project **Changes** tab splits into a file list and a diff pane.
- The idle session group reads 手すき, so it no longer reads as waiting on you.
- Dark chrome is unified with restrained orange accents; the command palette sits near the top of the window.
- README screenshots retaken from a demo workspace, and the README and user manual (English, 日本語, 한국어) brought up to date, including new Handoff and Knowledge chapters in the manual.
- Cleaned up redundant `.tmp/` entry in `.gitignore`.

### Fixed
- Terminal caret box no longer drifts one row below the running CLI's input row or outside the pane with scrollback: the cursor is tracked from tmux row coordinates, drawn only on rows the pane actually shows, and the IME candidate window follows it.
- Terminal caret box cannot go stale or escape the pane (change 167): a poll whose capture comes back without the marker line clears the stored cursor instead of keeping the last one, the box is gated to cells fully inside the pane (egui's spare row, half cells and columns past the pane's current width draw nothing until the next capture), and a pane-echoed marker line can no longer pose as the genuine cursor report.

## [0.1.0] - 2026-10-05

### Added
- Initial public release of Operon: A local-first macOS cockpit for AI coding agents.
- Multi-agent cockpit supervising Claude Code, Codex CLI, and Antigravity CLI sessions.
- Native macOS UI built with `eframe`/`egui` and Phosphor icons.
- Local-first architecture: SQLite session metadata store, zero telemetry, no accounts or cloud dependencies.
- Interactive git diff viewer with word-level diffing and line staging.
- Unified terminal integration over `tmux` with session supervision and timeline navigation.
- Multi-lingual documentation and UI support (English, 日本語, 한국어).
- Hardened SDLC governance, worktree isolation, and automated quality gates.
