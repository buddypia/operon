English | [日本語](CONTRIBUTING.ja.md) | [한국어](CONTRIBUTING.ko.md)

# Contributing to Operon

Thank you for considering a contribution. Issues, pull requests, and
translations are all welcome. You can write in English, 한국어, or 日本語 —
any of the three is fine everywhere.

## Development setup

- macOS (Apple Silicon or Intel)
- Rust via [rustup](https://rustup.rs/)
- `tmux` (needed to launch sessions; some tests exercise tmux-related logic)
- At least one agent CLI (`codex`, `claude`, `agy`) if you want to exercise
  launch paths manually

```sh
git clone https://github.com/buddypia/operon.git
cd operon
cargo run --release
```

Only one instance may hold the local session index at a time. Quit any
installed build before running from source, and vice versa.

## Before you open a PR

Run the same checks CI runs:

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

Six tests are `#[ignore]`d because they need an authenticated `codex`, `claude`,
or `agy` plus `tmux` on the machine. They are the only end-to-end coverage of
the cross-CLI restore, and CI cannot run them. If your change touches restore or
the shared-session archive, run them locally and say so in the PR:

```sh
cargo test --locked -- --ignored
```

Please keep `Cargo.lock` changes intentional and separate from unrelated
edits when possible.

## Pull request process

1. Create a branch for your change; do not commit to `main`.
2. Make the change, and run the checks in the previous section.
3. Open a PR and fill in the
   [PR template](.github/PULL_REQUEST_TEMPLATE.md): link the pipeline artifacts
   (or say why they were skipped), tick the checklist, and attach screenshots
   for UI changes.
4. Respond to review against the passes in [REVIEW.md](REVIEW.md). A person
   approves the merge.
5. If the change touches user-facing documentation, the three language versions
   are updated together (see Conventions).

## Commit messages

Use a short Conventional-Commits-style subject, as in the existing history:
`type(scope): what changed and why`, for example
`fix(tmux): the suite runs tmux on a server of its own` or
`docs: package-macos.sh builds dist/Operon.app`. Common types are `feat`, `fix`,
`docs`, `chore`, and `test`. If a change skipped the pipeline below, say so in
the commit message.

## How changes are shaped

Larger changes run through the pipeline in
[docs/sdlc/README.md](docs/sdlc/README.md): a committed `intent.md` (the problem,
not the solution), then `spec.md` (what it must do, and which of this project's
policies it touches), then `plan.md` (which files, in what order, with what
proof). The artifacts for one change live together under `docs/sdlc/changes/`.

Use it when a change adds user-visible behaviour, touches more than one module,
changes a persisted shape, adds a dependency, or alters a policy. Skip it for a
typo, a translation fix, or a one-line correction — and say so in the commit
message.

[REVIEW.md](REVIEW.md) is the review policy: the passes, where the line between
Important and a Nit falls, what is excluded, and which changes a person must
read before they merge.

## Conventions

- **Documentation lives in three languages.** User-facing docs changes should
  update [README.md](README.md), [README.ko.md](README.ko.md), and
  [README.ja.md](README.ja.md) together. Keeping them structurally identical
  makes review easier.
- **Design tokens have one source of truth.** Colour/theme changes must update
  [`DESIGN.md`](DESIGN.md) alongside the constants and pass the WCAG 2.1 AA
  contrast tests.
- **Durability matters.** Changes to the store schema, migration/rollback,
  cancellation sidecar, tmux stop/retry, or scan/output limits deserve extra
  tests and an explanation of the failure modes they cover.
- **Local-first means local-first.** No telemetry, accounts, or cloud calls.
  New dependencies are scrutinized accordingly.
- **No secrets.** Never commit API keys, tokens, or personal data.

## Licensing of contributions

By submitting a contribution, you agree that it is licensed under the
[MIT License](LICENSE), the same license as the rest of the project.

## Reporting bugs and vulnerabilities

Bugs go to this repository's **Issues** tab, using the bug report template.
Security vulnerabilities follow [SECURITY.md](SECURITY.md) — please do not open
public issues for them.
