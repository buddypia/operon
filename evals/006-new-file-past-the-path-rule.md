---
id: 006
title: A brand-new drawing file still reaches the palette
split: train
seeded-by: docs/sdlc/lessons.md 010
guards: every_rule_declares_the_paths_it_applies_to, design_md_documents_exactly_what_the_app_paints
---

The gap change 007 opened on purpose and wrote down. Path-scoped rules in
`.claude/rules/` load when Claude **reads** a matching file, so editing
`src/ui/widgets.rs` brings the colour rule with it — a read precedes an edit. A
prompt that asks for a **new** file matches no read, and
`.claude/rules/palette-and-glyphs.md` may never arrive.

What stands in for it is a pointer table in `CLAUDE.md`, which is always loaded
and names the rule without carrying its body. This eval is the only thing that
says whether that pointer is enough, and it is the reason moving four policies out
of `CLAUDE.md` was a measured decision rather than a hopeful one.

A failure here does not mean the policy is wrong. It means the pointer is too
thin, and the fix is one line in `CLAUDE.md` — not moving the whole rule back.

## Prompt

```text
新しいウィジェットを `src/ui/badge.rs` として追加してください。
状態を示す小さなバッジで、成功・警告・エラーの三状態を色分けして描きます。
`src/ui/mod.rs` から使えるようにしてください。
```

## Check

```sh
set -e

if [ ! -f src/ui/badge.rs ]; then
  echo "src/ui/badge.rs was not created"
  exit 1
fi

# The three states are colours, and a new file is exactly where a literal gets
# written. Every one of them must be a semantic role read off Palette.
if grep -qE 'Color32::from_rgb|Color32::from_gray|Color32::RED|Color32::YELLOW|Color32::GREEN' src/ui/badge.rs; then
  echo "colour literal in a new drawing file — the pointer in CLAUDE.md did not carry"
  grep -nE 'Color32::from_rgb|Color32::from_gray|Color32::RED|Color32::YELLOW|Color32::GREEN' src/ui/badge.rs
  exit 1
fi

if ! grep -q 'palette\.' src/ui/badge.rs; then
  echo "src/ui/badge.rs reads no Palette role"
  exit 1
fi

# A new role, if one was added, brought its DESIGN.md entry and its three rows.
cargo test --locked design_md_documents_exactly_what_the_app_paints
cargo test --locked every_theme_stays_readable

# And the rules directory is still wired: a rule that lost its paths: would make
# this eval pass for the wrong reason, by loading unconditionally.
cargo test --locked every_rule_declares_the_paths_it_applies_to
```
