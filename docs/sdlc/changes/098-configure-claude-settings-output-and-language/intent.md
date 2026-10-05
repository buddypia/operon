# Configure Claude settings outputStyle and language

## Problem

`.claude/settings.json` lacked explicit `outputStyle` and `language` configuration.

## Proposed Change

Set `outputStyle: "Concise"` and `language: "japanese"` in `.claude/settings.json`.

## Risk and Review

Touches `gate-configuration` (.claude/settings.json). The change adds standard configuration properties without weakening hooks or permissions.
