#!/usr/bin/env bash
# Whether the installed Codex can load the config.toml Operon writes into.
#
#   bash scripts/check-codex-config-shape.sh                 # ~/.codex/config.toml
#   bash scripts/check-codex-config-shape.sh path/to/config.toml
#   OPERON_CODEX_PATIENCE=60 bash scripts/check-codex-config-shape.sh
#
#   exit 0  Codex loads it
#   exit 1  Codex does not, or a top-level `hooks` assignment is still present
#   exit 2  this machine could not be asked — no codex, no config, the server
#           did not answer in time, or the caller passed something unusable.
#           Deliberately not exit 1: a slow answer and a refusal look identical
#           on the wire, and reporting a healthy config as broken is as useless
#           as a check that can only pass.
#
# **A caller reads the line, not only the code.** Exit 2 is also what bash
# itself returns when it cannot parse this file, so a caller that switches on
# the number alone cannot tell a healthy "I could not ask" from a broken
# script — and the two want opposite responses. Every deliberate exit 2 here
# prints one line beginning `UNKNOWN:` on stdout first. No such line with exit
# 2 means the script did not run, and that is a bug to report rather than a
# machine to leave alone. Raised by review round 13 after an exit 2 with no
# stdout at all was seen once and never reproduced; the collision is a fact
# independent of whatever that was.
#
# not-wired: no hook or workflow calls this, and none should — the same reason
# scripts/check-codex-rollout-shape.sh is not wired. It reads the config on THIS
# machine and asks the installed `codex` to load it; CI has neither, so an
# automated caller could only ever pass, and lesson 007 is that a check which
# can only pass is not a check. Run it by hand after updating Codex, and after
# any change to src/tmux/hooks.rs.
#
# Why it exists, and why a test could not do this job. Operon used to insert a
# top-level `hooks = true` at the head of ~/.codex/config.toml. Codex declares
# that key as a TABLE (`HooksToml`), so the line was the wrong type for the
# schema and Codex refused to load its configuration at all — every session on
# the machine, whether or not Operon launched it. It failed two different ways:
#
#   config.toml:421:2: cannot extend value of type boolean with a dotted key
#       once Operon's own [hooks.state."…"] blocks followed the scalar, and
#   config.toml:1:9: invalid type: boolean `true`, expected struct HooksToml
#       when they did not.
#
# `codex_hook_install_writes_a_config_codex_can_load` in src/tests.rs parses
# what the writer produces, and that catches the first shape. It cannot catch
# the second: `hooks = true` on its own is perfectly valid TOML. The only thing
# that knows the type is Codex, so the authority here is Codex itself, started
# against a copy of the file. Operon does not own this format and cannot hold an
# opinion about it that stays true — which is the whole of lesson 007.
#
# Non-destructive: the config is copied into a temporary CODEX_HOME and the real
# ~/.codex is never written to.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

read_only=""
config=""
while [ $# -gt 0 ]; do
  case "$1" in
    # Everything this script decides from the file alone, and nothing that needs
    # a live Codex. It exists so `the_config_check_and_the_writer_read_one_file_
    # the_same_way` in src/tests.rs can hold this script's awk and
    # src/tmux/hooks.rs's line editor to the same acceptance set. Two
    # implementations of one rule drifted apart twice in two review rounds — the
    # reader refusing an inline table the writer had learned to keep, then the
    # reader demanding an exact header the writer had learned to match — and
    # both times a paragraph was the only thing meant to keep them together.
    --read-only) read_only=1; shift ;;
    -*)
      echo "UNKNOWN: unrecognised option $1"
      echo "usage: check-codex-config-shape.sh [--read-only] [CONFIG]" >&2
      exit 2
      ;;
    *) config="$1"; shift ;;
  esac
done
config="${config:-$HOME/.codex/config.toml}"

if [ -z "$read_only" ] && ! command -v codex > /dev/null 2>&1; then
  echo "UNKNOWN: codex is not on PATH"
  echo "codex is not on PATH — this check needs the CLI it is checking against" >&2
  exit 2
fi
if [ ! -f "$config" ]; then
  echo "UNKNOWN: no config at $config"
  echo "no config at $config — nothing for Operon to have written into" >&2
  exit 2
fi

if [ -z "$read_only" ]; then
  home=$(mktemp -d)
  workspace=$(mktemp -d)
  trap 'rm -rf "$home" "$workspace"' EXIT
  cp "$config" "$home/config.toml"
  [ -f "$HOME/.codex/auth.json" ] && cp "$HOME/.codex/auth.json" "$home/auth.json"
fi

# --- what the file says about itself -----------------------------------------
#
# Read the same way src/tmux/hooks.rs reads it — a key is what sits left of the
# first `=` on a line that is not a comment and not a table header, a value is
# what sits right of it, and a table header may carry space inside its brackets
# and a comment after them — so this reports on the shape that writer recognises
# rather than on a second opinion about TOML.
#
# "The same way" is a claim that has been false twice, in opposite directions,
# and both times review found it by running the two against one file rather than
# by comparing the source. Once the writer learned to keep an inline table and
# this reader still refused it; once the writer learned to match a commented
# header and this reader still demanded an exact one, and answered "not set" for
# a file that plainly set it. Anything changed in `opens_table` or
# `assignment_of` has to be changed here in the same commit — the two are one
# rule with two implementations, which is lesson 001, and the only thing holding
# them together is this paragraph and the fixtures in src/tests.rs.

# TWO header patterns, mirroring `looks_like_table_header` and
# `is_table_header` in src/tmux/hooks.rs, because the two scans below stop for
# opposite reasons and a boundary has a direction that costs more:
#
#   header_shape  brackets that open the line and close it with nothing after
#                 them but a comment. Generous. Ends the TOP-LEVEL scan, where
#                 reaching too far means calling a key that belongs to
#                 somebody's table a top-level scalar — and the writer beside
#                 it DELETES what this only reports.
#   header_named  the same, and the name has to look like a TOML key path.
#                 Strict. Closes the [features] scan, where reaching too far
#                 only under-reports the flag.
#
# A bare `^\[` is neither: a multi-line array's continuation row (`  [1, 2],`)
# starts with `[` and is not a header. Splitting a name on every dot is not
# `header_named` either — `profiles."gpt-5.6"` is ONE table, and the pattern
# gets that right by alternating quoted and bare segments rather than by
# splitting. Operon writes `[hooks.state."…/hooks.json:stop:1:0"]` itself, so
# dots inside quotes are its own output.
#
# BOTH patterns carry the `[[name]]` alternative, and the count of brackets has
# to match on the two sides: `table_header_shape` takes a second `[` only
# together with a `]]`, so `[[name]` and `[name]]` are headers to neither. That
# alternative was added to the Rust in round 7 and NOT here, and the drift was
# immediately what drift here always is — this script printing `REFUSE: a
# top-level hooks … Operon now removes it` about a `hooks` inside somebody's
# `[[profiles]]`, which the writer (correctly) leaves alone. Two implementations
# of one rule, third time. The cross-check now carries `[[…]]` rows.
#
# The backslashes are DOUBLED because `awk -v` runs escape processing over the
# value before the program sees it: a single `\[` arrives as a bare `[` and
# turns the pattern into a character class that matches nothing a header looks
# like. Measured, not reasoned — the single-backslash version matched `[features]`
# zero times and made the top-level scan read past the first table into the
# `[features]` body, reporting `hooks = true` there as a top-level scalar.
header_shape='^[[:space:]]*(\\[[^]]*\\]|\\[\\[[^]]*\\]\\])[[:space:]]*(#.*)?$'
quote="'"
segment="([A-Za-z0-9_-]+|\"[^\"]*\"|${quote}[^${quote}]*${quote})"
path="[[:space:]]*${segment}([[:space:]]*\\\\.[[:space:]]*${segment})*[[:space:]]*"
header_named="^[[:space:]]*(\\\\[${path}\\\\]|\\\\[\\\\[${path}\\\\]\\\\])[[:space:]]*(#.*)?\$"

# Neither pattern above can answer the question that comes before it: whether
# this line is being read at all, or is text inside a value an earlier line
# opened. `[features]` inside a multi-line string is not a header and
# `hooks = true` inside one is not an assignment, and three separate defects in
# the writer were that one mistake wearing different clothes. So both programs
# below open with `structural()`, which mirrors `structural_lines` in
# src/tmux/hooks.rs and has to change in the same commit it does.
#
# It is called from an unconditional first rule because it CARRIES STATE: skip a
# line and the depth is wrong for every line after it.
#
# The two kinds of open value NEST, and the first version of both
# implementations could not say so: a `"""` opened inside an array was invisible
# and the brackets of its contents were counted as structure. Review reproduced
# three losses from that one gap — a flag written into the middle of somebody's
# string, a line deleted out of one, and a person's own `hooks` key deleted
# because a `]` in their prose closed the array early. `scan` below is the same
# single walk `src/tmux/hooks.rs` does, with `fence` remembering which delimiter
# closes the string and `depth` the array around it.
#
# `closing_offset` is one walk for all four quoted things TOML has, which is the
# same move `src/tmux/hooks.rs` makes: the only difference between the one-line
# kinds and the multi-line ones is the length of the delimiter, and the two
# LITERAL kinds have no escapes at all. The multi-line search used to be a plain
# `index(body, f)` that did not know about them, and `\"""` is the ONLY way TOML
# lets somebody write a literal `"""` inside a `"""` string — so it is what a
# note explaining Operon's own config format contains, not a corner. Review
# reproduced three losses through it.
#
# Returns the 1-based index of the character AFTER the delimiter, or 0 if the
# text runs out first.
structural_awk='
function closing_offset(text, delim, escapes,   i, c, n, escaped) {
  n = length(delim)
  escaped = 0
  for (i = 1; i <= length(text); i++) {
    if (!escaped && substr(text, i, n) == delim) return i + n
    c = substr(text, i, 1)
    escaped = (!escaped && escapes && c == "\\")
  }
  return 0
}
# Walks the structural characters of `text` from `d` brackets deep and leaves
# what is still open in the globals `open`, `depth` and `fence`.
function scan(d, text,   i, c, rest, at, f, body) {
  depth = d
  i = 1
  while (i <= length(text)) {
    rest = substr(text, i)
    f = ""
    if (substr(rest, 1, 3) == "\"\"\"")           { f = "\"\"\"" }
    else if (substr(rest, 1, 3) == "\047\047\047") { f = "\047\047\047" }
    if (f != "") {
      at = closing_offset(substr(rest, 4), f, f == "\"\"\"")
      if (at == 0) {
        open = "fenced"; fence = f
        if (depth < 0) depth = 0
        return
      }
      i += at + 2
      continue
    }
    c = substr(rest, 1, 1)
    i++
    if (c == "\"" || c == "\047") {
      body = substr(text, i)
      at = closing_offset(body, c, c == "\"")
      i += (at == 0 ? length(body) : at - 1)
    }
    else if (c == "#") { break }
    else if (c == "[") { depth++ }
    else if (c == "]") { depth-- }
  }
  if (depth > 0) { open = "array" } else { open = ""; depth = 0 }
}
function structural(line,   position, value, at) {
  if (open == "fenced") {
    at = closing_offset(line, fence, fence == "\"\"\"")
    if (at == 0) return 0
    scan(depth, substr(line, at))
    return 0
  }
  if (open == "array") { scan(depth, line); return 0 }
  # A comment is not an assignment and a table header is not one either, which
  # is what `assignment_of` in src/tmux/hooks.rs decides by rejecting a trim
  # that starts with `[`.
  if (line ~ /^[[:space:]]*#/ || line ~ /^[[:space:]]*\[/) return 1
  position = index(line, "=")
  if (position == 0) return 1
  value = substr(line, position + 1)
  gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
  scan(0, value)
  return 1
}
'

toplevel_scalar=$(awk -v header="$header_shape" "$structural_awk"'
  # `structural` runs FIRST and unconditionally, because it carries state: skip
  # a line and the depth is wrong for every line after it. `past` records that
  # the top-level region has ended rather than `exit`ing, so the walk reaches
  # the end of the file and END can ask whether it ever closed.
  { here = structural($0); opened = (open != "") }
  !here || past     { next }
  $0 ~ header       { past = 1; next }
  /^[[:space:]]*#/  { next }
  {
    position = index($0, "=")
    if (position == 0) next
    key = substr($0, 1, position - 1)
    value = substr($0, position + 1)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
    # A value opening with `{` is an inline table, which is the shape Codex
    # asks for — somebody configured their hooks by hand. The writer learned
    # to leave it alone and this reader had not, so a working config was
    # reported broken in the same run that watched Codex load it. A checker
    # that disagrees with the writer it checks is worse than no checker.
    #
    # A value that continues onto the next line is left alone for the same
    # reason: the writer removes whole lines and will not touch one of these,
    # so reporting it would name a line nothing is going to fix.
    #
    # `opened` is that test, and it is the state machine answering rather than
    # a second pattern beside it. It used to be `starts_with("[") && no "]"`,
    # which called `hooks = [[],` a value that ends on its line — the writer
    # carried the same mistake and removed the line, leaving the rest of the
    # array unattached. One walk, one answer, both implementations.
    # \047 is awk octal for a single quote. Written that way because this whole
    # program is inside a shell single-quoted string, where a literal '"'"' would
    # end it — the first version did exactly that and the script stopped
    # parsing, caught by every_shell_script_parses_under_the_oldest_shell_its_
    # shebang_finds rather than by anybody reading it.
    if (key == "hooks" && substr(value, 1, 1) != "{" && !opened) {
      hits = hits " " key " = " value
    }
  }
  # The veto the writer has, mirrored. A document that never closes what it
  # opened was misread somewhere, and a misreading is not a licence to name a
  # line for removal — `apply_codex_config` declines to remove one from such a
  # file, so reporting it would be this script asking for an edit that will not
  # happen. It costs an unrepaired scalar in a file Codex cannot load either way.
  END { if (hits == "" || open != "") exit 1; print hits }
' "$config" || true)

# The opening pattern matches what `opens_table` in src/tmux/hooks.rs matches —
# whitespace inside the brackets, a trailing comment — and stops short of a
# dotted child like `[features.context_management]`. The exact-match version
# this replaces answered "features.hooks: not set" for a config that plainly
# sets it, which is the reverse of the error above and the same root: two
# readers of one file that do not agree.
#
# The CLOSING pattern is the strict one, matching `end` in `set_features_hooks`:
# a `  [1, 2]` row inside a multi-line array must not close the table, or this
# reports `not set` for a file that sets the flag just below the array.
#
# The two boundary rules consult `structural()` and the row that reports the key
# does NOT, which is the same split `set_features_hooks` makes and for the same
# reason: a header that is really text opens the table in the wrong place, but a
# `hooks = true` that is really text only makes both implementations decline to
# write. The report follows the writer's veto, because what this line exists to
# say is what the writer will do — not a second opinion about TOML.
#
# The matched line is TRIMMED before it is printed. A config with no indentation
# produced `features.hooks:hooks = true`, with nothing between the colon and the
# answer; the caller adds the space and this drops the leading one a nested file
# would have brought with it.
#
# A dotted key at the ROOT sets the same flag without any header at all, and
# review found this answering `features.hooks: not set` about a file whose first
# line was literally `features.hooks = true`. Telling a person nothing is amiss
# is the worst of the three things this line can do, so `rooted` tracks the
# region before the first table header and the flag is looked for in both
# shapes.
#
# Its limit, recorded rather than glossed: an inline `features = { hooks = … }`
# is still reported as not set. That direction only under-claims — it says
# Operon's hooks may not run, and nothing is deleted on the strength of it —
# whereas the writer vetoes on that line, so the two do not disagree about any
# EDIT. They disagree about a sentence.
features_hooks=$(awk -v header="$header_named" "$structural_awk"'
  BEGIN { rooted = 1 }
  { here = structural($0) }
  here && /^[[:space:]]*\[[[:space:]]*features[[:space:]]*\][[:space:]]*(#.*)?$/ {
    inside = 1; rooted = 0; next
  }
  here && $0 ~ header { inside = 0; rooted = 0 }
  (inside && /^[[:space:]]*hooks[[:space:]]*=/) ||
  (here && rooted && /^[[:space:]]*features[[:space:]]*\.[[:space:]]*"?hooks"?[[:space:]]*=/) {
    line = $0
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
    print line
    found = 1
  }
  END { if (!found) exit 1 }
' "$config" || true)

trust_blocks=$(grep -c '^\[hooks\.state\.' "$config" || true)

# --- what Codex says ---------------------------------------------------------

# The `sleep` holds stdin open. Without it the writer reaches EOF as soon as the
# two requests are written, the server shuts down, and thread/start goes
# unanswered — which reads exactly like a refusal and is not one. Found by this
# script reporting "gave no answer" against a config Codex loads fine.
#
# It is also a deadline, and review found that the first version had only moved
# the lie rather than removed it: a server slower than the sleep — a cold start
# after an update, an auth refresh, a larger config — loses its stdin, exits,
# and produces the identical empty answer. Reproduced by setting the sleep to 1
# against a config that passes at 20. So no answer is NOT folded into the
# refusal path below: it exits 2, the same as the other "this machine could not
# be asked" conditions, and says TIMEOUT. Reporting a healthy config as broken
# is the same failure as a check that can only pass, pointing the other way.
patience="${OPERON_CODEX_PATIENCE:-20}"
answer=""
if [ -z "$read_only" ]; then
  answer=$( { printf '%s\n%s\n' \
      '{"id":1,"method":"initialize","params":{"clientInfo":{"name":"operon-shape","version":"0"}}}' \
      "{\"id\":2,\"method\":\"thread/start\",\"params\":{\"cwd\":\"$workspace\",\"ephemeral\":false,\"serviceName\":\"operon-shape\"}}"
    sleep "$patience"; } \
    | CODEX_HOME="$home" codex app-server --stdio 2>/dev/null \
    | awk '/"id":2/ && !seen { print; seen = 1 }' || true)
fi

echo "codex:  $([ -n "$read_only" ] && echo "not asked (--read-only)" || codex --version 2>/dev/null || echo unknown)"
echo "config: $config ($(wc -c < "$config" | tr -d ' ') bytes, $trust_blocks hooks.state blocks)"
if [ -n "$features_hooks" ]; then
  echo "        features.hooks: $features_hooks"
else
  echo "        features.hooks: not set — Operon's hooks may be registered and never run"
fi

problem=0
if [ -n "$toplevel_scalar" ]; then
  echo "REFUSE: a top-level \`hooks\` is assigned something other than a table:$toplevel_scalar"
  echo "        Codex declares top-level \`hooks\` as a table, so a scalar there is"
  echo "        the wrong type. Operon wrote this before change 060 and now"
  echo "        removes it; a config still carrying one was last written by a"
  echo "        build from before that change. An inline table is NOT this — it"
  echo "        is somebody's working configuration, and neither Operon nor this"
  echo "        check touches it."
  problem=1
fi

if [ -n "$read_only" ]; then
  echo "loads:  not asked (--read-only)"
  exit "$problem"
elif [ -z "$answer" ]; then
  # A verdict already reached from the file alone stands. Reading the config
  # needs no Codex, so folding that finding into "this machine could not be
  # asked" would throw away a defect the check had definitely found — and the
  # usage block above promises exit 1 for it.
  if [ "$problem" -ne 0 ]; then
    echo "        (codex was also not reachable in time, but the finding above"
    echo "         was read from the file and does not depend on it.)"
    exit "$problem"
  fi
  echo "UNKNOWN: codex app-server did not answer within ${patience}s"
  echo "TIMEOUT: codex app-server did not answer thread/start within ${patience}s."
  echo "         This is NOT a verdict on the config — a cold start after a Codex"
  echo "         update, an auth refresh, or a larger file can all outlast the"
  echo "         wait, and the symptom is identical to a refusal. Raise it with"
  echo "         OPERON_CODEX_PATIENCE=60 and run again."
  exit 2
elif printf '%s' "$answer" | grep '"error"' >/dev/null; then
  echo "REFUSE: codex cannot load it:"
  printf '        %s\n' "$(printf '%s' "$answer" | sed 's/.*"message":"//; s/".*//')"
  problem=1
else
  echo "loads:  codex app-server reached thread/start against a copy of this config"
fi

exit "$problem"
