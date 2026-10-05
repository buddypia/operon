#!/usr/bin/env python3
"""Watch this change's guards fail, once per thing they claim to catch.

    python3 docs/sdlc/changes/060-codex-cannot-load-the-configuration-operon-writes/mutations.py

Each mutation edits `src/tmux/hooks.rs`, runs the two tests over the Codex
config writer, restores the file, and reports. A substitution that matches
nothing is reported as this script's own failure rather than as a green guard —
a mutation that did not apply proves nothing, and that is how a mutation run
lies.

The first mutation is the defect itself, put back. It is the one that matters:
before this change the suite was green with that line in place, because the test
beside it asserted `config.starts_with("hooks = true\\n")` — the author's
intention restated — and never read the result back.
"""
import pathlib
import subprocess
import sys

TREE = pathlib.Path(__file__).resolve().parents[4]
SOURCE = TREE / "src" / "tmux" / "hooks.rs"
# The checker is mutated too, and that is the point of it being here. Three of
# this change's defects were the writer and the checker disagreeing about one
# rule, so a mutation set that only reaches the Rust half would leave the half
# that was wrong twice untested.
SCRIPT = TREE / "scripts" / "check-codex-config-shape.sh"
TESTS = [
    "codex_hook_install_writes_a_config_codex_can_load",
    "codex_hook_install_writes_the_feature_flag_and_trust_entries_once",
    "the_config_check_and_the_writer_read_one_file_the_same_way",
    # Added in round 7, and the run that demanded it is the argument for having
    # this file at all: teaching the line editor to carry the state of a
    # multi-line value turned SIX red mutations green in one go, because every
    # fixture that told the strict reader from the generous one did it with a
    # bracket-shaped row inside an array — and those rows are no longer read as
    # headers by either. The distinction survives only over input TOML would
    # refuse, which a fixture that parses its output cannot carry.
    "a_config_toml_would_refuse_is_still_not_a_config_to_delete_from",
]

MUTATIONS = [
    (
        "the defect itself: the top-level scalar is written again",
        "    if enable_hooks {\n        set_features_hooks(&mut lines);\n    }",
        '    if enable_hooks {\n        set_features_hooks(&mut lines);\n        lines.insert(0, "hooks = true".to_owned());\n    }',
    ),
    (
        "a machine an older Operon broke is left broken",
        "    for index in removing.into_iter().rev() {\n        lines.remove(index);\n    }",
        "    for index in removing.into_iter().rev() {\n        let _ = index;\n    }",
    ),
    (
        "the repair is skipped on the path that turns hooks off",
        "    let removing: Vec<usize> = (0..first_table)",
        "    let removing: Vec<usize> = (0..if enable_hooks { first_table } else { 0 })",
    ),
    (
        "the feature flag is written a second time beside another tool's",
        '        .any(|line| might_bind(line, "hooks"))',
        "        .any(|_line| false)",
    ),
    # This slot first held "a comment inside [features] is read as the key",
    # mutating away an explicit `starts_with('#')` branch in `key_of`. It came
    # back green, and the reason is that the branch could not matter: a
    # commented line yields the key `# hooks`, which never equals `hooks`. The
    # branch was removed rather than the mutation weakened — a guard whose
    # mutation cannot fail is not a guard — and what stands here instead is the
    # one piece of `key_of` that is load-bearing.
    (
        "the key is compared without trimming the space before `=`",
        "    let (key, value) = trimmed.split_once('=')?;\n    Some((key.trim(), value.trim()))",
        "    let (key, value) = trimmed.split_once('=')?;\n    Some((key, value.trim()))",
    ),
    # Round 2 of review. All three were found by running apply_codex_config, not
    # by reading it, and the first survived the whole suite before its fixture
    # existed — the branch it breaks is the one both earlier fixtures stepped
    # past, one by already having the key and the other by having no table.
    (
        "the feature flag is written above its own table header",
        '    lines.insert(header + 1, "hooks = true".to_owned());',
        '    lines.insert(header, "hooks = true".to_owned());',
    ),
    (
        "a [features] header with a trailing comment is not recognised",
        "    rest.is_empty() || rest.starts_with('#')",
        "    rest.is_empty()",
    ),
    (
        "a hooks table somebody wrote by hand is deleted too",
        "                    && !value.starts_with('{')\n                    && !opens_multiline_value(value)",
        "                    && !opens_multiline_value(value)",
    ),
    # Round 3. The first is the writer again; the other two are the checker,
    # which had been wrong twice and had nothing mutating it.
    (
        "a header with space inside its brackets is not recognised",
        "    let Some(rest) = rest.trim_start().strip_prefix(name) else {",
        "    let Some(rest) = rest.strip_prefix(name) else {",
    ),
    (
        SCRIPT,
        "the checker demands an exact header the writer does not",
        "  here && /^[[:space:]]*\\[[[:space:]]*features[[:space:]]*\\][[:space:]]*(#.*)?$/ {",
        "  here && /^[[:space:]]*\\[features\\][[:space:]]*$/ {",
    ),
    (
        SCRIPT,
        "the checker refuses an inline table the writer keeps",
        '    if (key == "hooks" && substr(value, 1, 1) != "{" && !opened) {',
        '    if (key == "hooks" && !opened) {',
    ),
    # Round 4. Review stopped reporting spellings and reported the DIRECTION the
    # code falls when it does not recognise one, so these mutate the falling
    # rather than the recognising: each one puts back "write anyway", which is
    # the difference between a flag that is not set and a Codex that will not
    # start.
    (
        "an unrecognised [features] spelling is written beside rather than left alone",
        '        if lines\n            .iter()\n            .any(|line| might_open_table(line, "features") || might_bind(line, "features"))\n        {\n            return;\n        }',
        '        if false {\n            return;\n        }',
    ),
    (
        "the generous reader answers yes to a dotted child, so the flag is never set",
        '    table_header_shape(line).is_some_and(|(found, _)| unquote_segment(found) == name)',
        '    table_header_shape(line).is_some_and(|(found, _)| found.contains(name))',
    ),
    (
        "a continuation row is taken for a table header again",
        "fn is_table_header(line: &str) -> bool {\n    table_header_name(line).is_some()\n}",
        "fn is_table_header(line: &str) -> bool {\n    line.trim_start().starts_with('[')\n}",
    ),
    # Distinct from the `is_table_header` mutation below it: this one puts the
    # old end-of-table test back at the ONE call site that reads the [features]
    # body, leaving `might_open_table` correct, so it isolates the range from
    # the predicate.
    (
        "the [features] body scan ends at a continuation row again",
        "        .find(|index| is_table_header(&lines[*index]))",
        "        .find(|index| lines[*index].trim_start().starts_with('['))",
    ),
    (
        "a key that merely starts with `hooks` is read as the flag",
        "        .is_some_and(|segments| unquote_segment(segments[0]) == name)",
        "        .is_some_and(|segments| unquote_segment(segments[0]).starts_with(name))",
    ),
    (
        "the first line of a multi-line top-level value is removed, orphaning the rest",
        "                    && !value.starts_with('{')\n                    && !opens_multiline_value(value)",
        "                    && !value.starts_with('{')",
    ),
    # Round 5. Both are one implementation of a shared rule left behind when its
    # neighbour was converted — the same drift as rounds 2 and 3, and the reason
    # `is_table_header` now has exactly one definition that every scan calls.
    (
        "the top-level scan ends at a continuation row, so the scalar is never repaired",
        "            outside[*index]\n                && (looks_like_table_header(&lines[*index])\n                    || opens_unreadable_table(&lines[*index]))",
        "            outside[*index] && lines[*index].trim_start().starts_with('[')",
    ),
    # Round 5. The two boundaries take opposite readers because their mistakes
    # cost opposite amounts, so each gets the OTHER one — swapping them is the
    # defect, and a single mutation over a shared reader would not express it.
    (
        "the top-level scan takes the strict reader and deletes somebody's key",
        "            outside[*index]\n                && (looks_like_table_header(&lines[*index])\n                    || opens_unreadable_table(&lines[*index]))",
        "            outside[*index] && is_table_header(&lines[*index])",
    ),
    (
        "the [features] body scan takes the generous reader and stops inside an array",
        "        .find(|index| is_table_header(&lines[*index]))",
        "        .find(|index| looks_like_table_header(&lines[*index]))",
    ),
    (
        "a key path is split on dots inside quotes",
        "            None if character == '\"' || character == '\\'' => quote = Some(character),",
        "            None if character == '\\u{0}' => quote = Some(character),",
    ),
    (
        "an array row is accepted as a table named `1, 2`",
        "    table_header_shape(line)\n        .map(|(name, _)| name)\n        .filter(|name| is_key_path(name))",
        "    table_header_shape(line).map(|(name, _)| name)",
    ),
    (
        SCRIPT,
        "the checker's header pattern loses its escaping and matches no header",
        "header_shape='^[[:space:]]*(\\\\[[^]]*\\\\]|\\\\[\\\\[[^]]*\\\\]\\\\])[[:space:]]*(#.*)?$'",
        "header_shape='^[[:space:]]*(\\[[^]]*\\]|\\[\\[[^]]*\\]\\])[[:space:]]*(#.*)?$'",
    ),
    (
        SCRIPT,
        "the checker closes the [features] table with the generous pattern",
        'features_hooks=$(awk -v header="$header_named"',
        'features_hooks=$(awk -v header="$header_shape"',
    ),
    (
        SCRIPT,
        "the checker ends the top-level scan with the strict pattern",
        'toplevel_scalar=$(awk -v header="$header_shape"',
        'toplevel_scalar=$(awk -v header="$header_named"',
    ),
    # Round 7, the rust review. Three Importants and they are one defect: a line
    # was read for what it looks like alone, when whether it means anything at
    # all depends on the lines before it. Each mutation below takes the state
    # machine away from exactly one scan, because the three reached the user
    # three different ways and a single mutation over `structural_lines` would
    # collapse them into one.
    (
        "the [features] body scan forgets which lines are inside a value",
        '    let end = (header + 1..lines.len())\n        .filter(structural)\n        .find(|index| is_table_header(&lines[*index]))',
        '    let end = (header + 1..lines.len())\n        .find(|index| is_table_header(&lines[*index]))',
    ),
    (
        "the [features] header is found inside a multi-line string",
        '    let Some(header) = (0..lines.len())\n        .filter(structural)\n        .find(|index| opens_table(&lines[*index], "features"))',
        '    let Some(header) = (0..lines.len())\n        .find(|index| opens_table(&lines[*index], "features"))',
    ),
    (
        "the top-level repair removes a line that is inside somebody's note",
        "    let removing: Vec<usize> = (0..first_table)\n        .filter(|index| outside[*index])",
        "    let removing: Vec<usize> = (0..first_table)",
    ),
    (
        "the top-level region ends at a header that is really text",
        "            outside[*index]\n                && (looks_like_table_header(&lines[*index])\n                    || opens_unreadable_table(&lines[*index]))",
        "            looks_like_table_header(&lines[*index])\n                    || opens_unreadable_table(&lines[*index])",
    ),
    (
        "an array-of-tables header is not a header, so a second one is appended",
        '    let (inner, closing, array) = match inner.strip_prefix(\'[\') {\n        Some(doubled) => (doubled, "]]", true),\n        None => (inner, "]", false),\n    };',
        '    let (inner, closing, array) = (inner, "]", false);',
    ),
    (
        "a bracket inside a string is counted as structure, hiding the rest of the file",
        "            quote @ ('\"' | '\\'') => {",
        "            quote @ '\\u{0}' => {",
    ),
    # And the decision the other way, which is not a reading of TOML but a
    # choice about which mechanism gets the last word: the VETOES do not consult
    # the state machine, so that a wrong answer from it cannot produce a
    # duplicate. Teaching them to skip text is what these put back.
    (
        "the veto learns to skip text, and appends a table beside one it did not see",
        '        if lines\n            .iter()\n            .any(|line| might_open_table(line, "features") || might_bind(line, "features"))\n        {',
        '        if (0..lines.len()).filter(structural).any(|index| {\n            might_open_table(&lines[index], "features")\n                || might_bind(&lines[index], "features")\n        }) {',
    ),
    (
        "the veto learns to skip text, and writes the flag a second time",
        '    if lines[header + 1..end]\n        .iter()\n        .any(|line| might_bind(line, "hooks"))',
        '    if (header + 1..end)\n        .filter(structural)\n        .any(|index| might_bind(&lines[index], "hooks"))',
    ),
    (
        SCRIPT,
        "the checker reads a header inside a multi-line string as a header",
        '  here && /^[[:space:]]*\\[[[:space:]]*features[[:space:]]*\\][[:space:]]*(#.*)?$/ {',
        '  /^[[:space:]]*\\[[[:space:]]*features[[:space:]]*\\][[:space:]]*(#.*)?$/ {',
    ),
    (
        SCRIPT,
        "the checker reports a top-level scalar that is inside somebody's note",
        '  !here || past     { next }',
        '  past              { next }',
    ),
    (
        SCRIPT,
        "the checker's bracket counting stops respecting quotes",
        '    if (c == "\\"" || c == "\\047") {',
        '    if (c == "") {',
    ),
    # Round 7 of review, the rust pass, and a second round of the same finding
    # in a new place: the two open values NEST and neither implementation could
    # say so. Three losses came through that one gap and all three parse, so
    # each mutation here is checked by a fixture asserting the person's text by
    # VALUE rather than by the document loading.
    (
        "the state machine forgets the array around a multi-line string",
        "            (depth as isize, &line[past..])",
        "            (0, &line[past..])",
    ),
    (
        "a multi-line string opened inside an array is invisible",
        "            .find(|(fence, _)| rest.starts_with(fence))",
        "            .find(|(fence, _)| depth == 0 && rest.starts_with(fence))",
    ),
    (
        "a value that runs past its line is judged by a pattern instead of the walk",
        "fn opens_multiline_value(value: &str) -> bool {\n    scan(0, value).is_some()\n}",
        "fn opens_multiline_value(value: &str) -> bool {\n    value.starts_with(\"\\\"\\\"\\\"\")\n        || value.starts_with(\"'''\")\n        || (value.starts_with('[') && !value.contains(']'))\n}",
    ),
    (
        "an escape inside a basic string closes it early",
        "        escaped = !escaped && escapes && character == '\\\\';",
        "        escaped = false;",
    ),
    (
        "the removal side loses the veto it spent six rounds without",
        "    if !understood {\n        return existing.to_owned();\n    }",
        "    if false {\n        return existing.to_owned();\n    }",
    ),
    # Round 9. The OTHER path in this function that deletes a line, and the one
    # that had been left out of every round of care the first one got.
    (
        "the block-dropping loop reads a header inside somebody's prose",
        "        if outside[index] && line.trim_start().starts_with('[') {",
        "        if line.trim_start().starts_with('[') {",
    ),
    (
        SCRIPT,
        "the checker does not know an array-of-tables header",
        "header_shape='^[[:space:]]*(\\\\[[^]]*\\\\]|\\\\[\\\\[[^]]*\\\\]\\\\])[[:space:]]*(#.*)?$'",
        "header_shape='^[[:space:]]*\\\\[[^]]*\\\\][[:space:]]*(#.*)?$'",
    ),
    (
        SCRIPT,
        "the checker forgets the array around a multi-line string",
        "    scan(depth, substr(line, at))",
        "    scan(0, substr(line, at))",
    ),
    (
        SCRIPT,
        "the checker names a line for removal in a file it misread",
        'END { if (hits == "" || open != "") exit 1; print hits }',
        'END { if (hits == "") exit 1; print hits }',
    ),
    # Round 9, the rust pass. TOML defines a root table three ways and the veto
    # knew one of them, so a person who wrote down what this change TEACHES —
    # "the flag lives at `features.hooks`" — lost their whole configuration to
    # `Cannot declare features twice`. Three mutations because the one predicate
    # answers three different files.
    (
        "a header-less `features` definition is not a definition, so a table is appended beside it",
        '|| might_bind(line, "features")',
        "|| false",
    ),
    (
        "only whole key paths are compared, so a dotted key is not seen as defining its root",
        "            None if character == '.' => {\n                segments.push(&name[start..index]);\n                start = index + character.len_utf8();\n            }",
        "            None if character == '\\u{0}' => {\n                segments.push(&name[start..index]);\n                start = index + character.len_utf8();\n            }",
    ),
    # The deliberate over-reach, asserted so that narrowing it is a decision and
    # not an edit: the veto reads EVERY line, so a `features.hooks` sitting under
    # `[other]` — a different table, beside which `[features]` would be legal —
    # also declines. Cheap direction, and the one the other two vetoes take.
    (
        "the veto stops before the first table, and writes beside a definition it skipped",
        "        if lines\n            .iter()\n            .any(|line| might_open_table(line, \"features\") || might_bind(line, \"features\"))",
        "        if lines\n            .iter()\n            .take_while(|line| !is_table_header(line))\n            .any(|line| might_open_table(line, \"features\") || might_bind(line, \"features\"))",
    ),
    # And the checker's half of the same finding: it answered `not set` about a
    # file whose first line was `features.hooks = true`. Telling a person nothing
    # is amiss is the worst of the three things that line can say, so one
    # mutation takes the reading away and one takes away the boundary that keeps
    # it from claiming `[other]`'s dotted key is the root flag.
    (
        SCRIPT,
        "the checker does not read a root-level dotted key as the flag",
        '  (here && rooted && /^[[:space:]]*features[[:space:]]*\\.[[:space:]]*"?hooks"?[[:space:]]*=/) {',
        "  (here && rooted && /^[[:space:]]*zzz-never-matches/) {",
    ),
    (
        SCRIPT,
        "the root region never ends, so another table's dotted key is reported as the flag",
        "  here && $0 ~ header { inside = 0; rooted = 0 }",
        "  here && $0 ~ header { inside = 0 }",
    ),
    # Round 10, the rust pass. The rule for "where does this quoted thing end"
    # had a second expression — a plain `find(fence)` for the multi-line kinds
    # beside an escape-aware walk for the one-line ones — and `\"""` is TOML's
    # only way to write a literal `"""` inside a `"""` string, so the gap is in
    # a note explaining Operon's own config format. The first two put the split
    # back; the third is the direction the escape must NOT be read in.
    (
        "the multi-line fence search forgets escapes again",
        "            let Some(past) = closing_offset(body, fence, !literal) else {",
        "            let Some(past) = closing_offset(body, fence, false) else {",
    ),
    (
        "the continuation line's fence search forgets escapes again",
        "            let Some(past) = closing_offset(line, fence, !literal) else {\n                return open;\n            };",
        "            let Some(past) = closing_offset(line, fence, false) else {\n                return open;\n            };",
    ),
    (
        "a backslash escapes before a literal FENCE, which has no escapes either",
        "            let Some(past) = closing_offset(body, fence, !literal) else {",
        "            let Some(past) = closing_offset(body, fence, true) else {",
    ),
    (
        "a backslash escapes inside a literal string, which has no escapes",
        '                } else {\n                    ("\'", false)\n                };',
        '                } else {\n                    ("\'", true)\n                };',
    ),
    (
        SCRIPT,
        "the checker's fence search forgets escapes",
        '      at = closing_offset(substr(rest, 4), f, f == "\\"\\"\\"")',
        "      at = closing_offset(substr(rest, 4), f, 0)",
    ),
    (
        SCRIPT,
        "the checker escapes before a literal fence, which has no escapes",
        '      at = closing_offset(substr(rest, 4), f, f == "\\"\\"\\"")',
        "      at = closing_offset(substr(rest, 4), f, 1)",
    ),
    (
        SCRIPT,
        "the checker's continuation fence search forgets escapes",
        '    at = closing_offset(line, fence, fence == "\\"\\"\\"")',
        "    at = closing_offset(line, fence, 0)",
    ),
    # Round 11. The trust blocks were appended unconditionally, so every shape
    # this function had DECIDED to preserve — the hand-written inline table, the
    # multi-line array, the multi-line string — became a config Codex cannot
    # load the moment entries were non-empty: `[hooks.state."K"]` is a dotted
    # key and TOML will not let one extend a non-table. Each protection
    # producing the failure it was added to prevent.
    (
        "the trust blocks are appended onto a top-level hooks they cannot extend",
        "    if can_hold_hook_state(&lines, entries) {",
        "    if true {",
    ),
    # And the boundary of that veto, which is the part worth pinning rather than
    # the veto itself: a DOTTED `hooks.enabled = true` makes a real table, the
    # blocks extend it, and refusing there would cost the flag for nothing.
    (
        "any key beginning with `hooks` blocks the trust blocks, dotted ones included",
        "                key.trim_matches(['\"', '\\'']) == \"hooks\"",
        "                key.trim_matches(['\"', '\\'']).starts_with(\"hooks\")",
    ),
    # Round 12. The veto above knew the first segment of the path and round 11
    # stopped there, but `[hooks.state."K"]` creates TWO tables on its way to
    # the key. A document binding `hooks.state` leaves `hooks` a perfectly good
    # table and still cannot take the append. Closing the spelling instead of
    # the shape, for the fifth time in this change.
    (
        "the veto asks about `hooks` and not about `hooks.state`",
        'const HOOK_STATE_PATH: &[&str] = &["hooks", "state"];',
        'const HOOK_STATE_PATH: &[&str] = &["hooks"];',
    ),
    (
        "a table header is not where the keys below it hang from",
        "            table = segments.into_iter().map(unquote_segment).collect();",
        "            table = Vec::new();",
    ),
    (
        "an array of tables at `hooks` is a table the blocks can extend",
        "            if array && targets.iter().any(|target| target.starts_with(&table)) {",
        "            if targets.iter().any(|target| target.starts_with(&table)) {",
    ),
    (
        "the veto reads a line inside somebody's note as an assignment",
        "        if !outside[index] {\n            continue;\n        }",
        "        if false {\n            continue;\n        }",
    ),
    # Round 13. The `[features]` body veto compared the WHOLE key path while
    # every other reader in the file had been taught to compare the first
    # segment, so a `hooks.state.inner = true` under `[features]` binds `hooks`
    # as a table and the flag was inserted beside it: `dotted key hooks
    # attempted to extend non-table type (boolean)`. Two lines of valid TOML.
    # This puts the deleted `might_assign` back, inlined, so the mutation is
    # the defect and not a paraphrase of it.
    (
        "the [features] veto compares the whole key path again",
        '        .any(|line| might_bind(line, "hooks"))',
        '        .any(|line| line.split_once(\'=\').is_some_and(|(left, _)| left.trim().trim_matches([\'"\', \'\\\'\']).trim() == "hooks"))',
    ),
    # Round 14. Three readers had learned the path rule and the fourth — the
    # only header reader that never had a review round — still compared the
    # header as RAW TEXT, so Operon's own block written back in a different
    # shape was a block nobody recognised and nobody vetoed, and the append
    # declared the table twice. The rest are the leaf itself, which no reader
    # asked about at all.
    (
        "Operon's own block is recognised only in the exact shape it writes",
        '                && table_header_shape(&line).is_some_and(|(name, array)| {',
        '                && owned.iter().any(|path| line.trim_start().starts_with(&format!("[hooks.state.\\"{}\\"]", path[2]))) && table_header_shape(&line).is_some_and(|(name, array)| {',
    ),
    (
        'nothing asks whether the key about to be declared is already bound',
        '            .any(|target| stands_in_the_way(&bound, target, table.len()))',
        '            .any(|target| HOOK_STATE_PATH.starts_with(&bound) && !target.is_empty())',
    ),
    (
        'an array of tables at the leaf is not in the way',
        '            if array && targets.iter().any(|target| target.starts_with(&table)) {',
        '            if array && targets.iter().any(|target| HOOK_STATE_PATH.starts_with(&table) && !target.is_empty()) {',
    ),
    (
        'a header that reaches past the leaf is treated like a dotted key that does',
        '    target.starts_with(bound) || (bound.starts_with(target) && target.len() > table_depth)',
        '    target.starts_with(bound) || bound.starts_with(target)',
    ),
    (
        'a key that cannot be written and read back is written anyway',
        '    if !entries.iter().all(|(key, _)| writable(key)) {',
        '    if entries.is_empty() && !entries.is_empty() {',
    ),
    (
        SCRIPT,
        'exit 2 stops being distinguishable from bash failing to parse the script',
        '  echo "UNKNOWN: no config at $config"\n',
        '',
    ),
    # Round 15. The generalisation of round 14 lost a case the narrow reader
    # had handled — a key containing `]`, which `table_header_shape` cannot
    # segment, so Operon's own block was invisible to every reader and the
    # append declared the table twice. Two of these are that defect and its
    # neighbour on the deleting side; two are the promise the script's header
    # makes about `UNKNOWN:` at the three paths a test can reach.
    # Round 16. Round 15 put this refusal in `can_hold_hook_state`, which is the
    # path that DECLINES TO WRITE, and left the three paths that EDIT running
    # past the same unreadable header — the one that deletes took somebody's key
    # with it. It is one answer in `structural_lines` now, before any path runs,
    # and the mutation follows it there. The second mutation is the `]` test
    # that separates "we may have misread a header" from "this is a header to
    # nobody": dropping it refuses `[a` as well and takes the repair away from a
    # machine an older Operon broke.
    (
        'the top-level scan runs past a header it cannot read, and deletes under it',
        "                && (looks_like_table_header(&lines[*index])\n                    || opens_unreadable_table(&lines[*index]))",
        "                && looks_like_table_header(&lines[*index])",
    ),
    (
        'the append does not ask whether the document has a header it cannot read',
        "        if opens_unreadable_table(line) {\n            return false;\n        }",
        "        if false && opens_unreadable_table(line) {\n            return false;\n        }",
    ),
    (
        'an unreadable header and a line that is no header at all are treated alike',
        "    trimmed.starts_with('[') && trimmed.contains(']') && table_header_shape(line).is_none()",
        "    trimmed.starts_with('[') && table_header_shape(line).is_none()",
    ),
    # Round 17. The third reader of the same predicate, and the expensive one:
    # the drop loop removes Operon's own trust blocks so the append can put them
    # back, and the append is what `can_hold_hook_state` has just refused. Both
    # halves of the gate are mutated — dropping the flag entirely, and asking it
    # per line instead of per document, which is the plausible wrong grain: a
    # readable header of ours can precede the unreadable one, and asked per line
    # the loop deletes there and is refused anyway.
    (
        'the drop loop removes blocks the append will then refuse to write back',
        "            dropping = !holds_unreadable_table\n                && table_header_shape(&line)",
        "            dropping = table_header_shape(&line)",
    ),
    # Round 19. The claim two comments depend on, which was true and held by
    # nothing. `install_event_groups` is module-private with no test of its
    # own, so the guard sits at the caller: `install_codex_hooks` must hand
    # back an empty key list on a removal. The PREVIOUS version of that same
    # claim was false and survived three rounds, which is the argument for
    # spending a line on a claim that is currently true.
    (
        "a removal hands back the keys it did not write, so the caller remembers them",
        "    let mut written = Vec::new();\n    if !remove_only {",
        '    let mut written = Vec::new();\n    if remove_only {\n        written.push(("Stop".to_owned(), 0, 0));\n    }\n    if !remove_only {',
    ),
    # Round 18. The caveat a reader reaches for first, and review measured that
    # it passed the whole suite before the removal row existed: letting the
    # removal path delete because nothing appends after it. The gate is there
    # for the wider reason — we did not re-derive this data and cannot — and a
    # decision no fixture can see is a habit rather than a decision.
    (
        'the removal path is let through the gate, because nothing appends after it',
        "            dropping = !holds_unreadable_table",
        "            dropping = (!holds_unreadable_table || entries.is_empty())",
    ),
    (
        'the drop loop asks about the line in front of it rather than the whole document',
        "            dropping = !holds_unreadable_table\n                && table_header_shape(&line)",
        "            dropping = !opens_unreadable_table(&line)\n                && table_header_shape(&line)",
    ),
    (
        'the segment reader trims inside the quotes, so the deleting reader takes a table TOML says is not ours',
        '    segment.trim().trim_matches([\'"\', \'\\\'\'])\n}',
        '    segment.trim().trim_matches([\'"\', \'\\\'\']).trim()\n}',
    ),
    (
        SCRIPT,
        'an unrecognised option exits 2 with nothing a caller can read',
        '      echo "UNKNOWN: unrecognised option $1"\n',
        '',
    ),
    (
        SCRIPT,
        'a machine without codex exits 2 with nothing a caller can read',
        '  echo "UNKNOWN: codex is not on PATH"\n',
        '',
    ),
]


def run_tests():
    for name in TESTS:
        done = subprocess.run(
            ["cargo", "test", "--locked", name],
            cwd=TREE,
            capture_output=True,
            text=True,
        )
        if done.returncode != 0:
            return False
    return True


def main():
    originals = {path: path.read_text(encoding="utf-8") for path in (SOURCE, SCRIPT)}

    if not run_tests():
        print("refusing: the tests are already red before any mutation", file=sys.stderr)
        return 1

    failures = 0
    for entry in MUTATIONS:
        target, label, find, replace = entry if len(entry) == 4 else (SOURCE, *entry)
        original = originals[target]
        if find not in original:
            print(f"[SCRIPT BUG] {label}: the text to mutate is not in {target.name}")
            failures += 1
            continue
        target.write_text(original.replace(find, replace, 1), encoding="utf-8")
        green = run_tests()
        target.write_text(original, encoding="utf-8")
        if green:
            print(f"[NOT CAUGHT] {label}")
            failures += 1
        else:
            print(f"[caught    ] {label}")

    for path, text in originals.items():
        if path.read_text(encoding="utf-8") != text:
            print(f"{path} was not restored correctly", file=sys.stderr)
            return 1
    if not run_tests():
        print("the tree was not restored correctly", file=sys.stderr)
        return 1
    print(f"\n{len(MUTATIONS) - failures} of {len(MUTATIONS)} mutations caught")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
