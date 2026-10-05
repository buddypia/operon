# Sanitize local developer username and enrich Cargo.toml metadata

## Problem

Before publishing the repository to GitHub Public, several closed change records
in `docs/sdlc/changes/` retain references to the developer's local machine username
`a13973` and local absolute paths (`/Users/a13973/...`).
In addition, `Cargo.toml` lacks standard open-source package metadata (`license`,
`repository`, `homepage`, `readme`, `keywords`, `categories`).

## Proposed Change

- Add `scrub-names` containing `a13973` so that closed change records can be
  sanitized in a single atomic commit.
- Sanitize occurrences of `a13973` in `docs/sdlc/changes/` (`003`, `004`, `056`,
  `057`, `061`, `065`, `066`, `067`, `068`, `069`, `082`, `083`, `103`, `104`),
  replacing hardcoded local paths with dynamic repo-relative resolution or generic paths.
- Enrich `Cargo.toml` with package metadata for public distribution.

## Risk and Review

Touches `Cargo.toml` (package metadata only, no dependency added) and documentation
records under `docs/sdlc/changes/`. Low risk with zero code or binary behavior changes.
