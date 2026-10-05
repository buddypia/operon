# Spec: a worktree starts from the mainline, and a taken name gets the next one

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Creating a worktree resolves a base ref before it runs `git worktree add`, in
   this order, stopping at the first that exists: the target of the
   `refs/remotes/origin/HEAD` symref; `refs/remotes/origin/main`;
   `refs/remotes/origin/master`; `refs/heads/main`; `refs/heads/master`.
2. When none of the five resolves, the creation is refused with a message that
   says so and names what would fix it. No base is invented, and in particular
   `HEAD` is never used as one.
3. `git worktree add` is given the base as a fully qualified ref
   (`refs/remotes/origin/main`, `refs/heads/main`), so a tag or a file with the
   same short name cannot be picked instead.
4. The new branch does not track the base: `--no-track` is passed, so a
   subsequent `git push` cannot push to the mainline.
5. A requested name whose branch, path, or registered worktree is already taken
   is retried as `<name>-2`, `<name>-3`, … up to `WORKTREE_NAME_MAX_ATTEMPTS`
   candidates; the first free one is used. Exhausting them is an error naming
   the last candidate tried.
6. After a successful add, `branch.<branch>.base` is set to the qualified base
   in the project's local config, and `push.autoSetupRemote` is set to `true`
   unless it is already set at any scope. Neither failing fails the creation.
7. The confirmation names the branch that was created and the base it started
   from. A person who typed `fix` and received `fix-2` learns it from the
   confirmation, not from the worktree list.
8. Every git invocation added here runs through the timeout wrappers in
   `src/exec.rs`.

## Behaviour

The interface does not change: the same field, the same button.

Typing `fix-login` in a project whose folder is on `feature/old` and pressing the
button produces a worktree on branch `fix-login` started from
`refs/remotes/origin/main`, and the notice reads
「worktree fix-login を origin/main から作成し、次のセッション用に選択しました。」.

Pressing it again with the same name produces `fix-login-2` from the same base,
and the notice names `fix-login-2`. This is the race-N flow: same name, same
base, one more worktree each time.

The states that are not the happy one:

- **No mainline.** A repository with no `origin` and no local `main` or `master`
  refuses: 「このリポジトリの起点ブランチを判定できませんでした。origin/HEAD、
  origin/main、main のいずれかが必要です。」 Nothing is created.
- **Names exhausted.** After `WORKTREE_NAME_MAX_ATTEMPTS` candidates:
  「空いている worktree 名が見つかりませんでした（最後に試した名前: {p0}）。」
- **Invalid name.** Unchanged: 「有効な Git ブランチ名ではありません。」 The
  suffix is applied to a name that is already valid, and `-2` cannot make a
  valid branch name invalid.
- **git refuses the add.** Unchanged: git's own stderr is surfaced.
- **The two config writes fail.** Silent. They are conveniences, and a worktree
  that exists must not be reported as a failure because a config write did not
  land.

## Design

`src/git.rs` gains four functions and one type:

```rust
pub(crate) struct WorktreeBase { pub qualified: String, pub display: String }
pub(crate) fn detect_worktree_base(project: &Path) -> Option<WorktreeBase>
pub(crate) fn worktree_name_candidate(name: &str, attempt: usize) -> String
pub(crate) fn is_worktree_name_available(project: &Path, name: &str, path: &Path) -> bool
pub(crate) fn configure_created_worktree(project: &Path, branch: &str, base: &str)
```

`detect_worktree_base` asks `git symbolic-ref --quiet refs/remotes/origin/HEAD`
first and, when that answers, verifies the target with
`git rev-parse --verify --quiet <target>^{commit}`; then it probes the four
fallbacks with the same verify. `display` is the ref with `refs/remotes/` or
`refs/heads/` stripped, which is what a person calls it.

`worktree_name_candidate(name, 0)` is `name`; attempt 1 is `name-2`. The
arithmetic lives in one function so the off-by-one is written once.

`is_worktree_name_available` is three checks: no `refs/heads/<name>`, no
directory at the destination, and no registered worktree with that path.

`configure_created_worktree` runs the two `git config` calls and ignores their
outcome, which is why it returns nothing — a caller cannot mistake the result
for the result of the creation.

`src/app.rs` `create_worktree` keeps its shape. Inside the background closure the
order becomes: validate the name, detect the base (refuse if none), walk
candidates until one is available, run `git worktree add` with `--no-track`, the
branch, the destination, and the qualified base, then
`configure_created_worktree`. The
`WorktreeCreated` result gains the branch and the base display so the notice can
name them.

`src/config.rs` gains `WORKTREE_NAME_MAX_ATTEMPTS: usize = 100` — the same
ceiling, and the point past which a person is doing something other
than what this loop is for.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing is painted; the existing field, button, and notice are unchanged in shape. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The attempt ceiling is `WORKTREE_NAME_MAX_ATTEMPTS` in `src/config.rs`. The ref names are a single ordered list inside `detect_worktree_base` and appear nowhere else; the suffix shape is only in `worktree_name_candidate`, which the availability loop calls rather than restating. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No persisted shape changes. The base is recorded in git's own config, which is where the next reader of the branch will look for it. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Every new call is `command_output_in_dir` or `run_command_with_timeout`, both of which carry a timeout. No shell. The branch name is validated by `git check-ref-format` before it reaches any argument list, and arguments are passed as a vector, never interpolated. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in `README.md`, `README.ja.md`, `README.ko.md` under the worktree section. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | No network: the base is read from refs already on disk. This is the reason a fetch was rejected. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | The candidate loop is bounded by `WORKTREE_NAME_MAX_ATTEMPTS`; base detection is at most six git calls; all use the existing 20-second wrapper. |

## Flagged concerns

- **A stale `origin/main` is still a wrong base, just a less wrong one.**
  Resolved: the display name is in the confirmation, so the person is told what
  they branched from and can fetch and remake. Fetching automatically is
  rejected below.
- **`push.autoSetupRemote` is a change to the user's git config.** Resolved: it
  is set at `--local` scope in their own project, only when unset at every
  scope, and it is what makes requirement 4's `--no-track` safe rather than
  annoying. A person who has an opinion has already set it, and their setting is
  detected and left alone.

## Acceptance

- `cargo test --locked` passes, including:
  - `detects_the_base_ref_from_origin_head`
  - `falls_back_through_main_and_master_when_origin_head_is_missing`
  - `refuses_to_create_a_worktree_with_no_detectable_base`
  - `numbers_the_second_worktree_of_the_same_name`
  - `skips_a_name_whose_branch_exists_without_a_worktree`
  - `qualifies_the_base_ref_so_a_tag_cannot_win`
  - `records_the_creation_base_in_git_config`
  - `leaves_an_existing_push_autosetupremote_alone`
- In the running app: with the project folder checked out on a feature branch,
  create a worktree; `git -C <new worktree> log --oneline -1` is the tip of
  `origin/main`, not of the feature branch, and the notice names `origin/main`.
  Creating the same name again yields `-2` and the notice says so.

## Rejected alternatives

- **Fetch before resolving the base.** It puts a network call, which can hang or
  prompt for credentials, on the path of a button that must feel immediate; and
  today's failure is not staleness but arbitrariness. A refresh belongs behind an
  explicit action.
- **Use `HEAD` when nothing resolves.** That is the present behaviour and the
  bug. A repository with no mainline is rare and the error tells the person what
  is missing.
- **Ask the person which base to use.** A picker is a screen, and the default is
  right often enough that asking every time is the wrong trade. When a picker
  exists this detection is what fills it in.
- **A `git-username` branch prefix.** It exists to keep many people's
  branches apart on a shared remote; it is a setting to add when someone wants
  it, not a default to inherit.
- **Reuse an existing branch when the name matches.** The rule
  ("check out the existing branch when it points at the base or shares its
  name") is subtle enough to surprise, and the suffix already gives the person a
  worktree. Taking the next name is never wrong; adopting a branch can be.
