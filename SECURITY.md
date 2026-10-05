English | [日本語](SECURITY.ja.md) | [한국어](SECURITY.ko.md)

# Security Policy

## Supported versions

Security fixes are applied to the latest code on `main`. Please update to the
most recent revision before reporting.

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Report privately through GitHub's *Security → Report a vulnerability*
(private vulnerability reporting) for this repository. Include:

- affected component (e.g. session index store, tmux stop path, restore,
  packaging script),
- steps or a proof of concept,
- your assessment of impact.

You can write in English, 한국어, or 日本語. We will acknowledge reports and
coordinate disclosure timing with you.

## Scope notes

Operon is local-first software: it has no server, no accounts, and no
telemetry. Areas of particular interest include:

- the local session metadata store and its migration/rollback behaviour;
- durability of cancellation records relative to tmux state changes;
- single-instance locking of the session index;
- bounded scans and truncation guarantees around untrusted repository content;
- the cross-CLI conversation restore path, which reads and writes transcript
  files owned by other tools.

The local shared-session archive intentionally stores original transcripts,
which may contain sensitive prompts, tool outputs, and code. Its directories are
created `0700` and the transcript snapshot `0600`, so the account that created
them is the only one that can read them; nothing in the app relaxes that.
Reports about that directory are still welcome.
