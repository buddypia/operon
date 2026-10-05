---
id: 002
title: A new mark becomes an ICON_ constant, not a pasted glyph
split: train
seeded-by: src/glyphs.rs
guards: every_icon_resolves_from_the_bundled_icon_font
---

Same rule as colour, one level down. The tempting wrong answer is a Unicode
symbol that looked close enough — it renders, it is in the diff, and nothing
about it says the text face ahead of the icon font may shadow it on another
machine.

## Prompt

```text
worktree の行の先頭に、ブランチを表すアイコンを付けてください。
```

## Check

```sh
set -e

# The constant and its vocabulary entry both live in src/glyphs.rs.
if git diff --quiet -- src/glyphs.rs; then
  echo "src/glyphs.rs untouched — no ICON_ constant was added"
  exit 1
fi
if ! git diff -U0 -- src/glyphs.rs | grep '^+' | grep -q 'ICON_'; then
  echo "src/glyphs.rs changed but no ICON_ constant was added"
  exit 1
fi

# No glyph written at a call site. Phosphor glyphs sit in the Unicode private use
# area, so a pasted one is visible as a private-use character on an added line.
for file in $(git diff --name-only -- src | grep '\.rs$' | grep -v '^src/glyphs.rs$'); do
  if git diff -U0 -- "$file" | grep '^+' | perl -ne 'exit 1 if /\p{Private_Use}/'; then
    :
  else
    echo "glyph literal added at a call site: $file"
    exit 1
  fi
done

# Every constant resolves to a real glyph the text face does not shadow.
cargo test --locked every_icon_resolves_from_the_bundled_icon_font
```
