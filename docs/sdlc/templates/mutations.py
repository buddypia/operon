#!/usr/bin/env python3
"""The mutation sweep a change copies. Put it in the change directory as
`mutations.py` and fill in the five blocks below.

WHAT A SWEEP IS FOR. A guard nobody has watched fail is a guess about what it
guards. The sweep is how that watching is done once per requirement and can be
re-done by anyone: break the thing a guard exists to catch, run the guard, and
require it to go red. `.claude/skills/sdlc/SKILL.md` step 10 asks for one; a
change's `state.yaml` records the result as `guards-verified-by-mutation`.

A SWEEP DELIBERATELY BREAKS TRACKED SOURCE AND PUTS IT BACK. That is the whole
technique and it is also the whole hazard: between the write and the restore,
the working tree holds a file somebody wrote to be wrong. Everything about the
restore below — atomic, signal-handled, verified afterwards — is there because
the window is real and the file in it is `src/history.rs`.

THIS FILE EXISTS BECAUSE THERE WAS NO TEMPLATE. Five sweeps were written before
it, each copied from whichever one was nearest, and all five inherited the same
unbounded `subprocess.run` — a mutation that makes a test hang hangs the sweep,
and the sweep is the evidence a change's guards work, so the failure mode is
"this change has no evidence", arriving as a terminal nobody looks at for an
hour. Change 067's review carried it as a nit; change 068 put the bound here
rather than into five closed changes' records of sweeps that were already run.

SIX RULES THE SWEEPS BEFORE THIS ONE LEARNED THE HARD WAY. They are in the code
below; they are restated here because a copy that drops one looks fine.

1. **Assert the target is present before replacing it.** `str.replace` that
   matches nothing returns the string unchanged and the run then reports the
   unmutated tree as "caught". A sweep that cannot fail is worse than no sweep.
2. **Write the mutated file rather than `shutil.copy2`-ing it.** copy2 preserves
   mtime, cargo reuses the previous binary, and the unmutated result gets
   reported as the mutated one.
3. **Bound every run, and make the bound reach the whole tree.** Bounding the
   *wait* is not enough. `subprocess.run(timeout=…)` kills the process Python
   forked and nothing under it, and a sweep's check is `cargo test`, which runs
   the test binary as a grandchild. `run_checks` below therefore starts a
   session and kills the group; a copy that simplifies it back into
   `subprocess.run` finishes on time and leaves the hanging test running.
4. **Mutate the claim, not only the counter.** If the change added or altered a
   sentence a person reads, one mutation has to weaken that sentence — leave
   every count correct and make the words wrong. `REVIEW.md` makes a
   counters-only sweep past a new sentence an Important finding, and
   `evals/007-mutate-the-claim-not-the-counter.md` is the same rule as a prompt.
5. **Restore through a signal, not only through a `finally`.** A `finally` runs
   on a normal exit and on Ctrl-C, and not on SIGTERM — so a sweep killed by an
   orchestrator, a logout, or a shutdown left the mutated file in the tree.
   Measured before this was written: SIGTERM mid-run, and the subject file still
   read `BROKEN` afterwards with the check still running beside it. The next
   `git add -A` would have committed it.
6. **Put the file back atomically, then check that you did.** A restore that
   truncates and rewrites can be interrupted halfway and leave half a file,
   which is worse than the mutation because it does not look deliberate. Write a
   sibling and rename; rename is the only step the filesystem promises is
   all-or-nothing. Then read every file back before exiting and say so loudly if
   one did not take. **A rename replaces the inode, so carry the mode across** —
   `mkstemp` creates at `0600`, and the first version of this file restored a
   755 script as a 600 one with its bytes intact, which no `git diff` and no
   read-back of the contents can see.

A WEAK MUTATION PASSES FOR THE WRONG REASON. Two shapes to avoid, both met:
commenting a line out when a guard searches for a substring the comment still
contains — delete it instead; and a replacement that breaks the file's syntax,
so both directions are judged by a parse error rather than by the claim.
"""
import atexit
import os
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

# 1. WHERE. An absolute path: this runs from anywhere, and a worktree session's
#    `GIT_DIR`/`GIT_WORK_TREE` do not point here.
ROOT = Path("<absolute path to this checkout>")

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = ["<the_guard_this_change_added>"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
MUTATIONS = {
    "<what-this-breaks>": (ROOT / "<file>", "<text that is there>", "<text that breaks it>"),
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
