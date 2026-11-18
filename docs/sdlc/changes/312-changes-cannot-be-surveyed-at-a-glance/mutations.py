#!/usr/bin/env python3
"""The mutation sweep for change 312: the change map carries every change at
its recorded position, what waits for a person first, the documents as
written, and the pipeline sends a session to it. Each mutation takes one of
those away and a `change_map_` test must refuse to pass.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = ["change_map_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MAP = ROOT / "scripts/change-map.py"
SKILL = ROOT / ".claude/skills/feature-pilot/SKILL.md"
README = ROOT / "docs/sdlc/README.md"

MUTATIONS = {
    # Requirement 1.
    "one change directory is left off the map": (
        MAP,
        "return sorted(entry for entry in base.iterdir() if entry.is_dir())",
        "return sorted(entry for entry in base.iterdir() if entry.is_dir())[:-1]",
    ),
    "the position is read from any block, sets: included": (
        MAP,
        "            inside = line.startswith(name + \":\")\n            continue\n        if not inside:\n            continue\n        found = re.match(r'^  ([\\w-]+)",
        "            inside = True\n            continue\n        if not inside:\n            continue\n        found = re.match(r'^  ([\\w-]+)",
    ),
    # Requirement 2.
    "a change waiting for the person is drawn as under way": (
        MAP,
        '"awaiting-user": "needs",',
        '"awaiting-user": "flight",',
    ),
    "the summary lists what is under way before what waits": (
        MAP,
        'for group, label in (("needs", "要対応"), ("flight", "進行中")):',
        'for group, label in (("flight", "進行中"), ("needs", "要対応")):',
    ),
    "the summary drops the sentence that says what to do": (
        MAP,
        'lines.append("    次にすること: " + change["resume"])',
        "pass",
    ),
    "the page puts the areas above what waits": (
        MAP,
        '  <section id="needs"><h2>あなたの判断を待っています <small id="needs-n"></small></h2><div id="needs-list"></div></section>\n  <section id="flight"><h2>進行中 <small id="flight-n"></small></h2><div id="flight-list"></div></section>\n  <section id="areas"><h2>領域ごとの変更 <small>どの部分に、これまで何をしてきたか</small></h2><div id="area-list"></div></section>',
        '  <section id="areas"><h2>領域ごとの変更 <small>どの部分に、これまで何をしてきたか</small></h2><div id="area-list"></div></section>\n  <section id="needs"><h2>あなたの判断を待っています <small id="needs-n"></small></h2><div id="needs-list"></div></section>\n  <section id="flight"><h2>進行中 <small id="flight-n"></small></h2><div id="flight-list"></div></section>',
    ),
    # Requirement 3.
    "other working copies are not read": (
        MAP,
        "            if directory.name not in changes:",
        "            if False:",
    ),
    "a detached copy outranks the one on a branch": (
        MAP,
        'trees.sort(key=lambda tree: tree[1] == "")',
        'trees.sort(key=lambda tree: tree[1] != "")',
    ),
    "a failed git listing stops the map": (
        MAP,
        "    except (OSError, subprocess.SubprocessError):\n        return None\n    if result.returncode != 0 or len(result.stdout) > GIT_BYTES:\n        return None",
        "    except (OSError, subprocess.SubprocessError):\n        return None\n    if result.returncode != 0 or len(result.stdout) > GIT_BYTES:\n        sys.exit(1)",
    ),
    "the summary does not say the other copies were unread": (
        MAP,
        'lines.append("他の作業コピーを読めませんでした（このチェックアウトの記録だけです）")',
        'lines.append("このチェックアウトの記録です")',
    ),
    # Requirement 4.
    "a document is tidied on its way into the page": (
        MAP,
        'docs.append({"name": doc, "text": text, "truncated": cut})',
        'docs.append({"name": doc, "text": text.strip(), "truncated": cut})',
    ),
    "an open finding loses the sentence written for the person": (
        MAP,
        'found[-1]["person"] = person.group(1)',
        'found[-1]["person"] = ""',
    ),
    # Requirement 5.
    "the test file counts as an area": (
        MAP,
        'if (path in ("src/tests.rs", "scripts/mutation_sweep.py")',
        'if (path in ("scripts/mutation_sweep.py",)',
    ),
    "a table row's unquoted path is not read": (
        MAP,
        '            if line.startswith("|"):\n                cells = line.split("|")',
        '            if False:\n                cells = line.split("|")',
    ),
    "a plan's prose paths count as files that change": (
        MAP,
        "        if inside:\n            lines.append(line)\n    return lines",
        "        lines.append(line)\n    return lines",
    ),
    "--find does not look at titles": (
        MAP,
        'or word in change["title"].lower()',
        "or False",
    ),
    "--find writes the page": (
        MAP,
        "        sys.stdout.write(find(model, arguments.find))\n        return 0",
        '        sys.stdout.write(find(model, arguments.find))\n        write_page(model, root / ".tmp/change-map.html")\n        return 0',
    ),
    # Requirement 6.
    "a document's text can close the data block": (
        MAP,
        '("<", "\\\\u003c")',
        '("<", "<")',
    ),
    "the line separator JavaScript refuses is left raw": (
        MAP,
        '("\\u2028", "\\\\u2028")',
        '("\\u2028", "\\u2028")',
    ),
    "the page is written into the tracked tree by default": (
        MAP,
        'out = Path(arguments.out) if arguments.out else root / ".tmp/change-map.html"',
        'out = Path(arguments.out) if arguments.out else root / "change-map.html"',
    ),
    # Requirement 7.
    "an undefined status is drawn as a finished change": (
        MAP,
        '"group": "needs" if problem else GROUPS[status],',
        '"group": "done" if problem else GROUPS[status],',
    ),
    "a change with no state file is dropped": (
        MAP,
        "    for directory in change_directories(root):\n        changes[directory.name]",
        '    for directory in change_directories(root):\n        if not (directory / "state.yaml").is_file():\n            continue\n        changes[directory.name]',
    ),
    "a document has no size ceiling": (
        MAP,
        "DOC_BYTES = 256 * 1024",
        "DOC_BYTES = 256 * 1024 * 1024",
    ),
    "a cut document is not marked": (
        MAP,
        "    cut = len(data) > limit\n",
        "    cut = False\n",
    ),
    "a change directory has no record ceiling": (
        MAP,
        "DOCS_PER_CHANGE = 40",
        "DOCS_PER_CHANGE = 4000",
    ),
    "a root with no changes is not an error": (
        MAP,
        '        sys.stderr.write("docs/sdlc/changes がありません: %s\\n" % root)\n        return 2',
        '        sys.stderr.write("docs/sdlc/changes がありません: %s\\n" % root)\n        return 0',
    ),
    # Requirement 8.
    "a status the pipeline defines has no place on the map": (
        MAP,
        '    "reviewing": "flight",\n',
        "",
    ),
    # Requirement 9, and rule 4: the sentences a session and a person read.
    "the skill goes back to listing one checkout": (
        SKILL,
        "1. **Resume or new.** `python3 scripts/change-map.py` lists what waits for a",
        "1. **Resume or new.** `ls docs/sdlc/changes/` lists what waits for a",
    ),
    "the skill no longer says a change may live in another worktree": (
        SKILL,
        "   another worktree. If one matches the request",
        "   this checkout. If one matches the request",
    ),
    "placement no longer reads the earlier changes": (
        SKILL,
        "  run `python3 scripts/change-map.py --find <module or noun>` for the earlier",
        "  look for the earlier",
    ),
    "the README claims a second record is kept": (
        README,
        "no second record of any position",
        "a second record of every position",
    ),
    "the README forgets which change narrowed the refusal": (
        README,
        "so change 312 generates a view of it",
        "so a script generates a view of it",
    ),
    "the README drops the map from what was built": (
        README,
        "| every change at a glance | `scripts/change-map.py` —",
        "| every change at a glance | a script —",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
