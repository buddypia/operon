---
id: 001
title: A colour change goes through the palette, not the call site
split: test
seeded-by: DESIGN.md
guards: design_md_documents_exactly_what_the_app_paints, every_theme_stays_readable
---

The rule an agent has to reach on its own: drawing code never names a colour, it
reads a semantic role off `Palette`, and the three `*_PALETTE` tables plus
`DESIGN.md` decide what the role looks like. A prompt phrased as a visual tweak is
the one most likely to produce a literal at the call site, because "slightly
deeper red" sounds like a number.

## Prompt

```text
エラー通知バナーの背景を、いまより少し濃い赤にしてください。
```

## Check

```sh
set -e

# The palette is where a colour lives.
if git diff --quiet -- src/theme.rs; then
  echo "src/theme.rs untouched — the colour was changed somewhere else"
  exit 1
fi

# No colour literal reached a drawing call site. src/theme.rs holds the tables and
# src/tests.rs quotes them to test them; every other module goes through a role.
for file in $(git diff --name-only -- src | grep '\.rs$' \
              | grep -v '^src/theme.rs$' | grep -v '^src/tests.rs$'); do
  if git diff -U0 -- "$file" | grep '^+' | grep -q 'Color32::from_rgb'; then
    echo "colour literal added at a call site: $file"
    exit 1
  fi
done

# DESIGN.md and the constants are checked against each other in both directions,
# and every ink still has to clear WCAG 2.1 AA on every surface.
cargo test --locked design_md_documents_exactly_what_the_app_paints
cargo test --locked every_theme_stays_readable
```
