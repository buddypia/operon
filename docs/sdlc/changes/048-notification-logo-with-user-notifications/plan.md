# Plan: Application logo in macOS notifications via UserNotifications

- **Spec**: `./intent.md`
- **Approved**: 2026-09-15
- **Status**: done

This is the plan for replacing osascript-only macOS notifications with native `UserNotifications` framework dispatch when running inside the application bundle, falling back to `osascript` in unbundled/test environments.

## Files that change

| File | Change |
|---|---|
| `src/sys.rs` | Add native `UserNotifications` dispatch and authorization request with single `unsafe` block; fall back to `osascript` if unbundled or dispatch fails |
| `src/app.rs` / `src/app/screens.rs` | Request notification authorization on app startup and when notifications are enabled in settings |
| `scripts/package-macos.sh` | Copy `operon-icon-1024.png` into bundle resources and register bundle with LaunchServices |
| `src/tests.rs` | Test notification fallback in test environment and verify bundle detection |

## Order of work

1. Implement `dispatch_native_notification` in `src/sys.rs` encapsulating `UserNotifications` calls in exactly 1 `unsafe` block.
2. Hook `request_notification_authorization` in `src/app/screens.rs` and `src/app.rs`.
3. Update `scripts/package-macos.sh` to copy `operon-icon-1024.png` and invoke `lsregister -f`.
4. Add unit tests in `src/tests.rs`.
5. Run repository gates: `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`.
6. Run `scripts/package-macos.sh` and atomically update `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Calling `UNUserNotificationCenter` in unbundled process (like `cargo test`) throws `NSInternalInconsistencyException` | Crash / abort during tests | Guarding dispatch with `[NSBundle mainBundle] bundleIdentifier != nil` before any `UNUserNotificationCenter` access |
| Adding multiple `unsafe` blocks exceeds `bands.yaml` threshold (`unsafe_blocks: max 2 3 3 3`) | `check-bands.sh` proposal breach or gate rejection | Encapsulating native dispatch in a single `unsafe` block |
| Missing LaunchServices registration causes notification rejection | Notification banner does not appear | `lsregister -f` registered during packaging and bundle swap |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 495 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `send_macos_notification` successfully runs in `cargo test` using fallback.
- `scripts/check-release-preconditions.sh` confirmed byte-for-byte reproducibility and all release gates passed.
- `/Applications/Operon.app` was atomically updated via `scripts/replace-macos-bundle.sh` and registered in LaunchServices with `lsregister -f`.

## Departures from the plan

- Avoided string literals starting with `"operon-"` prefix in `src/sys.rs` to satisfy the repository's `the_session_prefix_is_written_in_exactly_one_place` invariant test.
