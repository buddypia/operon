# Spec: claim gives back what a gone session held, and the backlog command alone takes the next item

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it. The tests run the
real script against the stand-in `gh` of change 323, from a throwaway git
repository the test makes, so the git side is real too.

1. `claim` with no number hands out an item whose only claim is at least 24
   hours old, whose holder names no linked working copy of the repository, and
   which no commit on a branch or remote-tracking ref names as `(#N)`. Before
   taking it, it deletes the old claim and comments
   「戻しました: 着手から 1 日以上たち、作業ツリーもこの項目のコミットもありません (着手: 名前)」;
   the item then holds one claim, the new one —
   `backlog_claim_gives_back_an_item_a_gone_session_held`.
2. `claim` does not give an item back, and hands out nothing for it, when any
   one of these holds: the claim is younger than 24 hours; a linked working
   copy's directory name equals the holder; a commit on a branch or
   remote-tracking ref names `(#N)`; the command runs outside a git repository;
   git does not answer —
   `backlog_claim_keeps_an_item_someone_may_still_work_on`.
3. `claim NUMBER` run from a linked working copy that was created after the
   item's claim, on an item whose claim is held under the main copy's directory
   name, moves the claim to this working copy: afterwards the item holds
   exactly one claim, under this copy's name, and a day later it is still not
   given back while this copy exists. The same command is refused with exit 3
   when the working copy is older than the claim (another session's copy) or
   the claim is held by another linked copy, and on an item already held under
   this copy's own name it prints the item and exits 0 —
   `backlog_claim_moves_to_the_working_copy_that_took_it_up`.
4. `claim NUMBER` applies requirement 1 to the item named: a holder left behind
   is given back and the item taken — a named case in the first test.
5. The skill says that the backlog command given alone takes the next item and
   carries it out, that the list alone is for a person asking what is left,
   that the claim is moved into the working copy right after it is opened, and
   no longer tells a session to release a gone session's claim by hand —
   `backlog_skill_takes_the_next_item_when_called_alone`.

## Behaviour

**The command alone.** `/backlog` with nothing after it, or a request for the
next or the most valuable thing, runs `claim` at once and carries the item out.
`list` is run, and the session stops after it, only when the person asks what
is left. A request naming an item runs `claim NUMBER`.

**Moving the claim into the working copy.** A session claims on the main copy,
opens its working copy, and from inside it runs `claim NUMBER` again. Since the
claim is held under the main copy's name, this is a linked copy, and the copy
was created after the claim (its administrative directory under the common git
directory, `git rev-parse --absolute-git-dir`, was born after the claim's time), the
script posts a claim under this copy's name, then deletes the earlier one, and prints
「#12 の着手をこの作業ツリーに移しました」 and the item. From then on the item is
protected for as long as that working copy exists, and after it is removed by
any commit naming `(#N)`. A session stopped between the post and the delete
leaves two claims; the earlier, the main copy's, still wins, and the item is
given back a day later as for any claim never moved.

**Giving back.** Only `claim` gives back, and only while deciding about an item
it would otherwise skip. An item is given back when all hold:

- its earliest claim was made 24 hours ago or more;
- no linked working copy (`git worktree list --porcelain`, every entry but the
  first, which is the main copy, whose directory still exists) has a directory
  name equal to the claim's holder, compared after the same normalising
  `claim` applies to its own name; locked entries count as existing;
- no commit on a local branch or remote-tracking ref (`git log --branches
  --remotes -F --grep "(#N)" -1`) carries `(#N)` in its message.

Then every claim on it is deleted, the comment above is posted (the holder's
name is the directory name already on the issue, nothing new), and `claim`
proceeds as for a free item. A claim made from the main copy and never moved
into a working copy, with no commit naming the item, is given back after a day.

**States that are not the happy one.**

- Not in a git repository, git missing, git exits non-zero, or git takes longer
  than the script's timeout: the item is treated as still worked on — never
  given back — and nothing is printed about it. Wrongly keeping an item costs a
  skip; wrongly giving one back costs duplicate work.
- A claim whose time cannot be read: not given back.
- Git's repository-pointer variables (`GIT_DIR`, `GIT_WORK_TREE`,
  `GIT_INDEX_FILE`, `GIT_COMMON_DIR`), which a git hook exports to its
  children, are removed from the script's git children, so git reads the
  repository of the directory the command runs in.
- Too large: each git answer is read up to 1 MiB; `git log` stops at the first
  match. Each git call is bounded by the same timeout as `gh` calls.
- Two sessions giving back the same item at once: both delete claims (a delete
  of one already gone is ignored), both post their claim, and the existing
  earliest-claim rule leaves one winner. 「戻しました」 may appear twice.

**What does not change.** `list`, `add`, `release`, `score`, `done`, the claim
comment's form, and the main copy's allowlist.

## Design

Placement: `EXTEND` — `scripts/backlog.py` and `.claude/skills/backlog/SKILL.md`
of change 323 (`python3 scripts/change-map.py --find backlog` names it). No
product feature: `features: "none"`.

- `scripts/backlog.py`:
  - `STALE_HOURS = 24`.
  - `git(*arguments)` — one `git` child, argument list, the pointer variables
    removed from its environment, output capped at 1 MiB, the script's timeout;
    returns stdout text, or `None` on any failure.
  - `name_of(text)` — the normalising `who()` applies, factored out so both
    sides compare alike; `who()` calls it.
  - `working_copies()` — `(main name, set of linked names)`, or `None`.
  - `left_behind(number, claim)` — True only when all three conditions hold;
    any `None` from git is False.
  - `give_back(number, claim)` and `move_here(number, claims)`;
    `born_after(stamp)` reads this working copy's administrative directory's
    birth time (`st_birthtime`, macOS) and is False when it cannot.
  - `claim()` — where a holder is found: held by this copy's own name → show
    it; named, held under the main copy's name, and run from a linked copy →
    `move_here`; `left_behind` → `give_back` and take it as free; otherwise as
    today.
- `.claude/skills/backlog/SKILL.md`: the opening says the command alone is the
  Run use; step 1 of Run is for "what is left" only; step 3 runs `claim N`
  inside the new working copy; "When something is off" says `claim` gives back
  a gone session's item on its own, and a manual `release` is for a working
  copy that exists but was abandoned.
- `src/tests.rs`: four tests, and a helper making a git repository with one
  commit in a fixture directory.

No new external program beyond `git`, which the repository already requires;
arguments are a list, never a shell string.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Nothing painted. |
| Icons | No | Nothing painted. |
| Identifier SSOT | Yes | The holder's name is normalised by one function used both when written and when compared. |
| Durability | No | No local store; the record is GitHub issues, unchanged in shape. |
| Subprocess safety | Yes | `src/exec.rs` governs the crate; this is a Python script. Each `git` child is an argument list with a timeout, an output ceiling and the repository-pointer variables removed, as the script's `gh` calls are bounded. |
| Documentation | No | No user-facing documents change; the skill is English. |
| Local-first | Yes | No new network call: `git` reads local refs only, no fetch. |
| Budgets | Yes | 1 MiB per git answer; `log -1`; candidates still capped at 100 per `claim`. |

Gate configuration (`docs/sdlc/risk.yaml` lists `scripts/backlog.py`): this
is a relaxation in `.claude/skills/sdlc/references/approval.md`'s terms — a
held item, which `claim` refuses today, can be taken without a person once its
holder left nothing behind for a day. The main copy's allowlist does not change.
The plan goes to the person.

## Flagged concerns

- **Sessions on another machine or clone** — the evidence is this clone's
  working copies and refs. A session elsewhere that holds an item for a day
  without pushing a commit naming it loses the item. Accepted as a limit: the
  repository is developed on one machine, and a pushed branch with `(#N)` is
  seen once fetched. Recorded here and in the skill.
- **A session that does not move its claim** — the only thing protecting a
  claim made on the main copy after the first day is the skill's step that
  moves it, or a commit naming the item. A session that skips both loses the
  item a day after claiming it. Accepted: the skill states the step, and the
  cost is a duplicate start, visible on the issue as two claims over time.
- **A working copy opened after another session's claim** — a session whose
  copy happens to be newer than another's main-copy claim, and which names that
  item, takes it over. A session names an item only when the person does or
  when its own `claim` handed it out, so this needs a person to name an item
  another session already holds.

## Acceptance

- `cargo test --locked backlog_` passes, including the four tests above.
- The mutation sweep catches each condition of `left_behind` removed, the age
  threshold changed, and git failure turned into "gone".
- Against the repository's own issue #2 after landing: the item is closed by
  `done`; nothing else is exercised against GitHub.

## Rejected alternatives

- A heartbeat the working session refreshes: a session that forgets is
  indistinguishable from one that died, and it adds a step to every session.
- Recording a session identifier in the claim: nothing can ask whether another
  session is alive, and the repository is public.
- Giving back in `list`: a listing should not write.
- Matching a working copy by the issue number in its branch or files: the
  branch name carries the change number, not the issue's, and a file search
  across working copies reads uncommitted work of other sessions.
- Fetching before checking commits: a network call per candidate, and a
  remote branch is fetched by every session's own push anyway.
