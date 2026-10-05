---
name: ship
description: Package the macOS bundle behind the fmt/test/clippy gates, atomically replace /Applications/Operon.app, then clean up obsolete bundles. Runs the full AGENTS.md release procedure in order.
disable-model-invocation: true
---

# Ship a verified build

`AGENTS.md` owns the procedure and its defaults. This skill runs them in order and
adds only what `AGENTS.md` does not say: what is safe to auto-fix, and how each
step fails.

**Stop at the first failure and report it — never continue to a later step.**

## 1. Package

```sh
bash scripts/package-macos.sh
```

It runs the three gates over the tree — `cargo fmt --check`, `cargo test --locked`,
`cargo clippy --locked -- -D warnings` — and stops on the first failure, then
compiles `target/release/operon`, signs the bundle, and swaps it into
`dist/Operon.app`, keeping the prior one at `dist/Operon.previous.app`. It never
touches `/Applications`.

`cargo fmt --check` is the only safe auto-fix: run `cargo fmt` and re-run. Test
and clippy failures are **not** auto-fixable here — report and stop.

The gates skip six `#[ignore]`d restore tests; `AGENTS.md` says when to run
them. If you could not, say so in the report.

If it reports "Another Operon packaging operation is already running", a
concurrent run holds the advisory lock on `dist/.operon-package.lockfile`. **Do
not delete that lock file** — the kernel releases it on exit or crash. Find the
other run.

## 2. Install

**Quit the running Operon first.** The installed app and a source build take the
same `flock` on the session index, and the install refuses while Operon runs.

```sh
bash scripts/replace-macos-bundle.sh <repo>/dist/Operon.app /Applications/Operon.app <repo>/dist/Operon.previous.app
```

`<repo>` is the checkout's absolute path, written out: the three arguments must
be plain paths. `.claude/hooks/guard-bash.sh` lets this one form through after
`scripts/check-release-preconditions.sh` re-establishes step 1 rather than
trusting it — the gates pass on the tree as it stands, the tree reproduces
`dist/Operon.app`, and Operon is not running. It names the condition it stopped
on; fix that rather than approving the prompt it falls back to.

## 3. Verify the installed build

1. Launch `/Applications/Operon.app` and confirm it reaches the home dashboard.
2. Move obsolete backup and `dist` bundles to Trash — Trash, not `rm`, so it stays
   recoverable. Retain a rollback copy only when the user asked for one.

## Report

State which gates passed, whether `/Applications` was replaced, and where any
retained rollback copy is.
