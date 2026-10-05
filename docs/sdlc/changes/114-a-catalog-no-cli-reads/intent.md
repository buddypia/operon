# Intent: a skill catalog that no CLI reads

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

Change 112 added `development-skills.md` and the test
`every_skill_is_named_in_a_flat_catalog` on the premise that Antigravity CLI
scans only flat `.agents/skills/*.md` files. Measured on 2026-10-04 in this
repository, the premise is false. Asked which skills its context lists,
`agy --print` named `root-cause`, `feature-pilot`, and `sdlc` as loaded from
`.agents/skills/<id>/SKILL.md`, and said `development-skills` was not listed.
`codex exec` listed the same directory skills. The catalog is read by no CLI,
and the test guards it. What the CLIs do need — `.agents` resolving to
`.claude`, and a `SKILL.md` whose frontmatter names its directory — nothing
checks.

## Who feels it, and when

Whoever adds a skill: the test sends them to edit a file no CLI reads. Every
session under Codex or Antigravity, if `.agents` stops resolving or a `name:`
drifts from its directory, because nothing would say so.

## Desired outcome

No catalog file. A test that fails when a skill would not load under Codex or
Antigravity: `.agents` not a link to `.claude`, a `SKILL.md` missing, its
`name:` not its directory, or its description empty or longer than the 1024
characters the skill format allows.

## Constraints this change inherits

- The commit gate refuses a falling test count. The catalog test is replaced
  by its correct form, not deleted.

## Systems likely affected

`development-skills.md`, `src/tests.rs`, `docs/sdlc/lessons.md`.

## Open questions

None.

## Not in scope

`.claude/skills/git-worktree-isolation-skills.md` is owned by bundle-sync
(`.claude/.bundle-receipt.json`) and comes back on a re-sync; it stays until
the source bundle stops writing it.
