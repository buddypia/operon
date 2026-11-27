# Spec: a backlog kept as GitHub issues, scored by ICE, handed out one item per session

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it. Every test runs
the real script against a stand-in `gh` placed first on `PATH`, which keeps
issues, labels and comments in one file under a lock; no test reaches GitHub.

1. `add` files one open issue carrying the label `backlog`, a label for its
   kind, and its three scores in the body, and prints its number and score —
   `backlog_add_files_an_issue_with_its_kind_and_scores`.
2. `add` with a score outside 1 to 10, an empty title, a title over 256
   characters, a body over 60000, or an unknown kind files nothing and exits 2
   with the reason —
   `backlog_add_refuses_what_it_cannot_score`.
3. `add` with the title of an item that is already open files no second issue
   and prints the existing number; when two were filed at the same moment, the
   later one is closed as a duplicate of the earlier —
   `backlog_add_does_not_file_the_same_item_twice`.
4. `list` prints open items by score, highest first, equal scores oldest first;
   an issue labelled `backlog` whose scores cannot be read is listed last with
   that said, never dropped — `backlog_list_puts_the_highest_score_first`.
5. `claim` hands out the highest-scored item nobody holds; the next `claim`
   hands out the next one; with nothing left it says so and exits 0 —
   `backlog_claim_hands_out_each_item_once`.
6. When several `claim`s run at the same moment, each item is handed to at most
   one of them, and the item is left holding one claim, the winner's — given
   that reading an issue's comments returns every comment already accepted
   (Design states this limit) —
   `backlog_claim_made_at_the_same_moment_has_one_winner`.
7. `release` makes a held item available again, including one held by a
   session that is gone, and `done` closes it with the reference it was given,
   after which neither `list` nor `claim` shows it —
   `backlog_release_frees_an_item_and_done_closes_it`.
8. `score` replaces an item's three scores, and `list` orders by the new ones —
   `backlog_score_reorders_the_list`.
9. When `gh` is missing, refuses, or does not answer in time, every command
   exits 2 with a Japanese sentence saying which, and a `claim` that failed
   part-way leaves no claim behind that it can still remove —
   `backlog_says_why_when_github_cannot_be_reached`.
10. On the main checkout the guard admits the script's command line and refuses
    its neighbours (another script under `python3`, `python3` alone with code,
    the name as a prefix such as a `.bak` copy, an output redirect), and
    `docs/sdlc/risk.yaml` lists the script as gate configuration —
    `backlog_command_runs_on_the_main_checkout`.
11. The skill exists, names only sub-commands the script has, `AGENTS.md` says
    when to read it, and the pipeline's closing step sends a session to it —
    `backlog_skill_names_commands_the_script_has`.

## Behaviour

One script, six sub-commands. It keeps no file: every fact is on the
repository's GitHub issues, so every working copy sees the same list.

```
python3 scripts/backlog.py add --title T --kind remaining|improvement|problem \
        --impact N --confidence N --ease N [--body TEXT] [--source TEXT]
python3 scripts/backlog.py list [--json]
python3 scripts/backlog.py claim [NUMBER] [--json]
python3 scripts/backlog.py release NUMBER [--reason TEXT]
python3 scripts/backlog.py score NUMBER --impact N --confidence N --ease N
python3 scripts/backlog.py done NUMBER [--ref TEXT]
```

**The score.** Impact, confidence and ease are whole numbers from 1 to 10 and
the score is their product, 1 to 1000. Impact is what the person gains when the
item is done; confidence is how sure the session is of that gain and of the
approach; ease is how little work it is (10 is minutes, 1 is days).

**An item's issue.** The title is the item in one line. The body is the text
given with `--body`, then where it was found (`--source`, for example the
change's name), then two lines the script owns:

```
ICE 336 = Impact 8 × Confidence 6 × Ease 7
<!-- backlog ice: impact=8 confidence=6 ease=7 -->
```

The comment line is what the script reads; the line above it is for a person
reading the issue. Labels: `backlog` on every item, and one of
`backlog:remaining`, `backlog:improvement`, `backlog:problem`. A label that
does not exist yet is created.

**What each command prints.**

- `add` → `#12 ICE 336 残り作業 タイトル` and the issue's address. A title already
  open → 「既に登録されています: #12」 and exit 0.
- `list` → one line per item, `#12  ICE 336 (I8 C6 E7)  改善  タイトル`, held
  items included and marked 「着手中: 名前 3時間前」. No item → 「backlog は空です」.
- `claim` → the item it took: number, score, kind, title, address, then the
  body. `--json` prints the same as one object, or `null`. Nothing available →
  「着手できる項目はありません」, exit 0. `claim NUMBER` takes that item or says
  「#12 は既に着手されています: 名前」 and exits 3.
- `release` → 「#12 を戻しました」; `done` → 「#12 を閉じました」; `score` → the
  item's new line.
- Words for kinds: `remaining` 残り作業, `improvement` 改善, `problem` 問題.

**Holding an item.** A claim is a comment on the issue carrying a line the
script owns, `<!-- backlog claim: token=… by=… -->`, where `by` is the host name
and the directory the command ran in. An item is held while it has such a
comment. `claim` posts its comment, reads the issue's comments back, and keeps
the item only if its own is the earliest claim there; otherwise it deletes its
own and, when no number was given, goes on to the next item. `release` deletes
every claim comment on the item. `done` closes the issue with a comment naming
the reference.

**States that are not the happy one.**

- Empty: as above for `list` and `claim`.
- Error: `gh` not on `PATH` → 「gh が見つかりません」; `gh` exits non-zero → 「GitHub
  に届きません: 」 followed by the first line it wrote; no answer within 60
  seconds → 「GitHub の応答がありません (60 秒)」. All exit 2.
- Too large: `list` and `claim` read at most 1000 open items (ten pages of
  100); past that the output ends with 「1000 件を超える項目は読んでいません」.
  Who holds an item is in its comments, read only for an item that has any:
  `list` reads the comments of at most 100 items and marks the rest
  「着手状況は未確認」; `claim` reads them one candidate at a time and gives up
  after 100 candidates. A body is cut at 16 KiB when printed by `claim`. `add`
  refuses a title over 256 characters (「タイトルが長すぎます」) or a body over
  60000 (「本文が長すぎます」), inside GitHub's own limits.
- Interrupted: a `claim` stopped between posting its comment and checking it
  leaves a claim held by a session that is gone. `list` shows who holds an item
  and since when, and `release` frees it; the skill says to release a claim
  whose working copy no longer exists.
- Denied: an unsigned-in `gh` is the error case above; its own message says to
  sign in.

**The skill.** .claude/skills/backlog/SKILL.md, two uses:

- *Record*, when a change closes: every piece of work left undone, every
  improvement noticed and every problem found and left becomes one `add`, with
  the scores argued in a sentence each in the body; the final report lists the
  items with their numbers instead of only describing them.
- *Run*, when the person asks for the next or the most valuable thing: `claim`,
  then carry the item through `.claude/skills/feature-pilot/SKILL.md` with the
  issue's text as the request and its number in `intent.md` and the commit
  subject; `done` after landing, `release` on giving up.

## Design

Placement: `NEW_MODULE` outside the crate — the repository has no backlog, no
issue tooling and no issue (`gh issue list --state all` is empty; `git grep -il
backlog` finds only words in unrelated scripts). Feature cited: none; this is
process tooling and `state.yaml` records `features: "none"`.

- scripts/backlog.py — Python 3 standard library only. Every GitHub call is
  one `gh api` child: program `gh`, arguments built as a list (never a shell
  string), 60 second timeout, output read to a 32 MiB ceiling. `{owner}/{repo}`
  in the path lets `gh` name the repository of the directory it runs in, so a
  working copy and the main checkout address the same issues; the script must
  be run from inside the repository, and outside one `gh`'s refusal is the
  error case of Behaviour. A read carries its parameters in the address and no
  field. A write names its method with `-X` and sends one JSON object on
  standard input (`--input -`), so a title, a body and the label list never
  ride on the command line.
  - list: `GET repos/{owner}/{repo}/issues?labels=backlog&state=open&per_page=100`
    with `--paginate --slurp`, entries carrying `pull_request` skipped.
  - add: labels read once and missing ones created (a label another session
    created in between, which GitHub refuses as existing, counts as created),
    the open list read for the
    title (compared after trimming, case-folding and collapsing spaces), then
    `POST …/issues`; the list is read once more and, if an older open item has
    the same title, the new one is closed with a comment naming it.
  - claim: `POST …/issues/N/comments`, `GET …/issues/N/comments`, and on a loss
    `DELETE …/issues/comments/ID`. Order among claims is the comment id, which
    GitHub assigns in creation order.
  - release, done, score: `DELETE` of claim comments; `PATCH …/issues/N` with
    `state=closed`; `PATCH` of the body with the two owned lines rewritten.
- .claude/skills/backlog/SKILL.md — the skill above.
- `AGENTS.md` — one When-line for the skill, paid for by shortening existing
  lines so the two always-on files stay within `ALWAYS_ON_CHARACTER_CAP`.
- `.claude/skills/sdlc/SKILL.md` §10 — one bullet: leftovers are recorded
  through the backlog skill before the final report.
- `.claude/config/worktree-policy.json` — one pattern in
  `trunk_bash_allowlist`: `python3\s+scripts/backlog\.py`. The guard anchors it
  and already refuses an output redirect into a file on main. What it admits is
  more than the reads main allows today: with any arguments, from main, the
  script files, edits and closes issues and posts and deletes comments on the
  repository's GitHub page. It writes nothing in the checkout.
- `docs/sdlc/risk.yaml` — the script's path joins `gate-configuration`: what
  main runs is now whatever this file says, so a later edit to it is judged as
  an edit to a gate (a path added to a surface is a tightening).
- `src/tests.rs` — the eleven tests, a stand-in `gh` written by the fixture
  helper, and the helper that runs the script bounded through
  `run_command_with_output_limit`.

No persisted shape of the application changes. Nothing runs in the application.

The limit of the claim rule, stated rather than hidden: it rests on GitHub
returning a comment it has already accepted when the comments are read back.
A reader served from a copy that has not yet seen an earlier claim would let
two sessions both keep an item. The stand-in cannot show this; the skill has a
session read `list` when it starts the work, where a second holder is visible.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | nothing is drawn; no `src/` file but `src/tests.rs` changes |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | no mark is added |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes, in spirit | `.claude/rules/identifiers.md` governs the crate; here the label names and the two owned comment lines are each written once in the script, and the tests read them back through `add` then `list`, `claim` then `release`, never by restating them |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | the local store is not touched and the script keeps no file |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | the application gains no child. The script's one child, `gh`, is started from an argument list with a timeout and an output ceiling; the tests start the script through `run_command_with_output_limit` |
| Documentation — user-facing docs change in all three languages together | no | `README.md` and its translations describe the application, which does not change |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | read against `docs/sdlc/README.md`: the promise is the application's. This is a developer tool that calls GitHub only when a session runs it, through the person's own `gh` sign-in, and sends only what the session wrote as the item |
| Budgets — any new scan or output path states its byte and item ceiling | yes | 1000 items, 32 MiB per `gh` answer, 60 seconds per call, 16 KiB of body printed |

## Flagged concerns

- **Widening the main allowlist is a relaxation of gate configuration.**
  `.claude/skills/sdlc/references/approval.md` gives that to a person. Asked on
  2026-11-27 with the alternative (record only inside a working copy, open one
  before taking an item) laid out; the person chose 「追加する (Recommended)」.
  The pattern admits this one script and the test for requirement 10 refuses
  its neighbours.
- **Issues are written to a site outside the machine.** The repository is the
  person's own and the request names GitHub issues as the place, so each `add`
  is the action asked for. The skill tells a session to put no secret, token,
  or other repository's private detail in an item.
- **The always-on files are 70 characters under their ceiling.** The When-line
  is about 150. Resolved by shortening existing `AGENTS.md` lines without
  removing a rule; `every_skill_loads_under_codex_and_antigravity` counts it
  and the tests that pin sentences there say whether a cut went too far.

## Acceptance

- `cargo test --locked backlog_` passes: the eleven tests above.
- `python3 docs/sdlc/changes/323-leftover-work-is-lost-after-the-completion-report/mutations.py`
  catches every mutation.
- `bash scripts/check-bands.sh` shows no new breach.
- On the real repository: one trial item is added, appears in `list`, is taken
  by `claim`, freed by `release`, and closed by `done`; the trial issue is
  closed afterwards.

## Rejected alternatives

- A backlog file in the repository — differs between working copies until each
  lands and collides when two sessions add to it; the request rules it out.
- Scores as labels (`ice:336`) — up to a thousand labels, and a rescore is a
  label swap that can half-happen; the body line is one write.
- GitHub Projects fields for the scores — sortable on the site, but needs a
  project, its field ids and another token scope before anything works.
- Holding an item by assigning the issue — every session is the same GitHub
  user, so an assignee cannot tell two sessions apart.
- Holding an item with a label alone — two sessions both add it and both
  succeed; a label has no order, a comment id does.
- A lock file under the shared `.git` directory — orders sessions on one
  machine only, and is a second record beside the issue.
- Running the script only inside a working copy — a session starts on main,
  where it would have to open a working copy before knowing which item it is
  for; the person chose the allowlist line instead.
- A hook that refuses a finish with unrecorded leftovers — nothing can tell
  from outside whether a session noticed something; prose in the skill, and a
  check only if it is skipped twice.
