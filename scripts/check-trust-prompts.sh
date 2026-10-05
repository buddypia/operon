#!/usr/bin/env bash
# Whether the workspace-trust prompt each agent CLI draws on THIS machine still
# says what src/tmux.rs expects it to say.
#
# Operon answers that prompt for you. It used to answer it with a bare Enter,
# which on Claude Code's layout confirms `No, exit` and kills the session Operon
# had just opened. The fix reads the affirmative option off the screen, which
# means the option vocabulary is now load-bearing: a CLI that renames
# `Yes, I trust this folder` turns automatic approval off silently, and the only
# symptom is a prompt sitting there waiting for a person.
#
# What it checks, and what it does not. It answers one question: are the phrases
# and glyphs in src/tmux.rs still the ones the three CLIs print? That is an
# observation, and it is the half that goes stale without anyone noticing.
# Whether a given screen is *answerable* is decided by
# `workspace_trust_answer_keys`, and proved by the guards over real captured
# panes in src/tests.rs. Reimplementing that decision in awk would be a second
# copy of it, free to drift from the first and certain to be the copy nobody
# runs — so this deliberately stops at the vocabulary.
#
# not-wired: no hook or workflow calls this, and none should. It launches the
# real `claude`, `codex`, and `agy` on this machine; a CI runner has none of
# them, so an automated caller could only ever report zero and pass. Lesson 007:
# a check that can only pass is not a check. Run it by hand after updating an
# agent CLI.
#
# What it does to your machine, stated plainly: it starts each CLI once, in a
# fresh empty git repository under the system temp directory, reads the screen,
# and kills the session. It never sends a keystroke, so no prompt is ever
# answered and no directory is ever trusted. It cannot stop a CLI from writing
# to its own config store on startup the way it would on any other launch.
#
# Usage: check-trust-prompts.sh [--save DIR] [--print-vocabulary]
#
# `--save` writes each observed pane to `DIR/<cli>.pane`, escapes and all, the
# way `tmux capture-pane -e` hands it to Operon. That is the form the fixtures
# in src/tests.rs are in, so when this reports a prompt it cannot answer, the
# pane needed to fix it has already been kept. Retyping one from what the screen
# looks like loses the escapes, and the escapes are most of the problem.
#
# `--print-vocabulary` prints what this script read out of src/tmux.rs and stops,
# launching nothing. It exists so a test can run it: the vocabulary is parsed out
# of Rust source with awk, and the day rustfmt reflows a row is the day that
# parse quietly returns something shorter. `the_trust_prompt_script_reads_the_
# same_vocabulary_the_app_uses` in src/tests.rs compares this output against the
# consts themselves, on a runner that has no agent CLI to observe.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/..}" || exit 1

seconds_to_wait=25
save_to=""
print_vocabulary=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    (--save)
      save_to="${2:-}"
      if [ -z "$save_to" ]; then
        echo "check-trust-prompts: --save needs a directory" >&2
        exit 1
      fi
      mkdir -p "$save_to" || exit 1
      shift 2
      ;;
    (--print-vocabulary)
      print_vocabulary=1
      shift
      ;;
    (*)
      echo "check-trust-prompts: unknown argument $1" >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# The vocabulary, read out of src/tmux.rs.
#
# Read rather than restated. A copy here would be a second place to update, and
# the failure it produces is the worst kind: this script would keep reporting
# that the phrases it knows are still on screen, having stopped checking the
# phrases Operon actually uses. Lesson 004.
# ---------------------------------------------------------------------------

# One `phrase<TAB>date-observed` per row of a `[TrustPhrase; N]` const. The
# strings are collected across the whole block and paired afterwards, because
# rustfmt breaks a long row onto four lines and a line-at-a-time reader would
# see half a row. `N` is then checked against the number of pairs found: if this
# ever parses the wrong thing, it says so instead of returning a short list.
phrases() {
  awk -v name="$1" '
    $0 ~ ("const " name ": \\[TrustPhrase;") {
      match($0, /TrustPhrase; [0-9]+/)
      declared = substr($0, RSTART + 13, RLENGTH - 13)
      inside = 1
      next
    }
    inside && /^\];/ { inside = 0 }
    inside && /^[ \t]*\/\// { next }
    inside {
      rest = $0
      while (match(rest, /"[^"]*"/)) {
        strings[++count] = substr(rest, RSTART + 1, RLENGTH - 2)
        rest = substr(rest, RSTART + RLENGTH)
      }
    }
    END {
      if (declared == "") {
        print "no const named " name " in src/tmux.rs" > "/dev/stderr"
        exit 3
      }
      if (count != declared * 2) {
        printf "%s declares %d rows, %d strings parsed\n", name, declared, count > "/dev/stderr"
        exit 4
      }
      for (i = 1; i <= count; i += 2) print strings[i] "\t" strings[i + 1]
    }
  ' src/tmux.rs
}

markers=$(phrases TRUST_PROMPT_MARKERS) || exit 1
yes_options=$(phrases TRUST_PROMPT_YES) || exit 1
no_options=$(phrases TRUST_PROMPT_NO) || exit 1

# `['a', 'b', 'c']` to one glyph per line. Per line and not one run of
# characters: two of the three are multi-byte, awk indexes bytes, and a run
# would be compared a byte at a time — `❯` and `›` share their first byte, so
# every box-drawing character on screen would have counted as a cursor.
#
# The right-hand side is taken from the `= [` and not from the first `[`, which
# is inside the `[char; 3]` type. \047 is the single quote, spelled in octal
# because the awk program itself is inside single quotes.
cursors=$(
  awk '
    /const TRUST_PROMPT_CURSORS/ {
      row = $0
      sub(/^[^=]*=[ \t]*\[/, "", row)
      sub(/\].*$/, "", row)
      count = split(row, parts, ",")
      for (i = 1; i <= count; i++) {
        glyph = parts[i]
        gsub(/[ \t]/, "", glyph)
        gsub(/\047/, "", glyph)
        if (glyph != "") { print glyph; found = 1 }
      }
    }
    END { if (!found) exit 3 }
  ' src/tmux.rs
) || exit 1

if [ -z "$cursors" ]; then
  echo "check-trust-prompts: TRUST_PROMPT_CURSORS parsed as empty" >&2
  exit 1
fi

if [ -n "$print_vocabulary" ]; then
  printf '%s\n' "$markers" | sed 's/^/TRUST_PROMPT_MARKERS\t/'
  printf '%s\n' "$yes_options" | sed 's/^/TRUST_PROMPT_YES\t/'
  printf '%s\n' "$no_options" | sed 's/^/TRUST_PROMPT_NO\t/'
  printf '%s\n' "$cursors" | sed 's/^/TRUST_PROMPT_CURSORS\t/'
  exit 0
fi

# ---------------------------------------------------------------------------
# Matching a captured screen against it.
# ---------------------------------------------------------------------------

# The first column of `phrases` output, for a `for` loop that must not split on
# the spaces inside a phrase. Newline-separated, read one line at a time.
first_column() {
  awk -F'\t' '{ print $1 }'
}

# Which phrase from the given list appears in the given lowercased text, or
# nothing. `case` rather than `grep`: a `grep -q` closing the pipe early raises
# SIGPIPE in the writer, and under `pipefail` that becomes this function's exit
# status. Lesson 018.
first_match() {
  matched=""
  while IFS= read -r phrase; do
    [ -n "$phrase" ] || continue
    case "$2" in
      (*"$phrase"*)
        matched="$phrase"
        break
        ;;
    esac
  done <<EOF
$(printf '%s\n' "$1" | first_column)
EOF
  printf '%s' "$matched"
}

# The cursor glyph marking the selected option, or nothing.
#
# Only lines that already carry an option phrase are looked at, and exactly one
# of them must be marked. Both halves matter, and the first was wrong once: an
# earlier cut asked whether *any* line on screen began with a glyph, and Codex's
# real trust prompt opens with `> You are in /private/var/…`. It would have
# reported the prompt answerable on the day Codex stopped marking its selected
# option — which is the one regression this check exists to see.
#
# $1 screen, $2 newline-separated option phrases, $3 newline-separated glyphs.
#
# Through the environment and not `awk -v`: a `-v` assignment is scanned for
# backslash escapes and rejects an embedded newline outright — `awk: newline in
# string` — which this did on its first run, for all three CLIs at once. It
# failed in the safe direction, reporting every prompt unanswerable rather than
# every prompt fine, but it reported nothing true.
cursor_on_option_line() {
  printf '%s\n' "$1" | TRUST_OPTION_PHRASES="$2" TRUST_CURSOR_GLYPHS="$3" awk '
    BEGIN {
      phrase_count = split(ENVIRON["TRUST_OPTION_PHRASES"], phrase, "\n")
      glyph_count = split(ENVIRON["TRUST_CURSOR_GLYPHS"], glyph, "\n")
    }
    {
      lowered = tolower($0)
      is_option = 0
      for (i = 1; i <= phrase_count; i++) {
        if (phrase[i] != "" && index(lowered, phrase[i])) { is_option = 1; break }
      }
      if (!is_option) next
      bare = $0
      sub(/^[ \t]+/, "", bare)
      for (i = 1; i <= glyph_count; i++) {
        if (glyph[i] != "" && index(bare, glyph[i]) == 1) { marked++; seen = glyph[i] }
      }
    }
    END { if (marked == 1) { print seen; exit 0 } exit 1 }
  '
}

# ---------------------------------------------------------------------------
# The observation.
# ---------------------------------------------------------------------------

observed=0
unanswerable=0
silent=0
installed=0
session=""
workspace=""

# A private tmux server, and a name outside `MANAGED_TMUX_PREFIX`.
#
# `tmux_command()` in src/tmux.rs passes no `-L`, so the app drives the default
# server. A probe session there is returned by `parse_operon_tmux_sessions` as an
# orphaned Operon session the moment its name starts with `operon-`, and the
# recovery screen offers to adopt it — persisting a session record whose worktree
# is a `mktemp -d` directory this script deletes seconds later. The socket keeps
# the probe off that server entirely; the name is the second lock on the same
# door.
# The socket carries `$$` because `cleanup` ends with `kill-server`: on a shared
# socket the first run to finish would kill the second run's live probes
# mid-observation, and the second would report every CLI silent.
#
# `-u` and the locale defaults mirror `tmux_command()` in src/tmux.rs, which is
# how the app starts every session. Without `-u` a pane in a shell with no
# locale is not in UTF-8 mode, `capture-pane` hands back `❯` and `›` mangled,
# and this reports Claude and `agy` unanswerable on a tree where the app — which
# passes `-u` — is fine.
probe_tmux() {
  LANG="${LANG:-ja_JP.UTF-8}" LC_ALL="${LC_ALL:-ja_JP.UTF-8}" \
    tmux -u -L "operon-trustprobe-$$" "$@"
}

# `probe` returns early on several paths and sleeps for up to 25 seconds on one
# of them. A Ctrl-C or a closed terminal in that window would otherwise leave a
# detached, live `claude`, `codex`, or `agy` running against a temp directory
# that is about to be removed.
#
# Idempotent, so `EXIT` can have it too: both guards are `-n` tests and
# `kill-server` on a server that is already gone is silent.
cleanup() {
  [ -n "${session:-}" ] && probe_tmux kill-session -t "$session" >/dev/null 2>&1
  [ -n "${workspace:-}" ] && rm -rf "$workspace"
  probe_tmux kill-server >/dev/null 2>&1
  return 0
}
trap 'cleanup; exit 130' INT TERM HUP
trap cleanup EXIT

probe() {
  label="$1"
  binary="$2"

  if ! command -v "$binary" >/dev/null 2>&1; then
    printf '  %-6s not installed — `%s` is not on PATH\n' "$label" "$binary"
    return
  fi
  installed=$((installed + 1))

  # Reported, not returned into silence. This was the last door left open on
  # "a check that can only pass": with an unwritable `TMPDIR` all three probes
  # fell out here, the report body came out empty, and the summary still said
  # every prompt was fine. The end-of-run invariant below now catches the next
  # such door without anybody having to name it first.
  workspace=$(mktemp -d "${TMPDIR:-/tmp}/operon-trust-probe.XXXXXX") || {
    printf '  %-6s could not make a probe directory under %s\n' "$label" "${TMPDIR:-/tmp}"
    silent=$((silent + 1))
    workspace=""
    return
  }
  # Version controlled and empty. Codex only asks about a directory git knows
  # about, so a plain temp directory reaches its startup screen and reports
  # nothing — which looks exactly like a CLI that dropped the prompt. Checked,
  # for that reason: an unreported `git init` failure would arrive as a CLI that
  # stopped asking.
  if ! git init -q "$workspace" >/dev/null 2>&1; then
    printf '  %-6s could not prepare a git repository to be asked about\n' "$label"
    silent=$((silent + 1))
    rm -rf "$workspace"
    workspace=""
    return
  fi

  session="trustprobe-$label"
  probe_tmux kill-session -t "$session" >/dev/null 2>&1
  # No `-x`/`-y`. An earlier cut asked for 200x50 "so a long option does not
  # wrap into two lines" — which is exactly the condition
  # `start_tmux_agent_session` cannot promise, since it passes no size either and
  # takes tmux's `default-size`. A probe that widens the pane until nothing wraps
  # is measuring a pane the app never creates, and the wrap is the failure: at 60
  # columns Claude Code's question breaks in half and the prompt goes wholly
  # unrecognised. Inheriting the same default is what makes this an observation
  # of what Operon will see.
  if ! probe_tmux new-session -d -s "$session" -c "$workspace" "$binary" >/dev/null 2>&1; then
    printf '  %-6s could not be started under tmux\n' "$label"
    silent=$((silent + 1))
    rm -rf "$workspace"
    session=""
    workspace=""
    return
  fi

  screen=""
  lower=""
  unwrapped=""
  marker=""
  elapsed=0
  while [ "$elapsed" -lt "$seconds_to_wait" ]; do
    sleep 1
    elapsed=$((elapsed + 1))
    screen=$(probe_tmux capture-pane -p -t "$session" 2>/dev/null)
    lower=$(printf '%s' "$screen" | tr '[:upper:]' '[:lower:]')
    # Two views of the same screen, matching the two the app takes.
    # `holds_a_trust_question` collapses whitespace across line ends before
    # searching, because a marker is a sentence and a sentence wraps; the option
    # search keeps the line breaks, because an option that wrapped is one the app
    # would not find either and a green here would be a lie about that.
    unwrapped=$(printf '%s' "$lower" | tr '\n\t' '  ' | tr -s ' ')
    # `markers` only. The affirmative options are deliberately not searched
    # here: `yes, continue` is what Codex offers when it wants to run a command,
    # and treating it as recognition is the exact defect lesson 021 records —
    # which would be worth little if the script written to police it repeated it.
    marker=$(first_match "$markers" "$unwrapped")
    [ -n "$marker" ] && break
  done

  if [ -n "$save_to" ] && [ -n "$marker" ]; then
    probe_tmux capture-pane -e -p -t "$session" >"$save_to/$label.pane" 2>/dev/null
    printf '  %-6s pane saved to %s\n' "$label" "$save_to/$label.pane"
  fi

  probe_tmux kill-session -t "$session" >/dev/null 2>&1
  rm -rf "$workspace"
  session=""
  workspace=""

  if [ -z "$marker" ]; then
    instead=$(printf '%s\n' "$screen" | sed 's/^[[:space:]]*//' | awk 'length && !seen { print; seen = 1 }' 2>/dev/null)
    silent=$((silent + 1))
    printf '  %-6s NO TRUST PROMPT within %ss — showed: %s\n' \
      "$label" "$seconds_to_wait" "${instead:-（空の画面）}"
    return
  fi

  observed=$((observed + 1))

  answer=$(first_match "$yes_options" "$lower")
  refusal=$(first_match "$no_options" "$lower")

  if [ -z "$answer" ]; then
    unanswerable=$((unanswerable + 1))
    printf '  %-6s UNANSWERABLE — recognised by "%s", but no phrase in TRUST_PROMPT_YES is on screen\n' \
      "$label" "$marker"
    return
  fi

  option_phrases=$(printf '%s\n%s\n' "$yes_options" "$no_options" | first_column)
  cursor=$(cursor_on_option_line "$screen" "$option_phrases" "$cursors")
  if [ -z "$cursor" ]; then
    unanswerable=$((unanswerable + 1))
    printf '  %-6s UNANSWERABLE — yes is "%s", but no single option line is marked with any of %s\n' \
      "$label" "$answer" "$(printf '%s' "$cursors" | tr '\n' ' ')"
    return
  fi

  printf '  %-6s wording still matches — yes "%s", no "%s", cursor "%s"\n' \
    "$label" "$answer" "${refusal:-（画面になし）}" "$cursor"
}

if ! command -v tmux >/dev/null 2>&1; then
  echo "check-trust-prompts: tmux is not installed; nothing can be observed" >&2
  exit 1
fi

echo "trust prompts, observed on this machine:"
probe claude claude
probe codex codex
probe agy agy
cleanup

# Rows nobody has watched a CLI print. Not a failure — some of them are
# defensive, carried from before this script existed — but a row that stays
# UNOBSERVED across several CLI updates is a row to question rather than trust.
unobserved=$(
  printf '%s\n%s\n%s\n' "$markers" "$yes_options" "$no_options" |
    awk -F'\t' '$2 == "UNOBSERVED" { print "  " $1 }'
)

echo
if [ -n "$unobserved" ]; then
  echo "vocabulary rows still marked UNOBSERVED:"
  printf '%s\n' "$unobserved"
  echo
fi

# A CLI that is installed and never showed its question is a failure, not a
# footnote. It is what a reworded prompt looks like from here, and it is
# indistinguishable in a summary line from a CLI that was never installed —
# which is why the two are counted separately. Without this the script reports
# `0 of 3 observed, all answerable`, exit 0, on a machine with no agent CLI at
# all: the "a check that can only pass is not a check" shape its own header
# cites from lesson 007.
# Every installed CLI has to have been counted exactly once, as observed or as
# silent. Three early returns in `probe` were each fixed after being named one at
# a time — `git init`, `new-session`, `mktemp` — and each time the symptom was
# the same: an empty report body under a summary line saying everything was
# fine. This is the version of that check that does not need the next door named
# before it closes.
if [ $((observed + silent)) -ne "$installed" ]; then
  echo "check-trust-prompts: $installed CLIs installed but $observed observed + $silent silent — a probe returned without reporting" >&2
  exit 1
fi

if [ "$unanswerable" -gt 0 ] || [ "$silent" -gt 0 ]; then
  echo "trust prompts: $observed of $installed installed CLIs observed, $unanswerable unanswerable, $silent silent"
  echo
  if [ "$unanswerable" -gt 0 ]; then
    echo "An unanswerable prompt means Operon leaves it on screen for a person —"
    echo "nothing breaks, automatic approval just stops. Add the wording the CLI"
    echo "now prints to TRUST_PROMPT_YES or TRUST_PROMPT_NO in src/tmux.rs, with"
    echo "today's date, and re-save the pane into src/tests.rs as a fixture with"
    echo "the --save option above."
  fi
  if [ "$silent" -gt 0 ]; then
    echo "A silent CLI is installed and did not ask. Either it reworded the"
    echo "question — add the new wording to TRUST_PROMPT_MARKERS — or it had"
    echo "nothing to ask about, which for Codex means the probe directory was"
    echo "not a git repository."
  fi
  exit 1
fi

if [ "$installed" -eq 0 ]; then
  echo "trust prompts: no agent CLI is installed; nothing was observed" >&2
  exit 1
fi

echo "trust prompts: $observed of $installed installed CLIs observed, all still worded as src/tmux.rs expects"
