# Final review: eval runs always have a deadline

Self-review: same session; two independent index reviewers approve
40207f546c49163c. Base: main 3e96ca3; reviewed index over 49afd14.

## Project profile and scope

Rust/macOS desktop app, local persistence, agent CLI integrations, GitHub Actions.
This change touches a developer eval script, its fixture tests and internal docs.
No production Rust, persisted app shape, GUI, dependencies or bundle changes.
REVIEW.md governs Important/Nit severity and the independent digest reviews.
The first source folder's last applicable implementation gap is the optional eval
deadline; folders two and three remain unread, so the overall goal remains open.

## Gate

| Evidence | Result |
|---|---|
| Regression before fix | No-GNU run reached outer 30s test bound; empty limit started probe agent |
| cargo test --locked the_eval_runner_ | 12 passed, zero failed |
| make q.fast | fmt and clippy exit 0 |
| lesson-admission named test | Failed for missing watched-failing evidence, then passed after correction |
| check-bands | exit 0, existing steering/module/subagent warn tiers |
| mutations.py | 4 of 4 caught; source restored |
| Independent code review | Both approve, zero findings, digest 40207f546c49163c |
| Independent first-folder coverage audit | No further materially applicable recommendation unaddressed; CI/landing still required |

## Dimension coverage

| Dimension | Evidence / applicability |
|---|---|
| D1 Intent | Closes the first-folder deadline gap; original three-folder scope preserved |
| D2 Logic | Validate limit before agent; same existing process-group helper on every platform |
| D3 Errors | Empty, zero, leading-zero, signed, noninteger and overflowing limits exit 2 before agent |
| D4 Tests | Actual regressions observed and all twelve old/new runner cases pass; early result cannot mask timeout |
| D5 Security | Separate quoted argv; no new shell-evaluated input; no real paid/key-bearing agent in tests |
| D6 Privacy | No new collection or transmission; only existing eval transcript behavior |
| D7 Supply chain | N/A: no dependency/manifest/CI change; existing Perl helper reused |
| D8 Performance | Bounded default 1800s; test process self-ends if deliberately unbounded |
| D9 Integrity | Existing throwaway worktrees and result classification preserved |
| D10 Reliability | Missing GNU utilities no longer remove deadline; child stops and infra remains outside pass rate |
| D11 ROI | Removes optional fallback and availability probe; reuses existing helper |
| D12 Compatibility | Stock bash-compatible script; explicitly valid positive 32-bit seconds |
| D13 DX | Conditions always report enforced deadline; invalid values get actionable stderr |

## Findings

No remaining implementation findings. Final checking did find and correct the
lesson-admission record omission before commit. That failure is not hidden by
passing runtime tests; the previously red gate was rerun and passed.

## Verified

Twelve runner tests, lesson admission, fmt/clippy, bands, four claim/behavior
mutations, two independent reviewers and the first-folder source coverage audit.

## NOT verified

Branch full suite belongs to CI and must pass before landing. GUI/live restores
are unchanged and were not exercised. No actual paid model eval was run.

## Known limitations

The helper terminates the agent process group. This change does not add an OS
sandbox or promise cancellation of deliberately detached processes. Folder two
and folder three are still outstanding under the original goal.

## Adversarial check

A timed-out agent could have already emitted a successful result: the stub does
exactly that and must still yield infra with zero judged runs. A machine without
GNU timeout could silently remove the limit: the test PATH excludes both commands.

This change is needed because an existing advertised budget was optional. A
new timeout implementation was rejected in favor of run-bounded.sh; the old
availability probe and its mirrored test helper were deleted. No new runtime or
framework can be removed to simplify further. The initial one-second probe assumed
fast process startup; an isolated pass and parallel failure exposed that premise,
so the probe allows five seconds and checks cancellation rather than speed.

## Review summary

Implementation: Go, no unresolved findings. Branch CI and landing are mandatory
external gates before advancing to the next source folder.

NO UNRESOLVED DECISIONS
