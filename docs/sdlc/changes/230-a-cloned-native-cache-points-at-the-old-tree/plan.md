# Plan: Native CMake configuration must not move between worktrees

- **Status**: approved

## Named cause and evidence

seed-target.sh clones target/debug wholesale, including absolute CMakeCache.txt
paths. The llama.cpp build script sees that cache and skips configuration. In
changes 227 and 229 this produced a CMake directory mismatch and a compiler include
path for removed OpenSSL 4.0.3. Recovering only one hash missed the clippy variant;
229's test passed but q.fast failed in the second build directory. This repeated
harness fault blocks verification of the second source folder. Folder three is
still unread. The exact 229 failure is in that change's verification.md.

## Files that change

- scripts/seed-target.sh: after successful clone, rename each direct Cargo native
  out/build/CMakeCache.txt to a unique sibling backup, only within the owned cloned
  tree; preserve source cache and compiled artifacts; report that native config
  may rebuild rather than promising only changed Rust will compile.
- src/tests.rs: two native variants, source byte preservation, compiled artifact
  retention, cache backup and real minimal CMake reconfiguration at the new path;
  guard against a symlinked parent writing outside the cloned worktree.
- docs/sdlc/lessons.md: repeated relocation failure and its guard.
- This change's state, review and mutation evidence.

## Order of work

1. Evaluate and commit this plan; bugfix skips intent/spec.
2. Watch cloned_native_cache regression fail on the original seeding script.
3. Add the narrow cache-reset step, adjust its message, keep existing no-cache and
   existing-target behavior, and verify the old seed regression still passes.
4. Verify the script against the two real failing cache variants in 229, then
   rerun its path test/q.fast so the docs change can proceed.
5. Run q.fast, bands, meaningful mutations and independent reviews; CI then main.
6. Return to 229, finish second-folder adoption, and only then open folder three.

## Boundaries and risks

Only CMakeCache.txt in direct target/debug/build/*/out/build outputs is reset;
no source or app data is changed. A physical-parent check prevents traversal out
of the owned destination through symlinked directories. Unique backup names
preserve old bytes. Missing caches remain a no-op; compiled objects stay seeded.
No new runtime, dependency, network path, production byte budget or persisted shape.
CMake is already required by this crate's native dependency; a tiny project with
no language/compiler lets the fixture verify configuration without building C++.
The test's commands are bounded and do not contact a service or use keys.

## Proof of completion

Named native-cache and old seed tests pass; CMake configures the cloned path;
source cache bytes and outside symlink targets are preserved; q.fast/bands pass;
mutations caught; independent review and branch CI passed before landing.

## Departures from the plan

None.
