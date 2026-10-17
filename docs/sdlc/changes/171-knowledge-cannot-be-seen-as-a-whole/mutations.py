#!/usr/bin/env python3
"""Change 171's mutation sweep: one break per requirement of spec.md, plus the
claim mutation (rule 4) on 「{n} 件が一致」. Requirement 9's time budget holds
only in a release build, so it is swept once by hand in release and recorded
in plan.md rather than here, where every check is a debug build.

The template's explanation of the technique and its six rules is in
docs/sdlc/templates/mutations.py; the code below is that file's, unchanged.
"""
import atexit
import os
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/171-knowledge-cannot-be-seen-as-a-whole")

RUN_SECONDS = 1800

TESTS = ["knowledge_map_", "a_knowledge_database_from_before_the_map_gains_its_table"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MAP = ROOT / "src/knowledge/map.rs"
STORE = ROOT / "src/knowledge.rs"
UI = ROOT / "src/ui/knowledge_map.rs"
APP = ROOT / "src/app/knowledge.rs"
MUTATIONS = {
    "req1-lines-join-any-two-entries": (
        MAP,
        ".filter(|(_, similarity)| *similarity >= KNOWLEDGE_MAP_EDGE_MIN_COSINE)",
        ".filter(|(_, similarity)| *similarity >= -1.0)",
    ),
    "req2-groups-of-one-kept": (
        MAP,
        ".filter(|nodes| nodes.len() >= KNOWLEDGE_MAP_MIN_GROUP)",
        ".filter(|nodes| nodes.len() >= 1)",
    ),
    "req3-group-named-by-least-central": (
        MAP,
        "members.into_iter().map(|(node, _)| node).collect()",
        "members.into_iter().rev().map(|(node, _)| node).collect()",
    ),
    "req4-springs-pull-nothing": (
        MAP,
        "let pull = (distance - rest) * (0.1 + 0.6 * similarity);",
        "let pull = 0.0 * (distance - rest) * (0.1 + 0.6 * similarity);",
    ),
    "req5-saved-position-kept-whatever-the-vector": (
        MAP,
        "(Some(position.vector_hash) == *fingerprint).then_some([position.x, position.y])",
        "true.then_some([position.x, position.y])",
    ),
    "req5-settling-moves-placed-entries-freely": (
        MAP,
        "extent * KNOWLEDGE_MAP_SETTLE_MAX_SHIFT,",
        "extent * 1000.0,",
    ),
    "req6-position-outlives-its-entry": (
        STORE,
        "entry_id INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE,",
        "entry_id INTEGER PRIMARY KEY,",
    ),
    "req6-save-writes-a-deleted-entry": (
        STORE,
        "SELECT ?1, ?2, ?3, ?4 WHERE EXISTS (SELECT 1 FROM entries WHERE id = ?1)",
        "SELECT ?1, ?2, ?3, ?4",
    ),
    "req7-vectorless-entry-on-top-of-the-rest": (
        MAP,
        "let radius = ((max[0] - min[0]).hypot(max[1] - min[1]) / 2.0) + 2.0;",
        "let radius = 0.0;",
    ),
    "req8-layout-requested-again-for-the-same-snapshot": (
        APP,
        ".is_some_and(|from| Arc::ptr_eq(from, &snapshot))",
        ".is_some_and(|from| !Arc::ptr_eq(from, &snapshot))",
    ),
    "req8-result-for-an-old-snapshot-drawn": (
        APP,
        ".is_some_and(|current| Arc::ptr_eq(current, &from))",
        ".is_some_and(|_current| true)",
    ),
    "req10-decision-drawn-as-a-disc": (
        UI,
        "KnowledgeKind::Decision => PointShape::Diamond,",
        "KnowledgeKind::Decision => PointShape::Disc,",
    ),
    "req10-size-ignores-references": (
        UI,
        "(KNOWLEDGE_MAP_MIN_RADIUS + 1.5 * uses.sqrt())",
        "(KNOWLEDGE_MAP_MIN_RADIUS + 0.0 * uses.sqrt())",
    ),
    "req11-outline-misses-members": (
        MAP,
        "hull: convex_hull(&positions),",
        "hull: convex_hull(&positions[..positions.len().min(2)]),",
    ),
    "req12-tooltip-miscounts-references": (
        UI,
        "n = entry.reference_count\n",
        "n = entry.reference_count + 1\n",
    ),
    "req13-map-choice-not-shared-with-the-list": (
        APP,
        "self.knowledge.open_entry = Some((project_id, entry_id));",
        "self.knowledge.open_entry = None;",
    ),
    "req14-search-hits-not-dimmed": (
        APP,
        "!(found && of_kind)",
        "!of_kind",
    ),
    "req14-claim-match-line-loses-its-words": (
        APP,
        'tf!("{n} 件が一致", n = count)',
        'tf!("{n} 件", n = count)',
    ),
    "req15-zoom-drifts-from-the-pointer": (
        UI,
        "self.offset = pointer - rect.min - egui::vec2(under[0], under[1]) * self.scale;",
        "self.offset *= 1.0;",
    ),
    "req15-resized-canvas-keeps-the-old-framing": (
        UI,
        "} else if view.fitted_size != rect.size() {",
        "} else if false {",
    ),
    "req16-entry-titles-overlap": (
        UI,
        "|| placed.iter().any(|(_, other)| other.intersects(candidate))",
        "",
    ),
    "req16-group-name-past-the-top-edge": (
        UI,
        "candidate.top() >= rect.top() && candidate.bottom() <= rect.bottom()",
        "candidate.top() > f32::MIN",
    ),
    "req16-group-names-overlap": (
        UI,
        "&& !groups.iter().any(|other| other.intersects(**candidate))",
        "",
    ),
    "req16-labels-placed-every-frame": (
        UI,
        "if self.key != Some(key) {",
        "if true {",
    ),
}

# 5. Nothing below this line is filled in.

# The marker a timed-out run prints. A timeout IS a catch — a hanging suite is
# not a green one — but it is a catch for a different reason than an assertion,
# and a reader has to be able to tell them apart to know whether the mutation
# proved anything.
TIMED_OUT = "時間切れ"

# How long to wait for a killed group's pipes to close before giving up on them.
# SIGKILL is immediate, so this only matters for a process that left the group by
# calling `setsid` for itself, or one in uninterruptible state. Bounded so that
# the code which exists to stop a hang cannot become one.
KILL_GRACE_SECONDS = 10

# Every file this sweep touches and what it held before. Filled once, up front,
# so a restore never has to re-read a file that may already be mutated. The
# signal handlers and `atexit` both read it, which is why it is module-level.
originals = {}
# The check currently running, so an interrupt can take its whole group down
# with it instead of orphaning a `cargo test`.
running = None
# Set once a signal handler has started. A Python-level handler runs in the eval
# loop, so without this a second signal would run `on_signal` inside itself.
handling = False
# Sibling temporaries that exist right now. `on_signal` ends the process with
# `os._exit`, which runs no `except` and no `finally`, so a write interrupted
# mid-flight cannot clean up after itself and the handler does it instead.
temporaries = set()


def write_atomically(path, text):
    """Replace `path`'s contents with `text`, all-or-nothing.

    A sibling temporary file in the same directory, then `os.replace`, which is
    a rename and the only write step the filesystem promises is atomic. The
    `write_text` this replaces truncates first, so an interrupt in the middle of
    a restore left half a file — which reads as a compiler error nobody can
    place rather than as a mutation somebody wrote.

    The name is registered before the write and dropped after it, so that a
    signal landing in the middle does not leave a `.<name>.XXXXXXXX` behind. One
    window stays open and is named rather than papered over: a signal delivered
    between `mkstemp` returning and the `add` below leaks that one file. It is
    prefixed with the target's own name, so a reader can see what it was.

    THE MODE IS CARRIED ACROSS. `mkstemp` creates at `0600` and `os.replace`
    swaps the whole inode, so a rename-based write silently replaces the target's
    permissions with the temporary's — the `write_text` this replaces wrote
    through the existing inode and could not. Measured while this file was being
    reviewed: the sweep mutated `scripts/check-installed-build.sh`, restored its
    bytes exactly, and left it `600` with no executable bit. The tree looked
    restored, `git diff` was empty, and the only trace was a mode change in
    `git diff --cached --summary`. A sweep over a hook would have disarmed it.
    """
    try:
        mode = stat.S_IMODE(os.stat(path).st_mode)
    except OSError:
        mode = None
    handle, temporary = tempfile.mkstemp(dir=str(path.parent), prefix=f".{path.name}.")
    temporaries.add(temporary)
    try:
        with os.fdopen(handle, "w", encoding="utf-8", newline="") as file:
            file.write(text)
            file.flush()
            os.fsync(file.fileno())
        if mode is not None:
            os.chmod(temporary, mode)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise
    finally:
        temporaries.discard(temporary)


def restore_all():
    """Put every mutated file back. Returns the paths that are still wrong.

    Idempotent, and cheap when there is nothing to do: a file already holding
    its original bytes is not rewritten, so `finally`, `atexit` and a signal
    handler may all run this in sequence without three writes and three mtimes.
    """
    still_wrong = []
    for path, text in originals.items():
        try:
            if path.read_text(encoding="utf-8") == text:
                continue
        except OSError:
            pass
        try:
            write_atomically(path, text)
        except OSError:
            still_wrong.append(path)
    return still_wrong


def kill_running_check():
    """Take the running check's whole process group down, if there is one."""
    child = running
    if child is None or child.poll() is not None:
        return
    try:
        os.killpg(child.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass
    try:
        child.communicate(timeout=KILL_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        pass


def on_signal(number, _frame):
    """Restore, stop the check, and leave — for the signals a `finally` misses.

    SIGTERM raises nothing in Python: the interpreter dies where it stands and no
    `finally` runs, so before this handler existed a sweep killed by an
    orchestrator or a logout left the mutated file in the working tree. SIGINT
    does raise, and is handled here too, so that both interrupts leave the tree
    in one state rather than in two a reader has to tell apart.

    THE FIRST THING IT DOES IS STOP LISTENING. This runs in the eval loop like
    any other Python code, and `kill_running_check` below blocks for up to
    `KILL_GRACE_SECONDS`, so a second interrupt — a second Ctrl-C, or the
    SIGKILL-after-SIGTERM an impatient orchestrator sends — would run this
    function again inside itself. The inner call would `os._exit` out of the
    outer one, abandoning a half-written temporary and calling `communicate()`
    twice on one `Popen`. `SIG_IGN` refuses the second signal, and `handling`
    covers the case of one already flagged before the handler was replaced.
    """
    global handling
    for interrupt in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(interrupt, signal.SIG_IGN)
    if handling:
        return
    handling = True
    kill_running_check()
    still_wrong = restore_all()
    for leftover in list(temporaries):
        # Whatever `write_atomically` was in the middle of when the signal
        # arrived. Its `except` will never run — see `os._exit` below.
        try:
            Path(leftover).unlink(missing_ok=True)
        except OSError:
            pass
    print(f"\nシグナル {number} を受け取りました。変異は戻してあります。")
    for path in still_wrong:
        print(f"!! 戻せませんでした: {path}")
    # `os._exit` because the work is done and `atexit` would only repeat it. It
    # runs no `except` and no `finally` on the way out, which is why the
    # temporaries above are unlinked here by hand rather than left to the
    # `write_atomically` call that created them.
    os._exit(1 if still_wrong else 130)


for _number in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(_number, on_signal)
atexit.register(restore_all)


def run_checks():
    """Every check, in order. `(green, output)`.

    `start_new_session=True` puts the check in a process group of its own, and
    the bound kills the GROUP. `subprocess.run(timeout=…)` kills only the
    process Python forked, and the check a sweep actually runs is `cargo test` —
    which runs the test binary as a grandchild. Bounding the wait without
    killing the group is true and useless: the sweep finishes and reports, and
    the hanging test binary stays on the machine until somebody notices. Found
    in review, reproduced twice, and the reason this is Popen rather than the
    one-liner it looks like it should be.

    A grandchild that calls `setsid` for itself leaves the group and outlives
    this, which nothing here can prevent. No check named above does — and the
    wait for a killed group's pipes is bounded anyway, so even that cannot turn
    the code which exists to stop a hang into one.
    """
    global running
    for command in CHECKS:
        running = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        child = running
        try:
            out, err = child.communicate(timeout=RUN_SECONDS)
        except subprocess.TimeoutExpired:
            kill_running_check()
            return False, (
                f"{TIMED_OUT} — {RUN_SECONDS} 秒で終わりませんでした: {' '.join(command)}\n"
                "変異がチェックをハングさせています。catch ではありますが、"
                "主張が守られた証拠ではありません。"
            )
        finally:
            if running is child:
                running = None
        if child.returncode != 0:
            return False, out + err
    return True, ""


def reason_from(output):
    """The one line worth printing beside a catch."""
    for line in output.splitlines():
        stripped = line.strip()
        if TIMED_OUT in stripped or "panicked at" in stripped or "guard" in stripped.lower():
            return stripped
    return "(理由行が読めません)"


for _path, _old, _new in MUTATIONS.values():
    originals[_path] = _path.read_text(encoding="utf-8")

failures = []
try:
    for name, (path, old, new) in MUTATIONS.items():
        text = originals[path]
        hits = text.count(old)
        if hits == 0:
            print(f"{name}: SETUP FAILED — 置換対象が見つかりません:\n{old}")
            sys.exit(1)
        if hits > 1:
            # Not an error — `str.replace` taking every occurrence is the
            # documented behaviour — but a mutation that moved four call sites
            # when its name says one is a different experiment from the one the
            # change directory will claim was run.
            print(f"{name}: 注意 — 置換対象が {hits} 箇所にあり、そのすべてが変異します")
        write_atomically(path, text.replace(old, new))
        green, output = run_checks()
        write_atomically(path, text)
        if green:
            failures.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのにチェックが通りました")
        else:
            print(f"{name}: caught — {reason_from(output)}")
finally:
    kill_running_check()
    unrestored = restore_all()

# Said out loud rather than assumed. A restore that did not take leaves broken
# source in a tracked file and the next `git add -A` commits it, so this is the
# one outcome that must not be mistaken for a sweep that merely found something.
if unrestored:
    print()
    for path in unrestored:
        print(f"!! 変異を戻せませんでした: {path}")
    print("作業ツリーに壊れたファイルが残っています。commit する前に git diff を読んでください。")
    sys.exit(1)

print()
print(f"{len(MUTATIONS) - len(failures)} of {len(MUTATIONS)} caught")
sys.exit(1 if failures else 0)
