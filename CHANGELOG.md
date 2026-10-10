# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Standardized open-source repository documentation and metadata (`CHANGELOG.md`).
- Extended `.gitignore` to cover macOS metadata (`._*`, `.Spotlight-V100`, `.Trashes`), editor configurations (`.idea/`, `.vscode/`, `*.swp`), and Rust tooling artifacts (`*.rs.bk`, `*.profraw`, `*.profdata`).

### Changed
- Cleaned up redundant `.tmp/` entry in `.gitignore`.

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
