# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Standardized open-source repository documentation and metadata (`CHANGELOG.md`).
- Extended `.gitignore` to cover macOS metadata (`._*`, `.Spotlight-V100`, `.Trashes`), editor configurations (`.idea/`, `.vscode/`, `*.swp`), and Rust tooling artifacts (`*.rs.bk`, `*.profraw`, `*.profdata`).

### Changed
- Sessions and files share central workspace tabs with draggable horizontal and vertical groups; the session sidebar lists projects. Internal file opens keep the session page, and closing a session tab leaves its agent running.
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
