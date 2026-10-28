# Plan: Join existing change evidence at an immutable revision

- **Spec**: `./spec.md`
- **Status**: done

## Files that change

| File | Change |
|---|---|
| scripts/change-receipt.py | Read bounded committed records and emit JSON |
| src/tests.rs | Isolated git fixtures for provenance, unknowns and input failures |
| .claude/skills/feature-pilot/SKILL.md | Discoverable closing command |
| docs/sdlc/changes/227-harness-guidance-gaps | Sequential source audit and verification evidence |

## Order of work

1. Commit intent, evaluate and commit spec/plan before source edits.
2. Watch fixture tests fail without the script; implement the read-only report.
3. Verify named tests, formatting/clippy, bands, and mutations of receipt claims.
4. Obtain independent review, push and watch CI, land on main and clean worktree.
5. Only then read the second supplied folder; keep the overall goal active.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Worktree bytes substituted for commit evidence | dirty edits change hashes/report | committed-evidence fixture |
| Historical statements interpreted as a new pass | report says fresh verification | claim mutation |
| Bad/huge/missing records silently accepted | misleading partial receipt | rejection fixture |
| Snapshot parent mistaken for whole-feature revert | rollback claim too broad | explicit scope and claim test |

## Proof of completion

Named tests pass, q.fast exits 0, bands exits 0; receipt reconstructs an existing
committed change with explicit null telemetry; CI succeeds on the branch head.
Review and mutation results are recorded alongside this plan.

## Departures from the plan

The first named-test command stopped before tests: the APFS-seeded CMake cache
still held the main checkout's build path and the removed OpenSSL 4.0.3 include
path. The cache explicitly showed those paths; fresh configuration now names
this worktree and the installed OpenSSL 4.1.0 headers. Only this session's cloned
CMake build directory was moved to a mktemp directory for preservation and
reconfiguration. No production dependency or shared cache was edited.


The first independent review found that git in a partial clone can lazily fetch
missing objects. This violates the existing local-only requirement, so explicitly
disable lazy fetch and add a promisor transport probe. Add real streaming overflow
and deadline cleanup probes too: the original oversized-record test alone checks
the blob-size guard, not the pipe reader. These are Important fixes, not nit edits.
The fixture's root-commit case now removes its own replace ref before constructing
the next input; otherwise commit-tree built that input from the replaced tree.
