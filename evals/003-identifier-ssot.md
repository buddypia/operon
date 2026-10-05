---
id: 003
title: Renaming the managed session prefix keeps one literal
split: test
seeded-by: docs/sdlc/lessons.md 001
guards: the_session_prefix_is_written_in_exactly_one_place
---

The incident this repository is most shaped by. A rename that moved the gate and
not the five call sites made every managed session unrecognizable to the app that
created it, and the suite stayed green throughout.

The check deliberately does **not** rely on the guard test alone. Lesson 004 is
that same guard going blind on this exact change, so the check greps the tree for
the new prefix as well.

## Prompt

```text
管理する tmux セッションの接頭辞を `operon-` から `opn-` に変更してください。
既存のセッションの扱いも壊さないようにしてください。
```

## Check

```sh
set -e

# The new value exists, and exists in the one place that owns it.
if ! grep -q '"opn-"' src/config.rs; then
  echo "src/config.rs does not declare the new prefix"
  exit 1
fi

# Nothing else spells it out. config.rs declares it; tests.rs quotes it to test it.
offenders=$(grep -rn '"opn-' src --include='*.rs' \
            | grep -v '^src/config.rs:' | grep -v '^src/tests.rs:' || true)
if [ -n "$offenders" ]; then
  echo "the prefix is written outside src/config.rs:"
  echo "$offenders"
  exit 1
fi

# The legacy prefix is still accepted: sessions named under the old name are
# running on real machines and recorded in real stores.
if ! grep -q 'MANAGED_TMUX_LEGACY_PREFIX' src/config.rs; then
  echo "the legacy prefix was removed — running sessions become unrecoverable"
  exit 1
fi

# The round trip, and then everything else the rename could have broken.
cargo test --locked the_session_prefix_is_written_in_exactly_one_place
cargo test --locked
```
