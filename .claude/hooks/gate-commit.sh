#!/usr/bin/env bash
# PreToolUse(Bash): the commit gate. Stage 4 of the AI-native SDLC — every
# session checks its own work before a human sees it, and the check that must
# hold without exception sits behind a hook rather than a sentence.
#
# AGENTS.md has always said "before packaging, run the three gates". That was a
# promise in prose: a green suite the agent never ran looks exactly like a green
# suite it did. This runs them, at the commit boundary, and blocks on failure.
#
# Placement follows the playbook: build-phase hooks stay fast and file-scoped
# (guard-write.sh), heavier checks belong at the commit stage (here).
#
#   OPERON_ALLOW_TEST_REMOVAL=1   deliberately drop or ignore a test
#
# The review gate — stage 5 — is not here. It is .githooks/reference-transaction,
# run by git inside the ref update, where the commit object is exactly what will
# land whatever the command said. This hook's part in it is to keep git able to
# find it: it installs the trampoline in .githooks/installed/ into the
# repository's own hooks directory before every commit, and refuses the plain
# spellings of a command that would point git away from that directory. Plain
# spellings: six rounds of review of a reader that tried to work out `-a`,
# `--amend`, a pathspec, or a `cd` from the command text are recorded in
# docs/sdlc/changes/035-*/plan.md, and are why nothing here claims to read a
# command exhaustively. What gets past this text is read off the branch itself
# by .claude/hooks/gate-stop.sh before the session ends.
#
# The hook's timeout in .claude/settings.json is 600 seconds, which is generous
# for a warm cache — about a minute for the suite — and can be short of a cold
# one. A timeout is a non-blocking error, so what a timeout skips is the three
# cargo gates and the erosion count; the review gate runs inside git and is not
# in this window at all. On the first build after a `cargo clean`, run the
# cargo gates by hand once.
#
# Every predicate here is `grep … >/dev/null`, never `grep -q`: under `pipefail`
# a `grep -q` that stops reading at its first match hands the pipeline the
# producer's SIGPIPE status, and a match reads as a miss. Entry 018 in
# docs/sdlc/lessons.md.
set -uo pipefail

# Read once and kept, because two fields are needed: the command, and the
# directory the tool reports it will run the command in. `cat | jq` consumed the
# payload and left the second unreadable.
payload=$(cat)
command=$(printf '%s' "$payload" | jq -r '.tool_input.command // empty')
# Read with backslash continuations joined and heredoc bodies removed: a commit
# message that names `core.hooksPath` because that is what the commit does, or
# a file written through `cat <<'EOF'` that spells `git -C dir commit`, is text
# in a body and decides nothing here. Quoted text on the command line itself
# stays, because `-c "core.hooksPath=/x"` is an override in quotes. What is
# decided is decided by git's own hook on the commit object.
# A `<<WORD` inside quotes — `grep -c "<<EOF" file` — is not an operator, and a
# body with no terminator is not a body: in either case nothing may be dropped,
# because what is dropped is read by nobody. The awk exits 1 on an unterminated
# body and the raw text is read instead.
strip_heredocs() {
  awk '
    # A copy of the line with quoted spans and arithmetic expansions blanked to
    # spaces, same length, so offsets still line up with the real line. `<<` is
    # only an operator in neither of those places: `$(( 1 << X ))` is a shift and
    # `"a <<EOF b"` is text, and reading either as a heredoc dropped every line
    # up to a later bare `X` or `EOF` — including the `git commit` — so the hook
    # exited 0 with the three cargo gates, the trampoline install and every
    # override refusal unrun. Round 20 closed only the unspaced `$((1<<X))`
    # spelling by looking at the single character in front; three more spellings
    # were still open, measured.
    function blanked(s,   out, i, n, c, rest) {
      # Quoted text, arithmetic and comments are not places a heredoc operator
      # can be, so they are blanked to spaces of the same length and the offsets
      # still line up. The state is NOT per line: shell quoting spans newlines,
      # so `echo "note:` on one line leaves the quote open, and a `<<EOF` on the
      # next is text rather than an operator. Resetting per record read it as an
      # operator and dropped every line to a later bare `EOF` — which the commit
      # form used throughout this repository, `-m "$(cat <<EOF ... EOF)"`,
      # supplies. Body lines never reach here, so they cannot advance the state.
      # No apostrophe may appear in these comments: the whole program is inside
      # a single-quoted shell word, and one closes it.
      out = ""
      n = length(s)
      for (i = 1; i <= n; i++) {
        c = substr(s, i, 1)
        if (gdepth > 0) {
          if (c == "(") gdepth++
          else if (c == ")") gdepth--
          out = out " "
          continue
        }
        # An ANSI-C quote takes backslash escapes, so an escaped quote does not
        # close it; a plain single quote takes none, and a backslash inside one
        # is a literal. The two states are told apart by the trailing marker.
        if (gq == "\047" || gq == "\047$") {
          if (gq == "\047$" && c == "\\" && i < n) { out = out "  "; i++; continue }
          out = out " "
          if (c == "\047") gq = ""
          continue
        }
        if (c == "\\" && i < n) { out = out "  "; i++; continue }
        if (gq == "\"" && c == "$" && substr(s, i + 1, 1) == "(" \
            && substr(s, i + 2, 1) != "(") {
          gsaved[++gsp] = gq; gq = ""; out = out "  "; i++; continue
        }
        if (gq == "\"") {
          out = out " "
          if (c == "\"") gq = ""
          continue
        }
        if (c == "$" && substr(s, i + 1, 2) == "((") {
          gdepth = 1; out = out "   "; i += 2; continue
        }
        if (c == ")" && gsp > 0) { gq = gsaved[gsp--]; out = out " "; continue }
        if (c == "<" && substr(s, i + 1, 1) == "<" && substr(s, i + 2, 1) != "<") {
          rest = substr(s, i + 2)
          if (match(rest, /^-?[ \t]*[\047"]?[A-Za-z_][A-Za-z0-9_]*[\047"]?/)) {
            out = out "<<" substr(rest, 1, RLENGTH)
            i = i + 1 + RLENGTH
            continue
          }
        }
        if (c == "#" && (i == 1 || substr(s, i - 1, 1) ~ /[ \t;&|(]/)) {
          while (i <= n) { out = out " "; i++ }
          break
        }
        if (c == "$" && substr(s, i + 1, 1) == "\047") {
          gq = "\047$"; out = out "  "; i++; continue
        }
        if (c == "\"" || c == "\047") { gq = c; out = out " "; continue }
        out = out c
      }
      return out
    }
    BEGIN { re = "<<-?[ \t]*[\047\"]?[A-Za-z_][A-Za-z0-9_]*[\047\"]?"; inbody = 0; gq = ""; gsp = 0; gdepth = 0 }
    inbody { t = $0; sub(/^\t+/, "", t); if (t == term) { inbody = 0 }; next }
    {
      probe = blanked($0); word = ""; j = 0
      if (match(probe, re)) {
        after = substr(probe, RSTART + 2, 1)
        # A heredoc operator sits where a redirection can. Reading the single
        # character in front got `1<<X` right and `2<<EOF` wrong; the question is
        # what the whole word in front is. Nothing (whitespace or an operator) is
        # a plain `cat <<EOF`; all digits is a file descriptor, `cat 2<<EOF`;
        # anything holding a `>` or `<` is a chained redirection, `cat >f<<EOF`.
        # A word of any other shape — `a<<b` — is not a redirection at all.
        # Arithmetic and quotes are already blanked out of `probe`, so `1<<X`
        # inside `$(( ))` never reaches this test.
        word = ""
        j = RSTART - 1
        while (j >= 1 && substr(probe, j, 1) !~ /[ \t;&|(]/) {
          word = substr(probe, j, 1) word
          j--
        }
        # `<<<` is a herestring with no body. The pattern can land on its second
        # `<`, which makes the word in front look like a chained redirection, so
        # the character on each side is checked as well as the word.
        before = (RSTART > 1) ? substr(probe, RSTART - 1, 1) : ""
        redirection = (word == "" || word ~ /^[0-9]+$/ || word ~ /[<>]/) \
          && before != "<"
        if (redirection && after != "<") {
          m = substr($0, RSTART, RLENGTH); sub(/^<<-?[ \t]*/, "", m); gsub(/[\047"]/, "", m)
          term = m; inbody = 1
        }
      }
      print
    }
    END { if (inbody) exit 1 }'
}
stripped=$(printf '%s\n' "$command" | strip_heredocs) || stripped=$command
joined=$(printf '%s\n' "$stripped" | sed -e ':a' -e '/\\$/N; s/\\\n/ /; ta')
# The word `git` as a command — `git`, `/usr/bin/git`, `(git`, `; git` — and not
# `.git/` as a path. One spelling, read by every check below; two readers that
# disagreed about what a git invocation is were round 11's finding.
git_word='(^|[^.[:alnum:]_-])git +'

block() {
  printf 'Commit blocked by the gate: %s\n\n%s\n' "$1" "$2" >&2
  exit 2
}

# --- a git command that would hide git's hook -----------------------------------
#
# `git -c core.hooksPath=… <anything>` and a `GIT_CONFIG_*` variable point git at
# no hook for that one command, and git says nothing. A commit is not the only
# command that makes one: a merge, a cherry-pick, a rebase, a pull do too, and
# each reaches the reference-transaction hook as a commit on a branch. Nothing
# here needs the override, so a git command carrying one of its two plain
# spellings is refused on sight — unless every git invocation in the command is
# one that reads and updates no ref, where the override has nothing to hide from,
# or the command is the remedy the refusal below asks for. The word is `git` as
# a command, not `.git/` as a path.
trimmed=$(printf '%s' "$joined" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')
read_only='^(log|show|diff|status|grep|blame|rev-parse|rev-list|merge-base|cat-file|ls-files|ls-tree|for-each-ref|name-rev|describe|shortlog|fsck|count-objects|check-ignore|help|version|--version|config --(get|get-all|get-regexp|list|show-origin)|config -l)( |$)'
updates_refs=0
# The subcommand is the first token past git's own leading options — `-C dir`,
# `--no-pager`, `-c key=value` — which the global CLAUDE.md prescribes over `cd`.
while IFS= read -r invocation; do
  [ -n "$invocation" ] || continue
  rest=$(printf '%s' "$invocation" | sed -E 's/^ ?[^ ]*git +//; s/^((-C ?[^ ]+|-c +[^ ]+|--no-pager|--literal-pathspecs|--no-optional-locks) +)*//')
  printf '%s' "$rest" | grep -E "$read_only" >/dev/null || updates_refs=1
done <<EOF
$(printf '%s' "$joined" | grep -oE "${git_word}[^;&|]*")
EOF
# The way out of a set `core.hooksPath` is to unset it, so that one command is
# excused — but only when it IS the command. `grep`'s `^` and `$` are line
# anchors and `$trimmed` is the whole multi-line invocation, so any command
# could buy its way past the refusal by carrying the rescue on a second line:
# measured, an override plus `git config --unset core.hooksPath` below it went
# from a named refusal to no refusal at all. REVIEW.md says the two plain
# spellings are refused on every git command that can update a ref, and the
# holes it does accept are spellings the filter cannot see — not this one.
rescue_only=0
# On its own line, and compared with `=`: written inline, the `-eq` that
# followed the pipe was read as a grep flag holding a `q` by
# `every_gate_script_reads_its_pipes_to_the_end`. A false positive, but the
# split is clearer anyway and the guard is right that this shape is hard to read.
command_lines=$(printf '%s\n' "$trimmed" | grep -c .)
if [ "$command_lines" = 1 ] \
  && printf '%s' "$trimmed" | grep -E '^git config( --(local|global|worktree|system))? --unset core\.hooksPath$' >/dev/null; then
  rescue_only=1
fi
if [ "$updates_refs" -eq 1 ] \
  && printf '%s' "$joined" | grep -iE 'hooksPath|GIT_CONFIG' >/dev/null \
  && [ "$rescue_only" -eq 0 ]; then
  block "a git command that overrides core.hooksPath or the git config" \
"The review gate is git's reference-transaction hook, and this command points git
away from it for one command. Nothing here needs that; run the command plainly."
fi

printf '%s' "$joined" | grep -E "${git_word}([^ &;|]+ +)*commit( |$)" >/dev/null || exit 0

# --- which tree is this commit going to land in? ------------------------------
#
# Not `CLAUDE_PROJECT_DIR`. That names the project the *session* belongs to, and
# AGENTS.md asks work to land from a worktree — so a session whose project
# directory is the main checkout and whose working directory is a worktree is
# the arrangement this repository prescribes, not an edge case.
#
# Measured in exactly that arrangement, and it did damage rather than merely
# refusing: this hook ran `cargo test --locked` over the main checkout's
# sources, which were another session's uncommitted tree, and reported 25
# failures of which three were tests that do not exist in the tree being
# committed. The same run inherited a repository pointer naming the worktree, so
# that foreign suite's own git tests committed the staged change onto the
# worktree's branch as `initial`, twice. A gate that vouches for the wrong tree
# is worse than no gate: it spends the suite's credibility on code nobody is
# committing, and it reports a failure the committer cannot reproduce or fix.
#
# A commit resolves its repository from the directory it runs in, then from the
# environment, then from the ancestors. So ask git the same question from the
# same place, and gate whatever it answers. The refusals below still hold: a
# commit redirected at another checkout is refused rather than followed, and
# `here` is now the landing tree, which is the directory a `-C` has to agree
# with for these cargo gates to be vouching for the code being committed.
# `the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in` fails when
# this goes back to reading `CLAUDE_PROJECT_DIR` first.
running_in=$(printf '%s' "$payload" | jq -r '.cwd // empty')
[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=.
# `cd --`, because a directory named `-P` passes the `-d` test above and would
# otherwise be read as an option, sending `cd` to $HOME.
landing=$(cd -- "$running_in" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)
cd -- "${landing:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0
export PATH="$HOME/.cargo/bin:$PATH"
# The landing tree is resolved, and from here git finds it from the directory.
# A repository pointer left in the environment would instead reach every git
# the suite starts: a session that exported GIT_DIR (Claude Code's
# EnterWorktree does) had its test fixtures rewrite the shared .git/config and
# commit onto the branch, three times (sdlc 078, 079). The suite drops them
# itself — `the_review_gate_fixture_cannot_reach_the_session_repository` — and
# this is the second layer, for the next spawn that forgets.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_COMMON_DIR GIT_PREFIX

note() { jq -n --arg m "$1" '{systemMessage: $m}'; exit 0; }

# --- a commit aimed at another checkout ----------------------------------------
#
# `--git-dir`, `--work-tree`, `GIT_DIR`, `GIT_WORK_TREE`, or a `-C` that lands
# elsewhere would have this directory's cargo gates vouch for that directory's
# code. `-C` is allowed only as git's first option, only once, and only when it
# resolves to this session's project directory: git applies a later `-C`
# relative to the earlier one, so `-C . -C ../other` is another directory
# spelled through this one, and a `-C` behind `--no-pager` or `-c` is the same
# spelled around the check.
if printf '%s' "$joined" | grep -E -- '--git-dir|--work-tree|GIT_DIR|GIT_WORK_TREE' >/dev/null; then
  block "a commit with --git-dir, --work-tree, GIT_DIR, or GIT_WORK_TREE" \
"This session's cargo gates would vouch for another checkout's code. Commit
from a session whose project directory is that checkout."
fi
here=$(pwd -P)
while IFS= read -r invocation; do
  [ -n "$invocation" ] || continue
  c_count=$(printf '%s\n' "$invocation" | grep -oE '(^| )-C ?("[^"]*"|'"'"'[^'"'"']*'"'"'|[^ ]+)' | grep -c . || true)
  [ "$c_count" -eq 0 ] && continue
  if [ "$c_count" -ne 1 ] || ! printf '%s' "$invocation" | grep -E '^git +-C ?("|'"'"'|[^ ])' >/dev/null; then
    block "a commit with more than one -C, or a -C that is not git's first option" \
"git applies each -C relative to the one before it, and a -C behind another
option is the same directory spelled around this check. Use one -C, first, or none."
  fi
  after=$(printf '%s' "$invocation" | sed -E 's/^git +-C ?//')
  case "$after" in
    \"*) target=${after#\"}; target=${target%%\"*} ;;
    \'*) target=${after#\'}; target=${target%%\'*} ;;
    *) target=${after%% *} ;;
  esac
  there=$(cd "$target" 2>/dev/null && pwd -P) || there=""
  [ "$there" = "$here" ] || block "a commit aimed at another directory ($target)" \
"This session's cargo gates would vouch for another checkout's code. Commit from
a session whose project directory is that checkout, or drop the -C."
done <<EOF
$(printf '%s' "$joined" | grep -oE "${git_word}[^;&|]*" | sed -E 's/^ ?[^ ]*git +/git /')
EOF

# --- the review gate is git's, and git has to be able to find it ---------------
#
# The trampoline in .githooks/installed/ is kept in the repository's own hooks
# directory — `$(git rev-parse --git-path hooks)`, which every linked worktree
# shares and which no checkout, branch switch, or move of the working tree
# removes — and refreshed here at every commit. It runs the tracked hook from
# the main checkout, so a branch cannot downgrade the gate by being committed
# from. A `core.hooksPath` would point git away from that directory, so one
# being set at all is refused: whoever set it is asked to unset it, not silently
# overruled.
# Whole means the last line is there: a prefix cut at a statement boundary is
# non-empty, executable, and exits 0 on everything.
[ -x .githooks/reference-transaction ] && [ "$(tail -n 1 .githooks/reference-transaction 2>/dev/null)" = '# operon: end of .githooks/reference-transaction' ] \
  || block ".githooks/reference-transaction" \
"The tracked review gate is missing, cut short, or not executable, so no commit can be judged."
[ "$(tail -n 1 .githooks/installed/reference-transaction 2>/dev/null)" = '# operon: end of .githooks/installed/reference-transaction' ] \
  || block ".githooks/installed/reference-transaction" \
"The trampoline that runs the review gate from the repository's hooks directory is missing or cut short."
common_hooks=$(git rev-parse --git-path hooks 2>/dev/null)
[ -n "$common_hooks" ] || block "git" "the repository's hooks directory could not be resolved"
hooks_path=$(git config --get core.hooksPath 2>/dev/null || true)
[ -z "$hooks_path" ] || block "core.hooksPath is set to $hooks_path" \
"The review gate lives in the repository's own hooks directory, and core.hooksPath
points git elsewhere. Run: git config --unset core.hooksPath — with --global if
that is where it is set."
# What is installed is the main branch's committed trampoline, and only when
# there is none, this checkout's working copy. Installing the working copy
# unconditionally was an entrance: this runs before the cargo gates, so a
# trampoline edited to decide nothing — marker line intact — was written into
# the shared hooks directory, the suite then went red on its text pins, the
# commit was refused, and the neutered trampoline stayed. That directory is
# shared by every linked worktree and nothing removes it, so every later ref
# update anywhere in the repository passed in silence until the next commit
# from a good tree. The same state arrived with no edit at all, from a timeout
# during `cargo test` or from a checkout of a branch carrying a different one.
# Round 16 settled this order for scripts/check-review.sh; a trampoline is not
# more trustworthy for judging nothing itself.
staged_hook="$common_hooks/.reference-transaction.$$"
# The hook can be killed — its own 600s timeout, a terminated session — and a
# `.reference-transaction.<pid>` left in a shared directory is litter that
# accumulates. git only runs the exact name, so it is harmless, not permanent.
trap 'rm -f "$staged_hook"' EXIT
mkdir -p "$common_hooks" || block "$common_hooks" \
"The repository's hooks directory could not be made."
trampoline_blob=refs/heads/main:.githooks/installed/reference-transaction
if git cat-file -e "$trampoline_blob" 2>/dev/null; then
  # Whole or not at all, and size-checked: a full disk writes a prefix, and a
  # prefix cut at a statement boundary is a trampoline that exits 0 on
  # everything.
  git cat-file blob "$trampoline_blob" > "$staged_hook" 2>/dev/null \
    && [ "$(wc -c < "$staged_hook" | tr -d ' ')" -eq "$(git cat-file -s "$trampoline_blob" 2>/dev/null || echo -1)" ] \
    || block "$common_hooks/reference-transaction" \
"The main branch's trampoline could not be written whole."
else
  cp .githooks/installed/reference-transaction "$staged_hook" \
    || block "$common_hooks/reference-transaction" \
"The trampoline could not be copied into the repository's hooks directory."
fi
[ "$(tail -n 1 "$staged_hook" 2>/dev/null)" = '# operon: end of .githooks/installed/reference-transaction' ] \
  || block "$common_hooks/reference-transaction" \
"The trampoline written into the repository's hooks directory is cut short."
# The mode before the sync, not after it. `fsync` and `F_FULLFSYNC` do flush a
# file's inode metadata, mode included — so what the order buys is not that the
# sync would otherwise miss the bit, but that the mode change is *inside* the
# state the sync forces rather than applied after it. Set it between the sync
# and the rename and only the mode is left unflushed, and git skips a
# non-executable hook with a one-line `hint:` nobody here reads, so the commit
# lands ungated. Reproduced in a throwaway repository.
chmod +x "$staged_hook" || block "$common_hooks/reference-transaction" \
"The trampoline could not be made executable."
# Data and mode on disk before the rename, and the directory entry after it:
# `mv -f` alone leaves APFS free to journal the directory entry while the
# extents are still unwritten, so a crash leaves a zero-byte hook — which bash
# exits 0 on and git takes as approval, for every worktree, until the next
# commit through Claude Code reinstalls it.
#
# `F_FULLFSYNC`, not `fsync(2)`: Apple documents `fsync` as not flushing the
# drive's own cache and directs power-loss durability to `F_FULLFSYNC`, which is
# what `src/store.rs` gets from `sync_all()` on this target. Doing it here
# through the `python3` the directory sync already needs keeps the shell path
# and the Rust path on the same primitive. A machine without `python3` is
# refused rather than quietly given the weaker guarantee: this hook has no
# business installing a gate it cannot make durable, and `REVIEW.md` states the
# guarantee without a footnote.
command -v python3 >/dev/null 2>&1 || block "python3" \
"The trampoline is synced to disk with F_FULLFSYNC through python3, which is not
on PATH. Install it, or run the commit outside Claude Code and accept that the
review gate is not installed."
# Exit 3 means the rename already happened and only the directory sync failed:
# the trampoline is installed, executable and whole, and refusing the commit
# then would be refusing it *because the gate is in place*. `src/store.rs` keeps
# this same distinction as `WriteOutcome::CommittedButNotSynced`; the shell had
# been collapsing the two.
#
# `F_FULLFSYNC` is what Apple documents for power loss, and `fsync(2)` is not —
# but `fcntl(2)` lists it for HFS, FAT, UDF and APFS only, and it raises
# ENOTSUP on the filesystems outside that list and EROFS on a read-only mount.
# Falling back to `fsync` there is a weaker guarantee, so it is said out loud
# rather than taken quietly: a degradation nobody can observe is the same
# failure as the `|| true` this replaced.
sync_status=0
python3 - "$staged_hook" "$common_hooks" <<'SYNC' || sync_status=$?
import fcntl, os, sys

target, directory = sys.argv[1], sys.argv[2]
landed = os.path.join(directory, "reference-transaction")


def durable(fd, what):
    try:
        fcntl.fcntl(fd, fcntl.F_FULLFSYNC)
    except OSError as full:
        os.fsync(fd)
        sys.stderr.write(
            "review gate: %s was flushed with fsync, not F_FULLFSYNC (%s); "
            "this filesystem does not implement it, so a power loss can still "
            "lose the write\n" % (what, full.strerror)
        )


fd = os.open(target, os.O_RDONLY)
try:
    durable(fd, "the trampoline")
finally:
    os.close(fd)
os.replace(target, landed)
# The rename itself, so the entry survives the crash the bytes now survive.
# Past this point the gate is installed, and a failure here is not a reason to
# refuse the commit it would have judged.
try:
    d = os.open(directory, os.O_RDONLY)
    try:
        durable(d, "the hooks directory")
    finally:
        os.close(d)
except BaseException as error:
    sys.stderr.write("review gate: %s\n" % error)
    sys.exit(3)
SYNC
# Said on stderr and carried on — deliberately not `note`, which exits 0 and
# would skip the three cargo gates below. A warning that swallowed the gates
# would be a worse bug than the refusal it replaces.
if [ "$sync_status" = 3 ]; then
  # Kept for a transcript, and carried into the notice below. stderr alone is
  # not a warning: this hook surfaces stderr only when it exits 2, so on the
  # path that keeps the commit — which is this one — the only line a person
  # sees is the one saying the gate passed. The invariant in
  # .claude/skills/durability-invariants/SKILL.md is keep-and-retry AND show a
  # persistent warning; the second half was going to a channel nobody reads.
  echo "review gate: the trampoline is installed and in force, but the hooks directory could not be synced; a power loss could still lose it. The commit is judged either way." >&2
  unsynced=" レビューゲートは配置されましたが、hooks ディレクトリを sync できませんでした。この commit は判定されます。電源断で hook が失われることがあります。"
elif [ "$sync_status" != 0 ]; then
  block "python3 or $common_hooks/reference-transaction" \
"The trampoline could not be synced to disk and installed. If python3 on PATH is
a version-manager shim with no version selected, that is the cause: this hook
uses it for F_FULLFSYNC."
fi
# Every other reader in this change observes what it got — size, last line —
# rather than trusting a status. This one trusted `exit 3` as proof the rename
# happened. A python3 that exits 3 without renaming had the hook announce a gate
# that was not there. The landed file is read.
[ -x "$common_hooks/reference-transaction" ] \
  && [ "$(tail -n 1 "$common_hooks/reference-transaction" 2>/dev/null)" = '# operon: end of .githooks/installed/reference-transaction' ] \
  || block "$common_hooks/reference-transaction" \
"The trampoline is not in the repository's hooks directory after the install, or
is not executable, or is cut short."

# --- what is about to be committed -------------------------------------------

changed=$( { git diff --cached --name-only; git diff --name-only; } | sort -u )
# Not a bare `exit 0`: an unsynced trampoline noticed above would vanish with it.
if [ -z "$changed" ]; then
  [ -n "${unsynced:-}" ] && note "Commit gate: nothing staged.${unsynced}"
  exit 0
fi

# Rust and the manifest, and the gates themselves: the suite reads every hook
# and every check-*.sh, so an edit to one of those is a change the suite has an
# opinion about.
if printf '%s\n' "$changed" | grep -E '\.rs$|^Cargo\.(toml|lock)$|^\.claude/|^\.githooks/|^scripts/check-.*\.sh$' >/dev/null; then
  needs_cargo=1
else
  needs_cargo=0
fi

# --- the three gates ----------------------------------------------------------
#
# Same three, in the same order, as AGENTS.md, CONTRIBUTING.md, the ship skill,
# CI, and the pull-request template. every_document_that_names_the_gates_names_
# the_same_three keeps this copy honest.

if [ "$needs_cargo" -eq 1 ]; then
  if ! output=$(cargo fmt --check 2>&1); then
    block "cargo fmt --check" "$output"
  fi
  if ! output=$(cargo test --locked 2>&1); then
    block "cargo test --locked" "$(printf '%s' "$output" | tail -60)"
  fi
  if ! output=$(cargo clippy --locked -- -D warnings 2>&1); then
    block "cargo clippy --locked -- -D warnings" "$(printf '%s' "$output" | tail -60)"
  fi
fi

# --- test erosion -------------------------------------------------------------
#
# The playbook blocks an agent from editing test files while fixing a bug. In a
# Rust repository the failure takes two shapes: the failing test is deleted, or
# it is marked #[ignore]. Both are visible as counts, so both are checkable.
# CI runs every test here that is not #[ignore]d; #[ignore] moves a test out of
# that number silently.

if [ "$needs_cargo" -eq 1 ] && [ "${OPERON_ALLOW_TEST_REMOVAL:-0}" != "1" ] \
  && baseline=$(git show HEAD:src/tests.rs 2>/dev/null); then
  count() { printf '%s' "$2" | grep -c "$1" || true; }
  working=$(cat src/tests.rs)
  was_total=$(count '^#\[test\]' "$baseline")
  now_total=$(count '^#\[test\]' "$working")
  was_ignored=$(count '^#\[ignore' "$baseline")
  now_ignored=$(count '^#\[ignore' "$working")

  if [ "$now_total" -lt "$was_total" ]; then
    block "src/tests.rs lost tests ($was_total → $now_total)" \
"A failing test is evidence, not an obstacle. If a test is genuinely obsolete,
say which behaviour stopped existing and re-run with OPERON_ALLOW_TEST_REMOVAL=1."
  fi
  if [ "$now_ignored" -gt "$was_ignored" ]; then
    block "src/tests.rs gained #[ignore] ($was_ignored → $now_ignored)" \
"#[ignore] removes a test from the CI run without removing the line, so the
suite still looks complete. The six that are ignored need a live agent CLI; a
seventh needs the same justification, plus OPERON_ALLOW_TEST_REMOVAL=1."
  fi
fi

if [ "$needs_cargo" -eq 1 ]; then
  note "Commit gate passed: cargo fmt --check, cargo test --locked, cargo clippy --locked -- -D warnings, no test erosion. The review gate runs inside git next.${unsynced:-}"
fi
note "Commit gate: no Rust, manifest, or gate change staged, so the cargo gates were skipped. The review gate runs inside git next.${unsynced:-}"
