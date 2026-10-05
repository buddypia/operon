#!/usr/bin/env bash
# Has this change been reviewed, by the reviewers it needs, on the diff it is?
#
# Stage 5's gate. `REVIEW.md` used to end at a person reading a diff, and a change
# that was never reviewed left the repository in the same state as one that was
# reviewed and approved: same commit, same green suite, nothing recording which
# happened. That is a single point of failure with no signal when it fails.
#
# What a test can decide is already decided by the suite. What is left is
# judgement, and judgement is delegated to the definitions in .claude/agents/ —
# but their answer is only worth keeping if it is bound to the diff it was about.
# So a verdict carries the digest of what it judged, and a verdict that does not
# match the current diff is not a weaker verdict, it is no verdict.
#
#   bash scripts/check-review.sh                 the change in flight, by hand
#   bash scripts/check-review.sh <change-dir>    that one
#   bash scripts/check-review.sh --all           every refusal at once, not just the first
#   bash scripts/check-review.sh --index         what a commit will land: the index
#   bash scripts/check-review.sh --against HEAD^ the diff an amended commit records
#   bash scripts/check-review.sh --commit <sha>  a commit object, as its own diff
#                                                against its first parent; what
#                                                .githooks/reference-transaction runs
#
#   exit 0   ship; stdout is the first-view block — or nothing here was to be judged
#   exit 1   blocked; stdout is ONE reason, because a gate whose first line is a
#            list is a gate whose first line stops being read
#   exit 2   the check could not be run at all
#
# It fails closed, and there is no bypass: an unreviewable change is an unmerged
# change. `the_review_gate_has_no_bypass` in src/tests.rs keeps that true.
#
# What it cannot see, stated here because REVIEW.md points at this file: a commit
# that touches no change directory, carries no dirty one, and stays off every
# surface docs/sdlc/risk.yaml calls high or hands to a reviewer. That is the
# one-line fix the pipeline is not meant to gate. Anything on those surfaces is
# refused without a change directory, and a review.yaml may not land without the
# code it judged, so the way around this gate is a low-risk file and a session
# that chose to take it.
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

# The repository is the one the caller is in, not the one this file sits in:
# the git hook may be running this script out of the main branch's tree, from
# a temporary file, for a checkout that has none.
repo_root=$(git rev-parse --show-toplevel 2>/dev/null) \
  || repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || {
  echo "the repository root could not be resolved"
  exit 2
}
cd "$repo_root" || { echo "cannot enter $repo_root"; exit 2; }

# Output budget. Findings reach a terminal and a hook's systemMessage, both read
# in full, and REVIEW.md caps a review at five nits for the reason this cap
# exists: volume is how a review stops being read. `cut -c` counts characters
# under the hook's UTF-8 locale, so the ceiling is characters, not bytes.
MAX_FINDINGS=5
FINDING_CHARS=200

# Words that mean something only inside this review machinery. A `person:`
# line that needs one of them has not been translated yet (sdlc 085).
HARNESS_WORDS='nits?|digest|carried|carry|rounds?|important'
# Hiragana and katakana (U+3040–U+30FF start with E3 81–83) and the common
# kanji block (U+4E00–U+9FFF start with E4–E9).
JAPANESE_LEAD_BYTES=$'\xe3[\x81-\x83]|[\xe4-\xe9]'

# One line per carried finding: `<id> US <person line> US <reviewer's text>`,
# the person line empty when the finding has none. US, the unit separator, and
# not a tab: `read` folds a run of tabs into one, and an empty person line
# would vanish into the field after it. The id is what precedes the
# first ` · `, the reviewer's text everything after `  - `. Reads the record
# the rest of this script judged, `$review_text`.
carried_items() {
  printf '%s\n' "$review_text" | sed -n '/^carried:/,/^[a-z]/p' | awk '
    function flush() { if (raw != "") { id = raw; sub(/ · .*/, "", id); print id "\037" person "\037" raw } }
    /^  - / { flush(); raw = substr($0, 5); person = ""; next }
    /^    person: "/ { person = $0; sub(/^    person: "/, "", person); sub(/"[[:space:]]*$/, "", person); next }
    END { flush() }'
}

# What is being judged. `byhand` is the "run it while reviewing" case: the index
# if anything is staged, otherwise the working tree. `--index` is the same
# question asked of the index alone — what a plain commit would land. `--commit`
# is what .githooks/reference-transaction passes from inside git, once the
# commit object exists and before the branch moves to it: the commit's own
# tree, judged against its first parent, which is the diff it records whether
# it was plain, amended, `-a`, a pathspec, or `-C` from another directory. A
# verdict about the working tree is a verdict about something other than what
# lands — the substitution this gate exists to remove, one layer down.
show_all=0
mode=byhand
base=HEAD
commit=""
target=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --all) show_all=1 ;;
    --index) mode=index ;;
    # An amended commit records its diff against HEAD's parent. The verdict has
    # to be about that diff, or a record from the commit being amended away
    # would describe half of what lands.
    --against) shift; base="${1:-}" ;;
    --commit) shift; commit="${1:-}"; mode=commit ;;
    "") ;;
    *) target="${1%/}" ;;
  esac
  shift
done
empty_tree=4b825dc642cb6eb9a060e54bf8d69288fbee4904
if [ "$mode" = commit ]; then
  git rev-parse --verify --quiet "${commit}^{commit}" >/dev/null 2>&1 \
    || { echo "--commit ${commit}: not a commit"; exit 2; }
  # The first parent, or the empty tree for a root commit.
  base=$(git rev-parse --verify --quiet "${commit}^" 2>/dev/null || echo "$empty_tree")
else
  git rev-parse --verify --quiet "${base}^{commit}" >/dev/null 2>&1 \
    || { echo "--against ${base}: not a commit"; exit 2; }
fi

# --- what the commit will record ---------------------------------------------
#
# Every file this gate reads — the record, the route table, the risk table, the
# state, the reviewer definitions — is read from where the commit reads it. In
# `--index` mode that is the index; a working-tree edit to routes.yaml that
# lowers a route to `none` and is never staged would otherwise lower the
# required set for this commit and leave no trace in it. By hand, the working
# tree is what there is.
landing() {
  case "$mode" in
    index) git show ":$1" 2>/dev/null ;;
    commit) git show "${commit}:$1" 2>/dev/null ;;
    *) cat "$1" 2>/dev/null ;;
  esac
}
# The same file as the base has it. The tables that decide what a diff needs —
# the risk surfaces, the route's reviewer set — are read from both sides and the
# stricter side wins, so a commit that lowers a table is still judged by the
# table it lowered. A commit is the one place that can edit its own judge.
landed_before() {
  git show "${base}:$1" 2>/dev/null
}
lands() {
  case "$mode" in
    index) git cat-file -e ":$1" 2>/dev/null ;;
    commit) git cat-file -e "${commit}:$1" 2>/dev/null ;;
    *) [ -f "$1" ] ;;
  esac
}
# `  key: "value"`, with trailing blanks or a comment tolerated after the quote.
field() { printf '%s\n' "$1" | sed -n "s/^  $2: \"\([^\"]*\)\" *\(#.*\)\{0,1\}$/\1/p" | head -1; }

# --- the digest: which diff is this ------------------------------------------
#
# Every path except the change directories. Including a change's own paper trail
# would invalidate the verdict at the moment the verdict was written down, which
# is a mechanism nobody can use. What the exclusion gives up — a spec that claims
# something false — is covered by the suite instead: harness_documents_only_
# name_paths_that_exist and scripts/check-readiness.sh both run before this gate
# is reached.
#
# Two things settled the index over the working tree for the hook:
#
#   - `git diff HEAD` cannot see an untracked file at all. This script was
#     invisible to its own digest until it was staged.
#   - In a tree two sessions share, the working tree carries the other session's
#     uncommitted work, so their save invalidated a verdict that was never about
#     their code. Refusing that is wrong in the safe direction, and a gate that
#     is wrong predictably is one people route around.
#
# One git invocation, chosen once, piped straight into the digest. Capturing the
# diff into a variable first would drop its trailing newline, and the digest
# would then be a number nobody outside this script could reproduce — which for
# a value printed in a refusal message is most of its usefulness gone.
EXCLUDE=':(exclude)docs/sdlc/changes'
case "$mode" in
  index) diff_mode=cached ;;
  commit) diff_mode=commit ;;
  *)
    if git -c core.quotePath=false -c color.ui=false diff --cached --quiet "$base" -- . "$EXCLUDE" 2>/dev/null; then
      diff_mode=worktree
    else
      diff_mode=cached
    fi
    ;;
esac
run_diff() {
  # `--text`: the `-diff` attribute makes git print `Binary files … differ` and
  # no content, so the `Command::new`/`unsafe` trigger below saw no `+` lines
  # and never fired — while `--name-only` stayed correct and the `index` line
  # kept the digest moving, so neither freshness nor surface classification
  # noticed. It arrives through `.git/info/attributes` (untracked, shared by
  # every worktree), an unadded `.gitattributes`, or `core.attributesFile`:
  # three paths that leave nothing in any commit. Measured: one matching added
  # line without the attribute, none with it, one again with `--text`. It is
  # on the digest too, deliberately — what the gate can read is part of what it
  # judged, so setting the attribute must move the digest rather than quietly
  # shrink what the digest covers.
  # `--no-ext-diff --no-textconv`: with `diff.external` set, or
  # `GIT_EXTERNAL_DIFF` in the environment, git prints nothing of its own and
  # the program's output instead. `GIT_EXTERNAL_DIFF=/usr/bin/true` made this
  # digest `e3b0c44298fc1c14` — the sha256 of empty input — for every diff, so
  # one recorded verdict at that digest was fresh forever, and the trigger that
  # widens the reviewer set on an added `Command::new` never fired either.
  # git exits 0, so the failure check below sees nothing wrong. A textconv
  # filter is the same mechanism, one path at a time.
  # `-c core.quotePath=false`: a path with a non-ASCII byte is otherwise
  # printed quoted and octal-escaped, and matches no surface.
  case "$diff_mode" in
    cached) git -c core.quotePath=false -c color.ui=false diff --no-ext-diff --no-textconv --text --cached "$base" "$@" -- . "$EXCLUDE" 2>/dev/null ;;
    commit) git -c core.quotePath=false -c color.ui=false diff --no-ext-diff --no-textconv --text "$base" "$commit" "$@" -- . "$EXCLUDE" 2>/dev/null ;;
    *) git -c core.quotePath=false -c color.ui=false diff --no-ext-diff --no-textconv --text "$base" "$@" -- . "$EXCLUDE" 2>/dev/null ;;
  esac
}
# git's own failure has to be visible before the hash is taken: `shasum` hashes
# empty input as readily as a diff, so a git that could not run would otherwise
# yield a digest — of nothing — and the gate would go on to judge it.
run_diff >/dev/null || { echo "the current diff could not be read; is this a git repository?"; exit 2; }
digest=$(run_diff | shasum -a 256 | cut -c1-16)
changed=$(run_diff --name-only)

# --- which surfaces the diff is on -------------------------------------------
#
# Surface → reviewer. This is the table under "What the gate requires" in
# REVIEW.md, and the surface names are risk.yaml's. Two guards in src/tests.rs
# hold the three copies together: `the_review_gate_names_surfaces_that_risk_yaml_
# still_has` fails when an arm here names a surface risk.yaml no longer has, and
# `the_review_gate_requires_the_reviewers_review_md_names` fails when this
# mapping and REVIEW.md's table disagree in either direction. What neither
# guards is a surface added to risk.yaml with no reviewer at all — that is a
# policy decision, made in REVIEW.md's table, and the second guard then makes
# this file follow it.
reviewer_for_surface() {
  case "$1" in
    store | bundle-swap) echo durability-reviewer ;;
    unsafe-and-path | subprocess | gate-configuration) echo subprocess-safety-reviewer ;;
    dependencies) echo rust-reviewer ;;
    *) echo "" ;;
  esac
}

# One line per risk.yaml surface the diff touches: `<surface> <tier> <reviewer>`,
# reviewer `-` when the surface has none. Both sides of the diff supply
# surfaces, so a surface removed by this very diff still classifies it.
touched_surfaces() {
  printf '%s\n' "$risk_tables" | grep -E '^  [a-z-]+: "' | sort -u | while IFS= read -r line; do
    surface=${line%%:*}
    surface=$(printf %s "$surface" | tr -d " ")
    tuple=$(printf '%s' "$line" | sed -n 's/.*"\(.*\)".*/\1/p')
    set -f
    set -- $tuple
    set +f
    [ "$#" -eq 4 ] || continue
    tier=$1
    paths=$4
    hit=0
    IFS=',' read -ra candidates <<<"$paths"
    for candidate in "${candidates[@]}"; do
      while IFS= read -r file; do
        [ -z "$file" ] && continue
        case "$file" in
          "$candidate" | "$candidate"/*) hit=1 ;;
        esac
      done <<<"$changed"
    done
    [ "$hit" -eq 1 ] || continue
    reviewer=$(reviewer_for_surface "$surface")
    printf '%s %s %s\n' "$surface" "$tier" "${reviewer:--}"
  done
}
# A risk table that does not land — removed or renamed in the index — would
# classify nothing and let every surface through. That is not "no surfaces
# touched"; it is a gate that cannot tell, and it says so.
# Captured once, not piped into `grep -q`: under `pipefail` a reader that stops
# early hands the writer a SIGPIPE, and the check would fail on its own success.
risk_tables=$( { landing docs/sdlc/risk.yaml; landed_before docs/sdlc/risk.yaml; } )
has_surfaces=$(printf '%s\n' "$risk_tables" | grep -E '^  [a-z-]+: "')
[ -n "$has_surfaces" ] \
  || { echo "docs/sdlc/risk.yaml has no surfaces in what this commit lands, so nothing can be classified"; exit 2; }
surfaces=$(touched_surfaces)

# The subprocess reviewer's subject is not a directory. Seven `Command::new` call
# sites live outside the `subprocess` surface's paths — in the drawing module,
# the history reader, the language detector — so a diff is also read for what it
# adds: a new child, a raw `.output()`, or an `unsafe` block pulls the reviewer
# in whatever file it lands in. REVIEW.md's table says so in the same words.
spawns_or_unsafe=0
if run_diff | grep -a -E '^\+.*(Command::new|\.spawn\(|\.output\(|\bunsafe\b)' >/dev/null; then
  spawns_or_unsafe=1
fi

# --- which change is being judged --------------------------------------------
#
# Not "the first one with an in-flight status". That is what guard-stage.sh does
# and it does not survive contact with this repository: 23 changes sit at
# `reviewing` because that is where a change stops when nobody gets to it, which
# is the exact failure this gate exists for. Selecting by status would have made
# every commit answer for change 008.
#
# The diff says which change it is. A session working on a change edits that
# change's own directory, so the change directory the tree touches is the one
# being committed. Staged first — a commit stages the change it is committing,
# and two sessions working in one tree both have dirty change directories while
# only one of them is being committed — then dirty or untracked, which is the
# session that wrote its verdict and forgot to stage it. Falling back to a status
# scan would re-introduce the guess, so it does not.

if [ -z "$target" ]; then
  # Against the same base as the digest: an amend's change directory may be
  # identical to HEAD's and still be the one the amended commit records. A
  # commit object is judged from its own diff alone — nothing outside it is
  # part of what it records.
  if [ "$mode" = commit ]; then
    touched=$(git -c core.quotePath=false -c color.ui=false diff "$base" "$commit" --no-renames --name-only -- docs/sdlc/changes 2>/dev/null |
      sed -n 's|^docs/sdlc/changes/\([^/]*\)/.*|\1|p' | sort -u)
  else
    touched=$(git -c core.quotePath=false -c color.ui=false diff --cached "$base" --no-renames --name-only -- docs/sdlc/changes 2>/dev/null |
      sed -n 's|^docs/sdlc/changes/\([^/]*\)/.*|\1|p' | sort -u)
  fi
  if [ -z "$touched" ] && [ "$mode" != commit ]; then
    touched=$(git status --porcelain -- docs/sdlc/changes 2>/dev/null |
      sed -n 's|^...docs/sdlc/changes/\([^/]*\).*|\1|p' | sort -u)
  fi
  count=$(printf '%s' "$touched" | grep -c . || true)
  # A scrub. Removing a name from the repository means editing the records of
  # changes that are already closed, and those cannot be committed one at a
  # time: a review.yaml may not land without code, and a directory may not be
  # deleted. So one change may own the scrub, by adding a `scrub-names` file in
  # this very diff, and the other directories it touches are judged by two
  # questions only — does anything they add, or any path they add, still carry a
  # listed name, and did they lose one. That lets a record lose a name and
  # nothing else this gate protects: the owner's own review.yaml is still read
  # below, and still bound to the digest. By hand there is no commit to judge,
  # so a scrub is not recognised there.
  # A merge is judged by the exemption below it, as before; a merge cannot be
  # split, so the scrub block must not turn it into a refusal.
  scrub_merge=0
  if [ "$mode" = commit ] && git rev-parse --verify --quiet "${commit}^2" >/dev/null 2>&1; then scrub_merge=1; fi
  if [ "$count" -gt 1 ] && [ "$mode" != byhand ] && [ "$scrub_merge" -eq 0 ]; then
    if [ "$mode" = commit ]; then scrub_range=("$base" "$commit"); else scrub_range=(--cached "$base"); fi
    scrub_diff() {
      git -c core.quotePath=false -c color.ui=false diff --no-ext-diff --no-textconv --text "${scrub_range[@]}" "$@"
    }
    scrub_added=$(scrub_diff -M --diff-filter=A --name-only -- docs/sdlc/changes) || { echo "the diff could not be read"; exit 2; }
    owners=$(printf '%s\n' "$scrub_added" | sed -n 's|^docs/sdlc/changes/\([^/]*\)/scrub-names$|\1|p')
    if [ -n "$owners" ] && [ "$(printf '%s\n' "$owners" | wc -l | tr -d ' ')" -eq 1 ]; then
      scrub_owner=$owners
      scrub_list=$(landing "docs/sdlc/changes/$scrub_owner/scrub-names" |
        sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | grep -v -e '^#' -e '^$')
      scrub_all=$(scrub_diff -M --name-status -- docs/sdlc/changes) || { echo "the diff could not be read"; exit 2; }
      scrub_bad=""
      if [ -n "$scrub_list" ]; then
        while IFS= read -r scrub_dir; do
          [ "$scrub_dir" = "$scrub_owner" ] && continue
          scrub_content=$(scrub_diff -M -U0 -- "docs/sdlc/changes/$scrub_dir") || { echo "the diff could not be read"; exit 2; }
          # Statuses come from the whole tree so a directory rename reads as a
          # rename, not as a deletion here and an addition there.
          scrub_paths=$(printf '%s\n' "$scrub_all" | SCRUB_PREFIX="docs/sdlc/changes/$scrub_dir/" LC_ALL=C awk -F'\t' \
            'index($2, ENVIRON["SCRUB_PREFIX"]) == 1 || ($1 ~ /^R/ && index($3, ENVIRON["SCRUB_PREFIX"]) == 1)')
          scrub_gained=$(
            printf '%s\n' "$scrub_content" | LC_ALL=C awk '/^diff --git /{h=1;next} /^@@/{h=0;next} h==0 && /^\+/{print substr($0,2)}'
            printf '%s\n' "$scrub_paths" | LC_ALL=C awk -F'\t' '$1 ~ /^[AMR]/ {print $NF}'
          )
          scrub_lost=$(
            printf '%s\n' "$scrub_content" | LC_ALL=C awk '/^diff --git /{h=1;next} /^@@/{h=0;next} h==0 && /^-/{print substr($0,2)}'
            printf '%s\n' "$scrub_paths" | LC_ALL=C awk -F'\t' '$1 ~ /^[DR]/ {print $2}'
          )
          # A deletion, or a move into the owner's directory, which is one.
          scrub_gone=$(SCRUB_PREFIX="docs/sdlc/changes/$scrub_dir/" SCRUB_OWNER="docs/sdlc/changes/$scrub_owner/" LC_ALL=C awk -F'\t' \
            '$1 == "D" || ($1 ~ /^R/ && index($2, ENVIRON["SCRUB_PREFIX"]) == 1 && index($3, ENVIRON["SCRUB_OWNER"]) == 1) {print "x"}' <<<"$scrub_paths")
          if [ -n "$scrub_gone" ]; then
            scrub_bad="${scrub_bad}${scrub_dir}: ファイルを削除しています
"
          elif LC_ALL=C grep -q -i -F -f <(printf '%s\n' "$scrub_list") <<<"$scrub_gained"; then
            scrub_bad="${scrub_bad}${scrub_dir}: 名前を足しています
"
          elif ! LC_ALL=C grep -q -i -F -f <(printf '%s\n' "$scrub_list") <<<"$scrub_lost"; then
            scrub_bad="${scrub_bad}${scrub_dir}: 名前を 1 つも消していません
"
          fi
        done <<<"$touched"
        if [ -z "$scrub_bad" ]; then
          touched=$scrub_owner
          count=1
        else
          echo "❌ BLOCK — ${scrub_owner} の scrub は、他の変更ディレクトリを名前の削除以外に使っています
$(printf '%s' "$scrub_bad" | sed 's/^/    /')"
          exit 1
        fi
      fi
    fi
  fi
  if [ "$count" -gt 1 ]; then
    if [ "$mode" = commit ] && git rev-parse --verify --quiet "${commit}^2" >/dev/null 2>&1; then
      exit 0
    fi
    echo "この diff は複数の変更ディレクトリに触れています。1 コミットに 2 つの変更は、このゲートが解く問題ではありません:
$(printf '%s\n' "$touched" | sed 's/^/    /')"
    exit 2
  fi
  [ -n "$touched" ] && target="docs/sdlc/changes/$touched"
fi

# No change directory anywhere in the tree. A one-line fix is meant to look like
# this, and gating every small edit is how a gate gets switched off — but the
# surfaces where a mistake is unrecoverable, and the ones REVIEW.md hands to a
# reviewer, may not be edited under that cover. Their diffs need a change and a
# verdict, so a commit that carries neither is refused rather than passed.
if [ -z "$target" ] && [ "$mode" != byhand ] && [ -n "$changed" ]; then
  guarded=$(printf '%s\n' "$surfaces" | awk '$2 == "high" || $3 != "-" { print $1 }' | paste -sd, -)
  [ "$spawns_or_unsafe" -eq 1 ] && guarded="${guarded:+$guarded,}Command::new/unsafe"
  if [ -n "$guarded" ]; then
    echo "❌ BLOCK — 変更ディレクトリのない commit が ${guarded} に触れています
docs/sdlc/risk.yaml がこの surface を high か、レビュアー付きに分類しています。
一行の修正でも、ここは判定なしには着地しません。docs/sdlc/changes/ に変更を開き、
必要なレビュアーの判定を review.yaml に記録してから commit してください。"
    exit 1
  fi
fi
[ -z "$target" ] && exit 0
if [ "$mode" = commit ]; then
  lands "$target/state.yaml" || { echo "no such change directory in $commit: $target"; exit 2; }
else
  [ -d "$target" ] || { echo "no such change directory: $target"; exit 2; }
fi

state_text=$(landing "$target/state.yaml")
[ -n "$state_text" ] || { echo "$target has no state.yaml, so its route cannot be read"; exit 2; }
route=$(field "$state_text" route)
# The route is spliced into a sed pattern below; a route that is not a plain
# word is not a route, whatever it would match.
case "$route" in
  "" | *[!a-z]*) echo "$target/state.yaml: route \`$route\` is not a plain lowercase word"; exit 2 ;;
esac
change=$(basename "$target")
review="$target/review.yaml"

# --- the record, read from where the commit reads it --------------------------
#
# A verdict written to the working tree and never staged would let this gate say
# SHIP while HEAD received the old record, or none — the verdict outliving the
# code it judged, one file over. So a working-tree copy that differs from what
# lands is a refusal, not a source.
review_text=""
review_present=0
review_unstaged=0
review_landing_changed=0
# Read from the record below when there is one. Declared here because `set -u`
# is on and the first-view block prints it on a path that does not go through
# every branch that sets it.
rounds=1
carried=""
# Whether the record is changing in what lands — including being deleted, which
# is why this is asked before asking whether the record lands at all.
case "$mode" in
  index) git diff --no-ext-diff --no-textconv --cached --quiet "$base" -- "$review" 2>/dev/null || review_landing_changed=1 ;;
  commit) git diff --no-ext-diff --no-textconv --quiet "$base" "$commit" -- "$review" 2>/dev/null || review_landing_changed=1 ;;
esac
if lands "$review"; then
  review_text=$(landing "$review")
  review_present=1
  if [ "$mode" = index ]; then
    git diff --no-ext-diff --no-textconv --quiet -- "$review" 2>/dev/null || review_unstaged=1
  fi
elif [ -f "$review" ] && [ "$mode" = index ]; then
  review_unstaged=1
fi

# --- nothing outside the paper trail -----------------------------------------
#
# A commit that lands only the change's own documents — the intent, the spec,
# the plan, a state.yaml moving to done — has no code for a reviewer to judge,
# and asking for a verdict on an empty diff would make every stage 1 commit
# carry an approval of nothing. One exception, and it is the whole point: a
# review.yaml may not land on its own. A verdict committed apart from the code it
# judged is a record in HEAD describing code that is not there.
if [ -z "$changed" ]; then
  if [ "$review_landing_changed" -eq 1 ]; then
    echo "❌ BLOCK — ${change} · 判定だけが commit されようとしています
${review} が変わる（または消える）のに、この commit にはコードの差分がありません。
判定は判断したコードと同じ commit に載せてください。"
    exit 1
  fi
  echo "review gate: ${change} の paper trail のみ。コードの差分がないので判定は不要です。"
  exit 0
fi

# --- which reviewers this diff needs -----------------------------------------
#
# The route says the floor; the surfaces widen it. A route may ask for more than
# the files touched; it may never ask for less, which is why the two are unioned
# rather than one overriding the other.

# The route's reviewer set, from whichever side of the diff asks for more.
# Before change 035 the column said whether a person had to sign off, `yes` or
# `no`; the first commit judged by this gate found that spelling on both sides
# of its diff — the table that renames it was not landed yet — and was refused
# for a route the table did have. The old words are read as the reviewer sets
# they meant: `yes`, a person must look, is `full`; `no` is `none`, and the
# surfaces the diff touches still widen it. A value in neither vocabulary is
# passed over; a route readable on neither side is exit 2.
as_set() { case "$1" in yes) echo full ;; no) echo none ;; *) echo "$1" ;; esac; }
rank() { case "$1" in full) echo 2 ;; craft) echo 1 ;; none) echo 0 ;; *) echo -1 ;; esac; }
reviewer_set=""
for side in landing landed_before; do
  set -f
  set -- $("$side" docs/sdlc/routes.yaml | sed -n "s/^  $route: \"\(.*\)\"$/\1/p" | head -1)
  set +f
  # Exactly five, and not "at least five". The copy of this script that judges a
  # commit is `refs/heads/main:scripts/check-review.sh`, which
  # .githooks/reference-transaction runs instead of the tree's own — a commit is
  # the one place that can edit its own judge. That copy requires exactly five,
  # so a tolerant parser here would only hide a route table the real judge
  # cannot read. Measured: change 064 added a sixth column, this script accepted
  # it, and every commit with a code diff on that branch was refused for a route
  # the table plainly had. Lesson 039.
  [ "$#" -eq 5 ] || continue
  candidate=$(as_set "$5")
  [ "$(rank "$candidate")" -ge 0 ] || continue
  if [ -z "$reviewer_set" ] || [ "$(rank "$candidate")" -gt "$(rank "$reviewer_set")" ]; then
    reviewer_set=$candidate
  fi
done
if [ -z "$reviewer_set" ]; then
  echo "route \`$route\` is not in docs/sdlc/routes.yaml on either side of this diff, or is not the 5 positional values with a reviewer set of full/craft/none"
  exit 2
fi

# The review-round ceiling, from docs/sdlc/review-rounds.yaml — its own file
# rather than a sixth column, for the reason just above. Read from the same two
# sides and the STRICTER, the smaller number, wins, for the mirror of the reason
# the reviewer set takes the wider one: raising your own ceiling inside the diff
# being judged is the move this has to be closed against. A route with no row on
# either side is 2, because a table that refuses to load must not be able to
# block every commit in the repository.
round_ceiling=""
for side in landing landed_before; do
  candidate=$("$side" docs/sdlc/review-rounds.yaml |
    sed -n "s/^  $route: \"\([0-9][0-9]*\)\" *\(#.*\)\{0,1\}$/\1/p" | head -1)
  [ -n "$candidate" ] || continue
  if [ -z "$round_ceiling" ] || [ "$candidate" -lt "$round_ceiling" ]; then
    round_ceiling=$candidate
  fi
done
[ -n "$round_ceiling" ] || round_ceiling=2

required=""
need() { case " $required " in *" $1 "*) ;; *) required="$required $1" ;; esac; }

case "$reviewer_set" in
  full) need rust-reviewer; need subprocess-safety-reviewer; need durability-reviewer ;;
  craft) need rust-reviewer ;;
  none) ;;
  *) echo "route \`$route\` names reviewer set \`$reviewer_set\`, which is not full/craft/none"; exit 2 ;;
esac

while read -r _ _ reviewer; do
  [ -n "$reviewer" ] && [ "$reviewer" != "-" ] && need "$reviewer"
done <<<"$surfaces"
[ "$spawns_or_unsafe" -eq 1 ] && need subprocess-safety-reviewer

required=$(printf '%s' "$required" | tr -s ' ' | sed 's/^ //;s/ $//')

# --- the refusals, cheapest first --------------------------------------------
#
# Ordered so that fixing the reason that is named makes the next one answerable
# rather than merely revealing it: an unstaged or absent review.yaml makes the
# rest unanswerable, a missing reviewer makes its verdict unanswerable, and so
# on. The sequence terminates instead of ping-ponging.

problems=()
note() { problems+=("$1"); }

if [ -n "$required" ] && [ "$review_unstaged" -eq 1 ]; then
  note "❌ BLOCK — ${change} · review.yaml が stage されていません
${review} の working tree の内容と、この commit が記録する内容が違います。
判定は判断したコードと同じ commit に載せてください: git add ${review}"
fi

if [ -n "$required" ] && [ "$review_present" -eq 0 ] && [ "$review_unstaged" -eq 0 ]; then
  note "❌ BLOCK — ${change} · レビュー記録がありません
${review} が必要です。必要なレビュアー: ${required}
それぞれを走らせ、判定と diff の digest (${digest}) を書いてください。"
fi

if [ "$review_present" -eq 1 ]; then
  recorded_digest=$(printf '%s\n' "$review_text" | sed -n 's/^digest: "\([^"]*\)" *\(#.*\)\{0,1\}$/\1/p' | head -1)

  # Every recorded line is judged, not only the required ones. A verdict
  # attributed to a reviewer that does not exist is the shape a fabricated
  # approval takes, and a do-not-approve from a reviewer the route did not ask
  # for is still a do-not-approve: it was run, it found something, and a gate
  # that shipped over it while printing its Important count would be lying in
  # its own first line.
  seen=""
  while IFS= read -r recorded; do
    reviewer=${recorded%%:*}
    reviewer=$(printf %s "$reviewer" | tr -d " ")
    [ -z "$reviewer" ] && continue
    # One line per reviewer. `field` reads the first, so a second line for the
    # same name — an approve above a do-not-approve — would be the one nobody
    # reads, and the display would show the reviewer twice as passing.
    case " $seen " in
      *" $reviewer "*)
        note "❌ BLOCK — ${change} · ${reviewer} の判定が 2 行あります
review.yaml はレビュアーごとに 1 行です。どちらが本当の判定か、記録からは読めません。"
        continue ;;
    esac
    seen="$seen $reviewer"
    if ! lands ".claude/agents/${reviewer}.md"; then
      note "❌ BLOCK — ${change} · ${reviewer} は定義されていないレビュアーです
.claude/agents/${reviewer}.md がありません。判定は実在するレビュアーのものでなければ、
記録されていることが担保にならなくなります。"
      continue
    fi
    line=$(field "$review_text" "$reviewer")
    set -f
    set -- $line
    set +f
    if [ "$#" -ne 3 ]; then
      note "❌ BLOCK — ${change} · ${reviewer} の判定が <verdict> <important> <nits> の 3 値ではありません
読めた値: ${line}"
      continue
    fi
    case "$2$3" in
      *[!0-9]*)
        note "❌ BLOCK — ${change} · ${reviewer} の Important と nits が数字ではありません
読めた値: ${line}"
        continue
        ;;
    esac
    case "$1" in
      approve | approve-with-nits) ;;
      do-not-approve)
        note "❌ BLOCK — ${change} · ${reviewer} が do-not-approve を出しています
Important ${2} 件。判定を出した根拠を直してから再レビューしてください。"
        continue
        ;;
      *)
        note "❌ BLOCK — ${change} · ${reviewer} の判定 \`${1}\` は approve / approve-with-nits / do-not-approve のいずれでもありません"
        continue
        ;;
    esac
    if [ "$2" != "0" ]; then
      note "❌ BLOCK — ${change} · ${reviewer} が Important を ${2} 件出しています
REVIEW.md の Important は「間違っている、危険、文書化された保証を壊す」です。
nit は blocking しませんが、Important は直してから commit してください。"
      continue
    fi
  done < <(printf '%s\n' "$review_text" | grep -E '^  [a-z-]+: "')

  for reviewer in $required; do
    if [ -z "$(field "$review_text" "$reviewer")" ]; then
      note "❌ BLOCK — ${change} · ${reviewer} の判定がありません
このルート (${route}) と変更されたファイルは ${reviewer} を必要とします。
必要なレビュアー: ${required}"
    fi
  done

  # A record that is required, or one that is being written in this commit,
  # has to be about this diff. A record nobody asked for and nobody touched —
  # a docs commit on a change reviewed earlier — is left alone and not shown.
  if { [ -n "$required" ] || [ "$review_landing_changed" -eq 1 ]; } \
    && [ "$recorded_digest" != "$digest" ]; then
    note "❌ BLOCK — ${change} · 判定が古い diff のものです
判定は diff ${recorded_digest:-（記録なし）} のもの、現在の diff は ${digest}。
古い承認は無い承認より悪いので、再レビューしてから commit してください。"
  fi

  # --- and how many times has this gone back? ---------------------------------
  #
  # The refusal above is the one that loops. It is correct — a verdict about
  # another diff is not a verdict — but combined with taking a nit it does not
  # terminate: the edit that satisfies a nit moves the digest, the digest voids
  # both approvals, the next round produces another nit. Change 060 spent
  # nineteen rounds there and its last behaviour change was round 17.
  #
  # So `rounds:` is counted against the route's ceiling, and past it there is
  # exactly one way on that is not another round: move the open nits into
  # `carried:` and land. That makes this a forcing function toward shipping
  # rather than one more obstacle — which matters, because the change it stops
  # is by definition one that has already been approved several times.
  #
  # `rounds:` is self-reported, like `attempts` in state.yaml. There is no way
  # to derive it: every round of the loop happens before any commit, so this
  # script sees only the last one. What it can do is be impossible to miss
  # while the loop is running — the count prints on every run, and this script
  # is run by hand every round. That is the design: the number is a mirror
  # first and a gate second.
  rounds=$(printf '%s\n' "$review_text" | sed -n 's/^rounds: \([0-9]*\) *\(#.*\)\{0,1\}$/\1/p' | head -1)
  [ -n "$rounds" ] || rounds=1
  carried=$(printf '%s\n' "$review_text" | sed -n '/^carried:/,/^[a-z]/p' | sed -n 's/^  - //p')
  if [ "$rounds" -gt "$round_ceiling" ] && [ -z "$carried" ]; then
    note "❌ BLOCK — ${change} · レビューが ${rounds} 巡目です（route ${route} の上限は ${round_ceiling}）
ここから先、もう一巡しても終わりません。nit を取り込むと digest が動き、
その nit に付いてきた承認が消えて、次の巡回が買われるだけです。

出口は 1 つだけで、それは「もう一度レビューする」ではありません:
review.yaml に carried: を書き、未処理の nit をそこへ移して着地させてください。
本当に Important が残っているなら、それは nit ではないので直してください。
どちらでもないなら docs/sdlc/templates/handoff.md を書いて人に渡してください。"
  fi

  # --- and can a person read what was left? ----------------------------------
  #
  # A carried finding is the reviewer's shorthand, kept verbatim for the change
  # that takes it up. It is not something a person can act on, and it was the
  # only text there was: closing change 083, a session reported "carry した nit
  # が7件" and a list of English file-and-line notes (sdlc 085). So each one
  # carries a `person:` line — plain Japanese, what goes wrong and whether the
  # person has to act — and that is the line a report is written from. Judged
  # when the record is judged at all, like the digest above.
  if { [ -n "$required" ] || [ "$review_landing_changed" -eq 1 ]; } && [ -n "$carried" ]; then
    while IFS=$'\037' read -r id person _; do
      [ -n "$id" ] || continue
      problem=""
      if [ -z "$person" ]; then
        problem="人が読む説明がありません"
      # Kana or kanji, by their UTF-8 lead bytes — not "any non-ASCII byte",
      # which the ` · ` in every carried id and an em dash both satisfy, so a
      # reviewer's English line pasted here would pass as the person's.
      elif ! printf '%s' "$person" | LC_ALL=C grep -E "$JAPANESE_LEAD_BYTES" >/dev/null; then
        problem="説明に日本語がありません"
      else
        jargon=$(printf '%s' "$person" | LC_ALL=C grep -i -o -w -E "$HARNESS_WORDS" | head -1)
        [ -n "$jargon" ] && problem="説明に、レビューの仕組みの中でしか通じない言葉 \`${jargon}\` が入っています"
      fi
      [ -n "$problem" ] || continue
      note "❌ BLOCK — ${change} · 残した指摘 ${id}: ${problem}
review.yaml の carried: の各項目のすぐ下に、4 字下げで書いてください:
    person: \"<何が・どんな時に起きるか。対応が要るか>\"
レビュアーの原文は、それを直す後の変更のための記録です。人への報告はこの行から書きます。"
    done < <(carried_items)
  fi
fi

if [ "${#problems[@]}" -gt 0 ]; then
  if [ "$show_all" -eq 1 ]; then
    printf '%s\n\n' "${problems[@]}"
  else
    printf '%s\n' "${problems[0]}"
  fi
  exit 1
fi

# --- the first-view block ----------------------------------------------------
#
# A verdict line, then what was checked, then findings only if there are any.
# Whether it shipped is the first thing on the first line, because that is the
# question, and everything after it is the evidence for the answer.

important_total=0
nit_total=0
passes=""
if [ "$review_present" -eq 1 ] && [ -n "$required" ]; then
  while IFS= read -r line; do
    reviewer=${line%%:*}
    reviewer=$(printf %s "$reviewer" | tr -d " ")
    set -f
    set -- $(field "$review_text" "$reviewer")
    set +f
    [ "$#" -eq 3 ] || continue
    important_total=$((important_total + $2))
    nit_total=$((nit_total + $3))
    mark="✓"
    [ "$3" != "0" ] && mark="⚠"
    passes="$passes  ${reviewer%%-*} $mark"
  done < <(printf '%s\n' "$review_text" | grep -E '^  [a-z-]+: "')
fi

printf '✅ SHIP  —  %s\n' "$change"
if [ -n "$required" ]; then
  printf '%s Important · %s Nits · reviewed at diff %s\n' "$important_total" "$nit_total" "$digest"
  # The round count prints even when it is 1 of 2. A number that only appears
  # once it is a problem is a number nobody has a baseline for, and this one's
  # whole job is to be noticed two rounds before it refuses.
  printf 'round %s of %s\n' "$rounds" "$round_ceiling"
  printf '\npasses %s\n' "$passes"
else
  printf '%s Important · %s Nits · diff %s\n' "$important_total" "$nit_total" "$digest"
  printf '\npasses  route %s は決定的な検査のみを要求します\n' "$route"
fi

findings=""
[ -n "$required" ] && findings=$(carried_items | head -n "$MAX_FINDINGS")
if [ -n "$findings" ]; then
  # The person's line where there is one. A record written before 085 has
  # only the reviewer's, and showing that is better than showing nothing.
  printf '\n直さずに残した点（どれも着地は止めない）\n'
  while IFS=$'\037' read -r _ person raw; do
    shown=${person:-$raw}
    printf '  · %s\n' "$(printf '%s' "$shown" | cut -c1-"$FINDING_CHARS")"
  done <<<"$findings"
fi
# operon: end of scripts/check-review.sh
