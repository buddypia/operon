# Owed to the ledger

This entry belongs in `docs/sdlc/lessons.md` and is not there yet. Another
session is holding ninety-three uncommitted lines at the end of that file, and
the number this entry takes depends on whether those land. Appending to a
document another session is writing is the collision change 051 exists to
refuse, so it waits here, written, rather than being written twice or written
over.

---

## 0NN — A target wrote the eighth copy of a list a test held seven of

**What happened.** Change 051 integrated the worktree-isolation harness. When wiring the `q.check` target into the `Makefile`, a generic Rust triple was written by hand:
`cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --
-D warnings`, `cargo test --all-features`. Those are not this repository's three
gates. `--locked` was gone from all three, which is the flag that keeps a gate
from updating `Cargo.lock` in place and reaching the network, and
`--all-features` made the set of code being checked different from the set the
commit gate checks. So `make q.check` could report a pass over a build the
commit gate would then refuse, and refuse a build the commit gate would take.

**Why it was invisible.** The repository already had the mechanism for exactly
this, and had had it for some time:
`every_document_that_names_the_gates_names_the_same_three` reads seven files and
asserts each names all three gates. Its own doc comment says "seven copies of
one list drift the moment a check is added to one of them." The `Makefile` was
the eighth copy, and that test's list is written out by name, so the new copy
was outside the mechanism from the instant it was created — not drifted later,
*born* drifted. The suite stayed at 495 passing. Two reviewers over six messages
read the 85-file diff and did not see it; the pass that did see it arrived after
the change had already landed.

**Cure.** `Makefile` added to the test's list, and `q.check` rewritten as the
same three, in the same order and spelling, as the other seven.

**The rule.** A test that enumerates the files it checks has that list as part
of its mechanism, so adding a copy of the guarded thing without adding it to
that list is not a small omission — it manufactures exactly the drift the guard
exists to prevent, and does it silently. Introducing harness targets across
multiple scripts is a common way for an untracked copy to appear if manual
additions bypass existing consistency checks.

**Guard.** `every_document_that_names_the_gates_names_the_same_three`, its list
extended from seven files to eight. Watched failing with `cargo test --locked`
put back to `--all-features` in the `Makefile`, which printed
"Makefile に `cargo test --locked` がありません — 検査の一覧が食い違っています",
then watched passing with it restored.
