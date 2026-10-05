#!/usr/bin/env python3
"""Change 069's sweep. Copied from docs/sdlc/templates/mutations.py — the second
change to use it, and the first to use it after editing it, so it is also the
evidence that the template's own hardening did not break the thing it is for.

One mutation per requirement, plus the two claim mutations REVIEW.md asks for.
Both leave every count and every exit code correct and make a printed sentence
untrue: `stamp-refusal-blames-the-repository` tells a reader that a stamp of
`HEAD` names a commit which is merely absent, and `git-failure-worded-as-stale`
calls a git that could not answer a bundle that is behind. Neither moves a
counter, and a sweep that only moved counters would report both as caught while
nothing had asked about the sentence.
"""
import atexit
import os
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

# 1. WHERE.
ROOT = Path(__file__).resolve().parents[4]

# 2. HOW LONG one check may take before the sweep stops waiting for it.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN.
TESTS = [
    "the_installed_build_check_refuses_a_stamp_it_cannot_verify",
    "the_installed_build_check_answers_zero_when_it_cannot_ask",
    "the_installed_build_check_does_not_call_a_git_failure_stale",
    "the_installed_build_check_reads_its_stamp_key_out_of_the_packager",
    "the_installed_build_check_reads_a_path_that_begins_with_a_hyphen",
    "the_mutation_sweep_template_puts_the_tree_back_when_it_is_killed",
    "the_mutation_sweep_template_restores_the_mode_it_found",
    "the_mutation_sweep_template_ends_a_run_that_will_not_finish",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

CHECKER = ROOT / "scripts/check-installed-build.sh"
TEMPLATE = ROOT / "docs/sdlc/templates/mutations.py"

# 4. WHAT TO BREAK.
MUTATIONS = {
    # A — the whole finding. Without this block the stamp goes to git as a
    # revision and `HEAD` resolves, so any bundle at all reports CURRENT.
    "stamp-used-as-a-revision": (
        CHECKER,
        """if [ "$stamp_is_an_object_name" = no ]; then""",
        """if false; then""",
    ),
    # A, THE CLAIM. The refusal stays, the exit code stays, and the sentence
    # blames the repository for not having a commit — when the value was never a
    # commit id. Same verdict, different thing to go and do: one reader looks for
    # another branch, the other rebuilds the bundle.
    "stamp-refusal-blames-the-repository": (
        CHECKER,
        """  echo "installed build: UNKNOWN COMMIT — $bundle carries a $stamp_key that is not an"
  echo "  object name: \\"$installed_commit\\". The stamp is read from outside this"
  echo "  repository and is spent as a git revision, so a value git would resolve to"
  echo "  something else — a branch, HEAD, a reflog entry — is refused rather than"
  echo "  followed. Rebuild the bundle with: bash scripts/package-macos.sh\"""",
        """  echo "installed build: UNKNOWN COMMIT — $bundle was built from $installed_commit,"
  echo "  which is not in this repository. Installed from another branch, or from a"
  echo "  commit since rewritten; nothing here can say whether it is behind.\"""",
    ),
    # C — the readability guard around the packager. Without it `set -e` ends the
    # script at sed's own error, at an exit code this file's header defines as
    # "the bundle is behind".
    "packager-read-unguarded": (
        CHECKER,
        """stamp_key=""
if [ -r "$packager" ]; then
  stamp_key="$(sed -n 's/^source_commit_key="\\([^"]*\\)"$/\\1/p' "$packager" 2>/dev/null | head -1 || true)"
fi""",
        """stamp_key="$(sed -n 's/^source_commit_key="\\([^"]*\\)"$/\\1/p' "$packager" | head -1)\"""",
    ),
    # E — three answers back into two. `merge-base --is-ancestor` returns 128 on
    # error, and folding that into "not an ancestor" turns a git that could not
    # answer into a positive claim that the bundle is behind.
    "git-failure-called-stale": (
        CHECKER,
        """  1)
    echo "installed build: STALE""",
        """  1 | *)
    echo "installed build: STALE""",
    ),
    # E, THE CLAIM. The branch stays, exit 0 stays — and the sentence a person
    # reads says the bundle is behind anyway.
    "git-failure-worded-as-stale": (
        CHECKER,
        """    echo "installed build: UNKNOWN — git could not compare $installed_commit with\"""",
        """    echo "installed build: STALE — git could not compare $installed_commit with\"""",
    ),
    # F — the handlers. A `finally` covers a normal exit and Ctrl-C and not
    # SIGTERM, so without these the mutated file stays in the working tree.
    "sweep-ignores-sigterm": (
        TEMPLATE,
        """for _number in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(_number, on_signal)
""",
        "",
    ),
    # F, the third part, and the only defect this change introduced rather than
    # fixed. The restore is atomic and its bytes are exact, and it drops the
    # mode: `os.replace` swaps the inode and `mkstemp` created it at 0600. A
    # sweep over a hook would leave the tree clean and the gate disarmed.
    "restore-drops-the-file-mode": (
        TEMPLATE,
        """        if mode is not None:
            os.chmod(temporary, mode)
""",
        "",
    ),
    # F, the other half. The tree is restored and the check it was running is
    # left behind — which for a real sweep is a `cargo test` that keeps going.
    "killed-sweep-leaves-its-check-running": (
        TEMPLATE,
        """    kill_running_check()
    still_wrong = restore_all()""",
        """    still_wrong = restore_all()""",
    ),
}

# 5. Nothing below this line is filled in.

TIMED_OUT = "時間切れ"

KILL_GRACE_SECONDS = 10

originals = {}
running = None
handling = False
temporaries = set()


def write_atomically(path, text):
    """Replace `path`'s contents with `text`, all-or-nothing, mode and all."""
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
    """Put every mutated file back. Returns the paths that are still wrong."""
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
    """Restore, stop the check, and leave — for the signals a `finally` misses."""
    global handling
    for interrupt in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(interrupt, signal.SIG_IGN)
    if handling:
        return
    handling = True
    kill_running_check()
    still_wrong = restore_all()
    for leftover in list(temporaries):
        try:
            Path(leftover).unlink(missing_ok=True)
        except OSError:
            pass
    print(f"\nシグナル {number} を受け取りました。変異は戻してあります。")
    for path in still_wrong:
        print(f"!! 戻せませんでした: {path}")
    os._exit(1 if still_wrong else 130)


for _number in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(_number, on_signal)
atexit.register(restore_all)


def run_checks():
    """Every check, in order. `(green, output)`."""
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

if unrestored:
    print()
    for path in unrestored:
        print(f"!! 変異を戻せませんでした: {path}")
    print("作業ツリーに壊れたファイルが残っています。commit する前に git diff を読んでください。")
    sys.exit(1)

print()
print(f"{len(MUTATIONS) - len(failures)} of {len(MUTATIONS)} caught")
sys.exit(1 if failures else 0)
