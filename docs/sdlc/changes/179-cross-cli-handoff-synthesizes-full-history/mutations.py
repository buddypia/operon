#!/usr/bin/env python3
"""Mutation sweep for change 179 (compact Handoff bundle)."""
import atexit
import os
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

# 1. WHERE.
ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/179-cross-cli-handoff-synthesizes-full-history")

# 2. HOW LONG one check may take before the sweep stops waiting for it.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN.
TESTS = [
    "compact_handoff_bundle_bounds_turns_and_bytes_and_strips_tool_spam",
    "compact_handoff_bundle_peels_nested_envelopes_and_preserves_display_goal",
    "import_full_cli_history_writes_handoff_bundle_and_manifest_without_touching_destination_store",
    "finish_full_history_import_launches_fresh_session_with_handoff_bundle_prompt",
    "the_restore_modal_names_what_it_is_doing",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

# 4. WHAT TO BREAK.
MUTATIONS = {
    "handoff-bundle-drops-byte-bound": (
        ROOT / "src/history.rs",
        "if candidate.len() <= HANDOFF_BUNDLE_MAX_BYTES {",
        "if true {",
    ),
    "handoff-bundle-keeps-tool-spam": (
        ROOT / "src/history.rs",
        '&& !trimmed.starts_with("[Tool call:")',
        "&& true",
    ),
    "handoff-bundle-nests-envelope": (
        ROOT / "src/history.rs",
        "original_task_from_session_goal(text)",
        "text.to_owned()",
    ),
    "handoff-manifest-claim": (
        ROOT / "src/history.rs",
        '"restore": "compact handoff bundle",',
        '"restore": "full conversation history",',
    ),
    "handoff-session-marks-full-history-true": (
        ROOT / "src/app.rs",
        "let uses_handoff_bundle = !imported.handoff_bundle.is_empty();",
        "let uses_handoff_bundle = false;",
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
