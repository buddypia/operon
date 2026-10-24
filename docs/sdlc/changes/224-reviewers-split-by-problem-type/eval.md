# Eval: does one reviewer reading the skill find what two reviewers found?

Run 2026-10-25, before any file outside this directory was edited. Kept so the
next fold — the Rust and subprocess reviewers, which ran together in 89 of 173
recorded reviews — is measured the same way.

## What the records say first

Tallied from every `review.yaml` under `docs/sdlc/changes/` (173 records):

| Reviewers that returned a verdict | Changes |
|---|---|
| rust, subprocess | 65 |
| rust | 58 |
| rust, subprocess, durability | 24 |
| subprocess | 20 |
| rust, durability | 4 |
| subprocess, durability | 1 |

Rounds narrated with an Important above zero: subprocess in 10 changes, rust in
5, durability in 1 (067) — and in 067 the subprocess reviewer raised it, and
durability took it up only when it was asked again. In 29 runs, durability
raised no Important of its own.

## Tasks

Only changes whose branch survives in git history can be replayed (history
starts at 6ebed6a, 2026-10-05). Three of them were reviewed by durability, and
each carried findings that are now the golden set:

| Task | Diff | Golden findings (from that change's `carried:`) |
|---|---|---|
| 203 | `2a8b539...cea5c11` | D1 a failed rename in `adopt_shared_statusline_marker` deletes the shared marker · R1 a failed marker removal makes the slot read as the person's · R2 the marker is keyed by the path's spelling · R3 several notices crowd the settings row |
| 208 | `3dae356...0b32a27` | D1 a second copy-diff click is silent · D2 a status landing after its project was removed leaves an entry · D3 `read_run_script` checks the size before an unbounded read · R1 source checks are substring matches · R2 the template menu is empty on its first frame |
| 209 | `38ae356...03cdba5` | D1 a crash between write and rename leaves a temp file in `approved-scripts/` · D2 copies are never pruned · R1 source checks are substring matches · R2 with git missing the land says the branch name is invalid |

D is a durability finding, R a Rust-craft one.

## Arms

- **Folded**: `rust-reviewer.md` as this change writes it, reading the
  durability skill's **Reviewing a diff against this** section. Two runs per
  task, because a fold that loses a finding may lose it in one run of two.
- **Split** (today): the current `durability-reviewer` and `rust-reviewer`, one
  run each per task, scored as their union.

Every run was blind: told not to read the change's own directory or anything
after its head commit, read-only, no subagents, and asked for every finding
rather than a capped list. Prompt, per run:

```text
You are acting as a code reviewer for the Operon repository at <worktree>. This
is a blind replay of a past review. Your reviewer definition: <definition>.
The diff under review: git diff <base>...<head> -- . ':!docs/sdlc/changes'.
Read code as it was at the head with git show <head>:<path>.
Read-only. Do not read anything under docs/sdlc/changes/<NNN>-*/ in any
revision, and do not look at commits after <head>. Do not spawn subagents.
Final message: your verdict line, then every finding as severity · file:line ·
symbol · failure scenario.
```

## Results

Golden findings found (a finding counts when it names the same symbol and the
same failure):

| Task | Folded run 1 | Folded run 2 | Split (rust ∪ durability) |
|---|---|---|---|
| 203 | D1 | R1 | D1, R1 |
| 208 | — | R2 | D1 |
| 209 | D2 | D1, D2, R2 | R2 |
| **mean per task** | | **3.5 of 13** | **4 of 13** |

By lens: durability findings 4 of 12 folded, 2 of 6 split — 33% each. The
durability reviewer re-run found neither of the two durability findings recorded
for 209, so the record itself is one sample, not ground truth.

Flips against the split arm: 203 lost one finding in each folded run (each found
one of the two the split found); 208 lost D1 in both folded runs, both of which
judged the background dedupe sound; 209 gained D1 and D2.

Unverified Importants: the split Rust reviewer called `@{-1}` in 209 an
Important without running git; the change's record says git asked from `/`
already refuses it. The folded runs raised the same point as a nit. No other
Important in any arm, which matches the records: these are final diffs.

Beyond the golden set, both folded runs on 209 found the subprocess finding
recorded for it (`BASH_SOURCE` points at the copy), and a folded run on 208
found one recorded for the subprocess reviewer (the cold first commit).

Cost per task: folded 46k–63k tokens, mean 54k, 180–490 s; split 54k–61k
tokens combined, mean 58k, 185–290 s each when the two run in parallel.

## Reading

Within this sample the folded reviewer is neither better nor worse at finding
what was found, costs about the same tokens, and takes longer than two reviewers
run side by side. What it removes is structure: one definition, one verdict, one
report to reconcile, and one copy of the durability procedure. The article's
rule is to keep a subagent only when the folded version scores worse; it does
not, so durability folds.

Three tasks and two runs is small. The 208 flip is the one to watch: if a later
store change lands a durability defect that a folded review read past, that is
an escape, and this file is the baseline to re-run against.

## Re-running it

Spawn one general-purpose agent per task and arm with the prompt above, the
definition pointed at the file under test, and score against the table. For the
next fold, add the subprocess findings recorded for 203, 208, and 209 to the
golden set and compare `rust-reviewer` reading the subprocess procedure against
the two reviewers apart.
