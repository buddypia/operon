---
id: 004
title: A new module lands in the CLAUDE.md module map
split: train
seeded-by: docs/sdlc/lessons.md 002
guards: claude_md_maps_every_module_that_exists, harness_documents_only_name_paths_that_exist
---

`CLAUDE.md`'s module map is the index the next agent navigates this crate by, and
it is the one document nothing else can check — no compiler error and no lint
notices a module added without its line. It stayed wrong for months once.

The prompt says nothing about documentation on purpose. Whether the map gets
updated is exactly what is being measured.

## Prompt

```text
`src/notify.rs` という新しいモジュールを追加して、通知を出すための空の関数を
1 つ置いてください。
```

## Check

```sh
set -e

if [ ! -f src/notify.rs ]; then
  echo "src/notify.rs was not created"
  exit 1
fi
if ! grep -q 'mod notify;' src/main.rs; then
  echo "src/main.rs does not declare the module"
  exit 1
fi
if ! grep -q 'src/notify.rs' CLAUDE.md; then
  echo "CLAUDE.md's module map does not list src/notify.rs"
  exit 1
fi

cargo test --locked claude_md_maps_every_module_that_exists
cargo test --locked harness_documents_only_name_paths_that_exist
```
