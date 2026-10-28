# Verification

## Native cache finding


The path test passed, but q.fast rebuilt a second CMake build variant whose cloned
cache still referenced main and removed OpenSSL 4.0.3. The earlier recovery had
covered only the test-profile variant. This is a repeated cache-seeding fault,
not a documentation failure; change 230 will prevent stale native configuration
from being reused across worktree paths before this change closes.

## Results

- Document-path test: 1 passed; q.fast now exits 0 after only the stale clippy
  CMakeCache.txt was preserved in scratch and CMake reconfigured that variant.
- Bands: exit 0, three existing warn tiers; always-loaded guide unchanged.
- Independent source coverage approved at 14659ae37ed20d2c.
- Independent template evaluation at fb30fdb85da95a05: comparable flat data is
  NO_GAIN; mismatched/absent evidence is INCONCLUSIVE. These are illustrative
  reasoning cases, not measured model results or automated enforcement.
- No new tests for reversible Markdown/template guidance; existing document
  validation and the independent guidance evaluation are the appropriate checks.
- No production source, dependency, store, UI or installed app change.

Keep this worktree's repaired build cache until change 230 and the third-folder
work no longer need it; cleanup is deferred only for that owned cache use.
