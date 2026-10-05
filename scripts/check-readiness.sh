#!/usr/bin/env bash
# Is this change's spec filled in well enough to implement from?
#
# Stage 2's gate, and the answer to a question the pipeline used to ask only of a
# reader's diligence. `docs/sdlc/templates/spec.md` says "never write n/a to a row
# you did not check" and "flagged concerns are answered before stage 3". Both were
# prose, and change 007's own spec reached `approved` carrying five stale paths.
#
# Structure, links, and completeness are all checked here, in one script rather
# than prose an agent checks against itself. Funnel and paywall analysis is left
# out rather than stubbed: a free local tool has no funnel.
#
# What this does NOT do, and says so in its own output: judge whether the design
# is any good. It judges whether the document is *filled in*. Presenting it as a
# quality verdict would be worse than not having it at all.
#
#   bash scripts/check-readiness.sh docs/sdlc/changes/NNN-slug
#   bash scripts/check-readiness.sh --all
#
#   exit 0  Go            — no blocking finding, no warning
#   exit 2  Conditional   — warnings only; proceed with care
#   exit 1  No-Go         — at least one blocking finding; stage 3 may not begin
set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

blocking=()
warning=()

block() { blocking+=("$1"); }
warn() { warning+=("$1"); }

# Prose with every backticked span and fenced block removed.
#
# The whole checker rests on this. Committed specs legitimately contain
# `Vec<Range<usize>>`, `<uuid>`, `<provider>/unparsable-line`, and
# `<stages> <attempts> <gate>` — all of them code or shapes being *discussed*.
# A checker that flagged those would be a checker people learn to skip, which is
# lesson 003's rule applied one document over: a name being discussed is not a
# name being pointed at.
prose() {
  sed -e '/^ *```/,/^ *```/d' -e 's/`[^`]*`//g' "$1"
}

check_change() {
  local dir="$1"
  local spec="$dir/spec.md"
  local intent="$dir/intent.md"
  local state="$dir/state.yaml"

  local route=""
  if [ -f "$state" ]; then
    route="$(sed -n 's/^ *route: *["'\'']*\([a-z0-9_-]*\)["'\'']*/\1/p' "$state" 2>/dev/null | head -1 || true)"
  fi

  case "$route" in
    bugfix|refactor|trivial|docs)
      warn "$dir: spec.md/intent.md がありません ($route route なら想定どおり)"
      return
      ;;
  esac

  # --- phase 1: is there an upstream at all -----------------------------------

  if [ ! -f "$intent" ]; then
    block "$dir: intent.md がありません"
    return
  fi
  if [ ! -f "$spec" ]; then
    # A bugfix or refactor route skips stages 1-2 by design, so a directory with
    # an intent and no spec is not automatically wrong — but nothing here can be
    # judged either. Say that rather than passing it silently.
    warn "$dir: spec.md がありません (bugfix/refactor route なら想定どおり)"
    return
  fi
  if ! grep -q 'intent.md' "$spec"; then
    block "$dir/spec.md: intent.md を参照していません"
  fi
  if ! awk '/^## Desired outcome/{f=1;next} /^## /{f=0} f&&NF{n++} END{exit !(n>0)}' "$intent"; then
    block "$dir/intent.md: Desired outcome が空です"
  fi

  local text
  text=$(prose "$spec")

  # --- phase 0: is it filled in ------------------------------------------------

  local left_over
  left_over=$(printf '%s\n' "$text" | grep -nE '<[a-z][a-z0-9 ,._/-]*>' | head -5)
  if [ -n "$left_over" ]; then
    block "$dir/spec.md: 雛形のプレースホルダが残っています:
$(printf '%s\n' "$left_over" | sed 's/^/      /')"
  fi

  local markers
  markers=$(printf '%s\n' "$text" | grep -nE '\b(TODO|TBD|FIXME|XXX)\b' | head -5)
  if [ -n "$markers" ]; then
    block "$dir/spec.md: 未決マーカーが残っています:
$(printf '%s\n' "$markers" | sed 's/^/      /')"
  fi

  # `- **Status**: draft | approved | superseded` unedited still holds the bars.
  if grep -qE '^\- \*\*Status\*\*:.*\|' "$spec"; then
    block "$dir/spec.md: Status が雛形の選択肢のままです"
  fi

  # --- phase 2: the sections a spec is made of ---------------------------------

  local section
  for section in "## Requirements" "## Behaviour" "## Design" \
                 "## Policy conformance" "## Flagged concerns" \
                 "## Acceptance" "## Rejected alternatives"; do
    grep -qF "$section" "$spec" || block "$dir/spec.md: \`$section\` の節がありません"
  done

  # --- phase 2b: a policy row nobody answered ----------------------------------
  #
  # The row this is for is the one whose "Applies?" cell is blank. The template
  # ships every row empty, so an untouched table is the default state and reads,
  # in a diff, like a table.

  local empty_cells
  empty_cells=$(awk '
    /^## Policy conformance/ { inside = 1; next }
    /^## / { inside = 0 }
    inside && /^\|/ && !/^\|[- :|]*\|$/ && !/^\| Policy \|/ {
      n = split($0, cell, "|")
      for (i = 2; i < n; i++) {
        gsub(/^[ \t]+|[ \t]+$/, "", cell[i])
        if (cell[i] == "") { print NR ": " substr($0, 1, 70); break }
      }
    }' "$spec" | head -5)
  if [ -n "$empty_cells" ]; then
    block "$dir/spec.md: Policy conformance に空欄の行があります:
$(printf '%s\n' "$empty_cells" | sed 's/^/      /')"
  fi

  # --- phase 1.5: requirements reach acceptance --------------------------------

  local requirements acceptance
  requirements=$(awk '/^## Requirements/{f=1;next} /^## /{f=0} f&&/^[0-9]+\./{n++} END{print n+0}' "$spec")
  acceptance=$(awk '/^## Acceptance/{f=1;next} /^## /{f=0} f&&NF{n++} END{print n+0}' "$spec")
  if [ "$requirements" -eq 0 ]; then
    block "$dir/spec.md: 番号付きの要件がありません"
  fi
  if [ "$acceptance" -eq 0 ]; then
    block "$dir/spec.md: Acceptance が空です — 何をもって完了とするか書かれていません"
  fi

  # --- phase 3: flagged concerns are answered, not merely listed ---------------
  #
  # The pipeline's own rule: a flagged concern is answered before stage 3, not
  # carried into it. A concern with no answer is the most reliable predictor of a
  # stage-3 rewrite, and it was the one thing nothing checked.
  #
  # What is checkable is *substance*, not a keyword. The first cut of this demanded
  # the words "Answer" or "Resolved" and rejected four already-committed specs
  # that answer their concerns in ordinary prose — a checker that is wrong about
  # the corpus it ships with is a checker nobody runs. So a concern fails when its
  # bullet carries no body: the template's own stub, `— why it is unresolved, what
  # would resolve it`, is 43 characters and does not clear the floor.

  local stubs
  stubs=$(awk -v floor=80 '
    function flush() {
      if (bullet != "" && body < floor) print bullet " (" body " chars)"
    }
    /^## Flagged concerns/ { inside = 1; next }
    /^## / { flush(); bullet = ""; inside = 0 }
    inside && /^- \*\*/ {
      flush()
      line = $0
      sub(/^- \*\*[^*]*\*\*/, "", line)     # the body starts after the bold title
      bullet = substr($0, 1, 60); body = length(line)
      next
    }
    inside && bullet != "" && /^[^-]/ { body += length($0) }
    END { flush() }' "$spec" | head -5)
  if [ -n "$stubs" ]; then
    block "$dir/spec.md: 中身のない flagged concern があります:
$(printf '%s\n' "$stubs" | sed 's/^/      /')"
  fi

  # --- warnings: true but not blocking -----------------------------------------

  if [ ! -f "$dir/state.yaml" ]; then
    warn "$dir: state.yaml がありません — 別セッションが状態を推測することになります"
  fi
  if [ -f "$dir/plan.md" ] && ! grep -q 'Proof of completion' "$dir/plan.md"; then
    warn "$dir/plan.md: Proof of completion がありません"
  fi
}

# --- what to check --------------------------------------------------------------

targets=()
case "${1:-}" in
  --all)
    while IFS= read -r dir; do targets+=("${dir%/}"); done \
      < <(find docs/sdlc/changes -mindepth 1 -maxdepth 1 -type d | sort)
    ;;
  "")
    echo "usage: check-readiness.sh <change-dir> | --all" >&2
    exit 2
    ;;
  *)
    targets=("${1%/}")
    [ -d "${targets[0]}" ] || { echo "no such change directory: ${targets[0]}" >&2; exit 2; }
    ;;
esac

# `:-` here for the same reason as below: under `set -u`, bash 3.2 aborts on an
# empty array with `targets[@]: unbound variable` while 5.3 expands it to
# nothing. `--all` against a repository with no changes yet is the case, and the
# two shells disagreed about it.
for target in "${targets[@]:-}"; do
  [ -n "$target" ] || continue
  check_change "$target"
done

# --- verdict ---------------------------------------------------------------------

for item in "${blocking[@]:-}"; do [ -n "$item" ] && printf 'blocking  %s\n' "$item"; done
for item in "${warning[@]:-}"; do [ -n "$item" ] && printf 'warning   %s\n' "$item"; done

# The findings are passed, not named. `local -n` would be shorter and was what
# this said, but a nameref is bash 4: `#!/usr/bin/env bash` finds 3.2 on a stock
# macOS, where `local: -n: invalid option` goes to stderr, the loop iterates over
# nothing, and the function echoes 0 — so this gate printed its blocking findings
# and then reported none of them. Anything added here runs under 3.2.
count() { local n=0; for i in "$@"; do [ -n "$i" ] && n=$((n + 1)); done; echo "$n"; }
blocking_count=$(count "${blocking[@]:-}")
warning_count=$(count "${warning[@]:-}")

echo
if [ "$blocking_count" -gt 0 ]; then
  echo "readiness: No-Go — $blocking_count blocking, $warning_count warning"
  echo "stage 3 may not begin. Answer these in the spec, not in the implementation."
  exit 1
fi
if [ "$warning_count" -gt 0 ]; then
  echo "readiness: Conditional Go — 0 blocking, $warning_count warning"
  exit 2
fi
echo "readiness: Go — ${#targets[@]} change(s), 0 blocking, 0 warning"
echo "This says the documents are filled in. It does not say the design is right."
exit 0
