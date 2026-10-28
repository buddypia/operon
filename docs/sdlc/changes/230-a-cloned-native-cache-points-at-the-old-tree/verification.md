# Verification: relocated native configuration

Named cause: cloned CMakeCache.txt retains absolute build and dependency paths;
CMake skips configuration, then fails against the old directory/OpenSSL version.
The original new regression failed because the cloned cache remained active.

| Check | Result |
|---|---|
| cloned_native_cache regression | Pass; two variants actually reconfigure with CMake, source bytes and objects retained, recovery available, external symlink target untouched |
| Existing seed/format regression | Pass |
| make q.fast | Pass |
| Lesson admission | Pass, lesson 102 records the watched-failing regression |
| bash syntax / git diff whitespace | Pass |
| Bands | Pass with existing warn tiers |
| Mutations | Five of five caught: active cache, one-variant-only reset, source write, external symlink write, false rebuild claim |
| Independent review | Both approve, zero findings at 5944b4a68dd2a927 |
| Real failure recovery | 229's failing clippy build passed in 1m46 after only its stale CMakeCache.txt was preserved and reconfigured |

The fixture initially compared /var with /private/var and was corrected to compare
both physical locations. It still rejects the old build directory; this was not
an implementation relaxation. The script resets only direct Cargo out/build
native caches in the owned destination, retains unique sibling backups, and
leaves existing targets/no-cache behavior alone. Branch CI is required before
landing. No production Rust, app data, dependency or bundle changed.

Self-review dimensions: intent/correctness pinned by the regression (D1/D2);
missing caches and symlink parents covered (D3); red/green plus mutation evidence
(D4); quoted argv and physical containment (D5); no secret reads (D6); no dependency
change (D7); existing clone retained, only config rebuilt (D8); source ownership
and backups checked (D9); failures visible, old seed regression preserved (D10);
reuse existing seeder, no new runtime (D11); stock bash and existing CMake toolchain
(D12); truthful developer-facing rebuild notice (D13). Independent reviewers
judged the index, not the mutation worktree.

NO UNRESOLVED DECISIONS
