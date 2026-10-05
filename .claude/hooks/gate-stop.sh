#!/usr/bin/env bash
# Stop: a change may not end with what it promised still unsettled.
#
# Blocks a completion claim that has no evidence behind it. It reads the contract
# in each open change's state.yaml, the approvals ledger, and the commits on the
# branch.
#
# It used to refuse any uncommitted Rust as "no gate run behind it" too, without
# looking for a gate run, so it said so after the gates had passed as well. The
# gates stand on the paths out instead: `.claude/hooks/gate-commit.sh` runs them
# on a commit that touches Rust, and `scripts/package-macos.sh` runs them over
# the tree before it builds the application a person runs. Sdlc 096 took that
# half out.
#
# It refuses once per position and never twice: a contract item settled or a
# further edit to the code is a new position. A gate that can block the same
# tree again is a loop, not a gate — so the state is stamped before the refusal,
# and the second attempt to finish the same tree is allowed through. Answering
# the refusal by settling the contract is the point; answering it by trying again is
# permitted, and visible in the transcript. The branch audit at the end is
# not stamped: an unjudged commit on the branch is refused at every stop until
# the branch is clean, because trying again does not make it so — every stop
# except the one Claude Code retries inside the same turn after a refusal, which
# `stop_hook_active` marks and which this exits 0 on, since a hook that refuses
# that one too is a loop rather than a gate.
#
#   OPERON_SKIP_STOP_GATE=1   finish without the check (say why)
set -uo pipefail
# `git replace` rewrites what every later `git` call sees: a replacement object
# can give a commit a different tree, so the diff this gate judges is not the
# diff the repository keeps. The refs live in `refs/replace/*` and a clone
# fetches them on request, so they are not local-only. Judge the real objects.
export GIT_NO_REPLACE_OBJECTS=1
# And the graft file, which does the same thing by another mechanism and is
# not stopped by the variable above: `$GIT_DIR/info/grafts` — or GIT_GRAFT_FILE
# — rewrites a commit's parents, so `sha^` names a commit the author chose. A
# twin with the same tree makes the diff empty and the gate finds nothing to
# judge. Measured: `-c core.graftFile=/dev/null` does not close it; this does.
export GIT_GRAFT_FILE=/dev/null

payload=$(cat)

# Claude Code sets this when a Stop hook has already fired for this stop. Its own
# loop guard; respected here rather than duplicated.
[ "$(printf '%s' "$payload" | jq -r '.stop_hook_active // false')" = "true" ] && exit 0
[ "${OPERON_SKIP_STOP_GATE:-0}" = "1" ] && exit 0

# --- which tree is this session working in? -----------------------------------
#
# Not `CLAUDE_PROJECT_DIR`, and not the directory this hook happens to start in.
# The first names the project the *session* belongs to; AGENTS.md asks work to
# land from a worktree of that project, so a session whose project directory is
# the main checkout and whose working directory is a worktree is the arrangement
# this repository prescribes. The second is the Claude Code process's own
# working directory, which is that same main checkout.
#
# Measured with the variable unset, which is its state here: this hook reported
# the open contract items of changes 039 and 049 to a session in a worktree
# where neither directory exists. Eleven stops were answered by reading items
# that could not be settled from where they were reported — a gate pointing at
# another tree does not merely miss, it manufactures work.
#
# Worse than the wrong list, it was two lists. `git diff HEAD` is answered from
# `GIT_DIR`/`GIT_WORK_TREE` when they are set, and
# `.claude/scripts/worktree-init.mjs` writes both into a new worktree's
# settings — so the Rust file list came from the worktree while the contract
# scan below, a filesystem glob relative to the working directory, came from the
# main checkout. One refusal assembled out of two repositories. And the stamp,
# which is what keeps this gate from refusing the same position twice, landed in
# the main checkout's `target/`, shared by every worktree on the machine.
#
# So ask git the same question from the place the session reported, with the
# environment left as the session has it. This is change 057's fix, which
# `.claude/hooks/gate-commit.sh` has carried since — the sweep across to here is
# what was missing, and `the_stop_gate_reads_the_tree_the_session_is_working_in`
# is what now holds both hooks to the same answer.
# Where this hook's own file is, resolved BEFORE the `cd` below. A relative
# `$0` resolves against the process's working directory, which is the main
# checkout — and that is precisely the case being detected, so resolving it
# afterwards would answer with the tree under suspicion. `$0` rather than
# `CLAUDE_PROJECT_DIR` for the same reason: the variable is the thing in doubt.
hook_directory=$(cd -- "$(dirname -- "$0")" 2>/dev/null && pwd)

running_in=$(printf '%s' "$payload" | jq -r '.cwd // empty')
[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=.
# `cd --`, because a directory named `-P` passes the `-d` test above and would
# otherwise be read as an option, sending `cd` to $HOME.
working_tree=$(cd -- "$running_in" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)
cd -- "${working_tree:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0
git rev-parse --git-dir >/dev/null 2>&1 || exit 0

# The stamp path is written here and nowhere else.
# `the_stop_gate_stamp_is_spelled_in_exactly_one_place` reads the literal out of
# this file rather than restating it — the lesson-004 rule applied to a shell
# script, because a guard that retypes the value it guards survives the rename
# that breaks the thing it was watching.
stamp="target/.operon-stop-gate"

# Is this hook the tree's own copy? The repository's reference-transaction hook
# reads the review gate out of `refs/heads/main`, and a worktree session runs
# whatever the main checkout's configuration points at:
#
# (The path to that hook is deliberately not written here.
# `the_review_gate_is_still_not_installed_by_anything_committed` looks for a
# file in this directory that names both that directory and a git hooks
# destination, and cannot tell a sentence about a mechanism from a line that
# runs it — the same trap `shell_code` was written for one file over.)
# the fix for a defect found here does nothing for anyone until the merge lands,
# and until then every finding below is about a tree this file does not live in.
# That was survivable; what was not is that nothing said so. Three stops were
# spent diagnosing a refusal that named change directories which do not exist in
# the tree being worked in, and the output looks exactly like a real finding.
#
# It does not refuse on a mismatch — that would stop every worktree session on
# this machine until main takes the merge, which is worse than the defect. The
# whole cost was not knowing.
hook_repository=$(cd -- "${hook_directory:-.}" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)
# Asked here rather than reusing `working_tree`, so the answer covers all three
# resolution paths above — the reported cwd, the inherited pointers, and the
# fallback to `CLAUDE_PROJECT_DIR` — with one question instead of three.
judged_tree=$(git rev-parse --show-toplevel 2>/dev/null)

# --- what has this change promised that is still unsettled? --------------------
#
# The three gates are
# the floor every change shares; a `machine` item in a change's own state.yaml is
# what *this* change said done would mean. An open change whose contract still
# reads pending is not finished, however green the suite is.
#
# Only `machine` items, and only in a change that is still open: a `human` item
# cannot be settled by a command, and a done or archived change is history.
#
# And only in a change this tree is working on: its directory has uncommitted
# files, or a commit on this branch since it left `main` touches it. Reading
# every open change handed each session the pending items of thirty it was not
# working on, most parked behind 035, which it answered by hand at every stop
# and again in every new tree, since the stamp is per tree (sdlc 081).
# `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on`.
#
# Paths, not `git status` lines: uncommitted and untracked files, then the
# branch's commits, with `-z` so git quotes nothing. Parsing porcelain text read a staged rename
# as the directory it left and missed any path git quoted — one with a space
# (sdlc 083). `--no-renames`, because a detected rename is printed by its new
# path alone, and a file moved out of one change into another would drop the
# change it left.
in_play=$(
  {
    git diff -z --no-renames --name-only HEAD -- docs/sdlc/changes 2>/dev/null
    git ls-files -z --others --exclude-standard -- docs/sdlc/changes 2>/dev/null
    git diff -z --no-renames --name-only "$(git merge-base refs/heads/main HEAD 2>/dev/null)" HEAD -- docs/sdlc/changes 2>/dev/null
  } | tr '\0' '\n' | sed -n 's|^docs/sdlc/changes/\([^/]*\)/.*|\1|p' | sort -u
)
# With no `main` to fork from, nothing can say what this branch committed, so
# every open change is read, as before 081: an unread contract is the failure
# this section exists to prevent, and a list is only noise.
every_change=0
git merge-base refs/heads/main HEAD >/dev/null 2>&1 || every_change=1
open_contract=$(
  for state_file in docs/sdlc/changes/*/state.yaml; do
    [ -f "$state_file" ] || continue
    if [ "$every_change" = 0 ]; then
      printf '%s\n' "$in_play" | grep -xF "$(basename "$(dirname "$state_file")")" >/dev/null || continue
    fi
    status=$(sed -n 's/^  status: "\(.*\)"$/\1/p' "$state_file" | head -1)
    # `if`, not a one-line `case`: macOS's stock /bin/bash 3.2 rejects a
    # `case … ;; esac` written on one line inside a command substitution, and
    # the whole audit below would be unreachable on the machine it ships to.
    if [ "$status" = done ] || [ "$status" = archived ]; then continue; fi
    awk -v change="$(dirname "$state_file")" '
      /^contract:/ { inside = 1; next }
      inside && /^[a-z]/ { inside = 0 }
      inside && /"machine pending / {
        item = $1; sub(/:$/, "", item)
        how = $0; sub(/.*"machine pending /, "", how); sub(/"$/, "", how)
        print "  " change " → " item ": " how
      }' "$state_file"
  done
)

# --- and is every approval those changes claim written down? ------------------
#
# Sdlc 082. An artifact that is not a significant decision is approved by an
# evaluator, and the ledger in the change directory is the only record that it
# was. scripts/check-approvals.sh refuses a claim with no line behind it, and an
# automatic approval in a kind an escape handed back to a person. Only for the
# changes in play, for the same reason as the contract above. Its one-line
# summary is how a person learns approvals happened without being asked each
# time; `the_stop_gate_reports_automatic_approvals_in_one_line`.
approval_findings=""
approval_summary=""
approval_dirs=$(
  printf '%s\n' "$in_play" | while IFS= read -r change; do
    [ -n "$change" ] && [ -d "docs/sdlc/changes/$change" ] && printf 'docs/sdlc/changes/%s\n' "$change"
  done
)
if [ -f scripts/check-approvals.sh ] && [ -n "$approval_dirs" ]; then
  # One name per line and no glob or space in a change name, so the split is
  # the list.
  # shellcheck disable=SC2086
  approval_findings=$(bash scripts/check-approvals.sh $approval_dirs 2>/dev/null)
  [ "$?" = 1 ] || approval_findings=""
  # shellcheck disable=SC2086
  approval_summary=$(bash scripts/check-approvals.sh --summary $approval_dirs 2>/dev/null)
fi

# Once per summary: the same count at the next stop is not news.
say_approvals() {
  [ -n "$approval_summary" ] || return 0
  mark="approvals $(printf '%s' "$approval_summary" | shasum | cut -d' ' -f1)"
  [ -f "$stamp" ] && grep -xF "$mark" "$stamp" >/dev/null && return 0
  mkdir -p "$(dirname "$stamp")" 2>/dev/null && printf '%s\n' "$mark" >> "$stamp"
  jq -n --arg m "$approval_summary" '{systemMessage: $m}'
}

# --- and is every commit on the branch one the review gate judged? -------------
#
# .githooks/reference-transaction judges a commit as git makes it, and REVIEW.md
# lists the ways a commit can reach the branch around it: a hooks path
# overridden for one command in a spelling gate-commit.sh does not read, a hook
# removed by hand, a clone before its first commit here. None of those leaves a
# trace at the time. This reads the commits themselves: every commit on HEAD
# since the last one this gate saw pass — or since the commit that first added
# the hook — goes through scripts/check-review.sh --commit, which asks the
# commit object and not the command. What it finds is not undone here; it is
# named, and the session does not end on it in silence.
#
# The mark is one ref per branch, refs/operon/audited/<branch>, so a stop on a
# side branch does not pin main's mark at the fork and make every later stop
# on main re-read everything since. It advances as each commit passes, so an
# audit the Stop hook's budget cuts short resumes where it stopped rather than
# starting over, and only forward: a detached checkout of an old commit audits
# nothing and must not drag it back. A branch with no mark starts from main's
# when that is an ancestor, else from the commit that added the hook; a branch
# whose history has neither has no anchor here and is guarded by the git hook.
#
# The script is the main branch's committed copy, or the main checkout's
# working one only before a committed one exists — never this checkout's when
# this checkout is a branch, which could stub it, and never a working copy
# while a committed one exists, since an edit interrupted mid-way can exit 0
# early with its last line untouched — and when it cannot be had the stop is
# refused rather than the audit skipped in silence.
branch=$(git symbolic-ref --short -q HEAD 2>/dev/null || echo HEAD)
# One ref per branch, the branch's slashes flattened so `feature/sub`'s mark
# cannot sit where `feature`'s would have to be written.
audited="refs/operon/audited/$(printf '%s' "$branch" | tr '/' '-')"
unjudged=""
audit_failure=""
# Whole means the script's last line is the marker: `-s` is one byte deep, and
# a prefix cut at a statement boundary still parses and exits 0 on everything.
whole() { [ -s "$1" ] && [ "$(tail -n 1 "$1")" = '# operon: end of scripts/check-review.sh' ]; }
common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)
main=""
[ -n "$common" ] && main=$(dirname "$common")
gate="$main/scripts/check-review.sh"
gate_tmp=""
# The commit that added the git hook, asked before anything else here. A branch
# whose history has none, and that no mark anchors, is the exemption described
# above — the git hook's to watch, not this — and asking for the script first
# turned that exemption into a refusal at every stop, never stamped, in any
# repository without one. Since the tree judged is the one the session reports
# it is working in, that was any repository a session happened to be in.
gate_commit=$(git log --diff-filter=A --format=%H -- .githooks/reference-transaction 2>/dev/null | tail -1)
anchored=$gate_commit
[ -n "$anchored" ] || anchored=$(git rev-parse --verify --quiet "$audited" 2>/dev/null) \
  || anchored=$(git rev-parse --verify --quiet refs/operon/audited/main 2>/dev/null) \
  || anchored=""
if [ -n "$anchored" ]; then
  if git cat-file -e refs/heads/main:scripts/check-review.sh 2>/dev/null || [ -z "$main" ] || ! whole "$gate"; then
    blob=refs/heads/main:scripts/check-review.sh
    if git cat-file -e "$blob" 2>/dev/null; then
      if gate_tmp=$(mktemp "${TMPDIR:-/tmp}/operon-check-review.XXXXXX" 2>/dev/null); then
        trap 'rm -f "$gate_tmp"' EXIT
        if git cat-file blob "$blob" > "$gate_tmp" 2>/dev/null \
          && [ "$(wc -c < "$gate_tmp" | tr -d ' ')" -eq "$(git cat-file -s "$blob" 2>/dev/null || echo -1)" ] \
          && whole "$gate_tmp"; then
          gate=$gate_tmp
        else
          audit_failure="the main branch's scripts/check-review.sh could not be written whole"
        fi
      else
        audit_failure="no temporary file could be made for the main branch's scripts/check-review.sh"
      fi
    else
      audit_failure="scripts/check-review.sh is missing or cut short in the main checkout and the main branch does not carry it"
    fi
  fi
fi
if [ -z "$audit_failure" ] && [ -n "$anchored" ]; then
  # The tip is fixed before the walk: a commit that reaches the branch while
  # the walk runs is not in the list, and a mark advanced to a re-read HEAD
  # would name it as audited without anyone having read it.
  head=$(git rev-parse HEAD 2>/dev/null)
  since=$(git rev-parse --verify --quiet "$audited" 2>/dev/null) \
    || { since=$(git rev-parse --verify --quiet refs/operon/audited/main 2>/dev/null) \
         && git merge-base --is-ancestor "$since" "$head" 2>/dev/null; } \
    || since=$gate_commit
  # An amend or a rebase leaves the mark off the branch — not an ancestor of
  # the tip — and a mark that can only move forward would then never move
  # again, the walk growing at every stop. It steps back to where the branch
  # and the mark part, which everything before was audited up to, and no
  # earlier than the commit that added the hook.
  if [ -n "$since" ] && [ -n "$head" ] && ! git merge-base --is-ancestor "$since" "$head" 2>/dev/null; then
    fork=$(git merge-base "$since" "$head" 2>/dev/null)
    # Clamped to the gate commit only when the gate commit is on this branch.
    # A branch rewound below it and rebuilt does not have it as an ancestor, and
    # pinning `since` there left both `is-ancestor` guards below false forever:
    # the mark never advanced again and every stop re-walked the same range.
    # `^fork` and `^gate_commit` exclude the same commits when the clamp does
    # apply, so nothing is dropped from the walk by declining it here.
    if [ -n "$gate_commit" ] && [ -n "$fork" ] \
      && git merge-base --is-ancestor "$fork" "$gate_commit" 2>/dev/null \
      && git merge-base --is-ancestor "$gate_commit" "$head" 2>/dev/null; then
      fork=$gate_commit
    fi
    since=$fork
    [ -n "$since" ] && git update-ref -m "operon stop gate: mark stepped back to the fork" "$audited" "$since" 2>/dev/null
  fi
  # Every path that arrives here without a range is a refusal, not a skip.
  # `git merge-base A B` prints nothing and exits 1 when the two share no
  # ancestor, so a mark left behind by a deleted orphan branch, inherited by a
  # new branch of the same name, emptied `since` and skipped the whole audit in
  # silence — and, since nothing rewrote the mark, at every stop after it. This
  # is the third time an audit has gone quiet rather than refused (a script
  # nowhere, an unwritable TMPDIR), so the guard is on the class: any missing
  # range at all. `gate_commit` empty is the documented exemption — a branch
  # with no gate commit in its history is the git hook's to watch, not this.
  if [ -n "$gate_commit" ] && { [ -z "$since" ] || [ -z "$head" ]; }; then
    audit_failure="the mark $audited points at history this branch does not share, so nothing could be audited — remove it with: git update-ref -d $audited"
  fi
  if [ -n "$since" ] && [ -n "$head" ]; then
    for sha in $(git rev-list --reverse --topo-order "$head" "^$since" 2>/dev/null); do
      if verdict=$(bash "$gate" --commit "$sha" 2>&1 </dev/null); then
        if [ -z "$unjudged" ] && git merge-base --is-ancestor "$since" "$sha" 2>/dev/null; then
          git update-ref -m "operon stop gate: audited" "$audited" "$sha" 2>/dev/null
        fi
      else
        # Capped, the way the script caps itself: a message nobody reads is
        # no message.
        unjudged="$unjudged
  $(git log -1 --format='%h %s' "$sha")
$(printf '%s\n' "$verdict" | head -8 | sed 's/^/      /')"
      fi
    done
    if [ -z "$unjudged" ] && git merge-base --is-ancestor "$since" "$head" 2>/dev/null; then
      git update-ref -m "operon stop gate: audited" "$audited" "$head" 2>/dev/null
    fi
  fi
fi

if [ -z "$open_contract" ] && [ -z "$unjudged" ] && [ -z "$audit_failure" ] &&
  [ -z "$approval_findings" ]; then
  say_approvals
  exit 0
fi

# The position: what is refused, and the tree it is refused in. The same
# position is never refused twice; a contract item settled, or any further edit
# to the code, is a new one — so an item still pending at the real finish is
# refused again however long ago the first refusal was. Content, not just
# names: reverting a file has to count. The audit is not in it: see the header.
state=$( { printf '%s\n' "$open_contract"; printf '%s\n' "$approval_findings"; \
           git diff HEAD -- '*.rs' Cargo.toml Cargo.lock; \
           git ls-files -z --others --exclude-standard -- '*.rs' | xargs -0 -r cat --; } 2>/dev/null \
         | shasum | cut -d' ' -f1 )

if [ -z "$unjudged" ] && [ -z "$audit_failure" ]; then
  [ -f "$stamp" ] && grep -qxF "$state" "$stamp" && { say_approvals; exit 0; }
  mkdir -p "$(dirname "$stamp")" 2>/dev/null || exit 0
  printf '%s\n' "$state" >> "$stamp"
fi

# --- and is the application a person can launch built from any of this? --------
#
# Decided by what a change touched, which is knowable on its first day, and
# until now established by reading a diff after everything was committed. Twice
# the answer was "no" by luck; the third time it was "yes" and surfaced only
# when the release gate refused.
#
# Reported alongside what is already being refused, and never a refusal of its
# own: a stale bundle is not something the session's next edit can answer, and a
# gate that blocks every stop until someone repackages is the failure mode, not
# the fix.
#
# AFTER the stamp, and that ordering is the whole of this block's safety. This
# is the first thing this hook asks that is not inside the repository —
# `/Applications/Operon.app` can sit on a mount that does not answer — and a
# Stop hook that blocks blocks the session. Asked before the stamp, a hang would
# also mean the position was never recorded, so the next stop would re-enter the
# identical hang and the file's own promise to refuse once and never twice would
# not hold on this path. Stamped first, a hang costs one stop.
#
# Bounded as well, because one stop is still a stop: perl's `alarm` is preserved
# across `exec`, which is the portable timeout on a machine with no `timeout(1)`.
# Only the check's own exit 1 — behind, or a stamp this repository has never
# heard of — produces a report. Exit 0 covers "no bundle", "no stamp" and
# "current", none of which is news; the deadline expiring (142) and a missing
# perl (127) are silence, like every other way this question can fail to get an
# answer.
installed_report=""
if [ -f scripts/check-installed-build.sh ]; then
  installed_report=$(perl -e 'alarm shift; exec @ARGV' \
    "${OPERON_INSTALLED_CHECK_SECONDS:-5}" bash scripts/check-installed-build.sh 2>/dev/null)
  [ "$?" = 1 ] || installed_report=""
fi

{
  # First, because it changes how everything under it should be read.
  if [ -n "$hook_repository" ] && [ -n "$judged_tree" ] && [ "$hook_repository" != "$judged_tree" ]; then
    printf 'Stop gate: this hook lives in another checkout.\n\n  hook:  %s\n  tree:  %s\n\n' \
      "$hook_repository" "$judged_tree"
    cat <<'CHECKOUT'
Everything below is about the tree, not about the hook. If the hook itself is
what is wrong, fixing it needs a session whose working directory is that
checkout — editing it from here would be vouching for another repository.

CHECKOUT
  fi

  if [ -n "$open_contract" ]; then
    printf 'Stop gate: an open change still has machine contract items pending.\n\n%s\n\n' "$open_contract"
    cat <<'CONTRACT'
Each is what that change said "done" would mean. Run it and record the verdict in
its state.yaml as passed, or mark it waived with the reason on the same line.

CONTRACT
  fi

  if [ -n "$approval_findings" ]; then
    printf 'Stop gate: an approval is claimed that the ledger does not back.\n\n%s\n\n' "$approval_findings"
    cat <<'APPROVALS'
Have the evaluator judge the artifact as it stands and append its line, or put
the question to a person — .claude/skills/sdlc/references/approval.md says which.

APPROVALS
  fi
  [ -n "$approval_summary" ] && printf '%s\n\n' "$approval_summary"

  if [ -n "$unjudged" ]; then
    printf 'Stop gate: the branch carries a commit the review gate did not pass (mark: %s).\n%s\n\n' "$audited" "$unjudged"
    cat <<'AUDIT'
Each reached the branch around .githooks/reference-transaction. Amend a record
in, revert it, or say why it stays and move the mark past it by hand; the mark
is the ref named above, and moving it back by hand is how a commit it passed
over gets read again. This one repeats at every stop until the branch is clean,
except the retry Claude Code makes inside this turn.

AUDIT
  fi

  if [ -n "$audit_failure" ]; then
    printf 'Stop gate: the branch audit could not run — %s.\n\n' "$audit_failure"
    cat <<'NOAUDIT'
Nothing has read the commits on this branch since the last mark. Make the script
reachable — a checkout that carries scripts/check-review.sh, or a writable
TMPDIR — and stop again.

NOAUDIT
  fi

  if [ -n "$installed_report" ]; then
    printf 'Stop gate: the installed application is not built from this repository.\n\n%s\n\n' \
      "$installed_report"
  fi

  cat <<'TAIL'
If a gate fails, that failure is the work — not an obstacle to report around. If
something genuinely cannot be verified here, say so plainly and why; this gate
will not stop you a second time on this position.
TAIL
} >&2
exit 2
