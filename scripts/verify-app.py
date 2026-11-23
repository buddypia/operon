#!/usr/bin/env python3
"""Prove a feature in the built application, not in a test of it. Change 314.

Starts target/debug/operon apart from the person's data and tmux, follows the
steps a feature's document gives, keeps a picture and the window's tree for
each, stops what it started, and says pass or fail.

  python3 scripts/verify-app.py run --list            features that have steps
  python3 scripts/verify-app.py run palette           build, launch, steps, stop
  python3 scripts/verify-app.py launch [--seed project] [--plan]
  python3 scripts/verify-app.py doctor                is this instance worth driving?
  python3 scripts/verify-app.py tree | shot NAME      look by hand at a launched one
  python3 scripts/verify-app.py stop

Exit 0 pass, 1 a step did not hold, 2 the run could not start.

Steps live in docs/features/<id>/spec.md under `## Seen in the running app`,
indented four spaces, one `verb words` per line:

  start empty|project   first line; the store the application opens with
  click WORDS           press the one control whose name is, or contains, WORDS
  see WORDS             WORDS are on the window (waits a few seconds)
  gone WORDS            WORDS are not on the window (waits a few seconds)
  shot NAME             keep a picture of the window

Controls are pressed through the accessibility tree by the `agent-desktop`
CLI: the pointer is not moved and nothing is typed. That tree offers no typing
without taking the keyboard, so a feature that needs typed input starts from a
seed instead. The window is brought to the front when a run starts, and again
when a press goes unanswered: the application handles a press only while it is
drawing frames, and macOS stops a window that is not in front from drawing
(measured in change 314: a press to a window behind others returned in 16 ms
and took effect only once the window was brought forward).

Isolation is by environment and nothing else: the application's data folder
follows HOME, and tmux follows TMUX_TMPDIR once TMUX is unset. The scratch
folder sits directly under /tmp because the hook socket and the tmux socket are
unix sockets, whose paths end at 104 bytes on macOS.
"""
import argparse
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERBS = ("start", "click", "see", "gone", "shot")
SEEDS = ("empty", "project")
SECTION = "## Seen in the running app"
STEP_LINES_MAX = 64
TREE_BYTES_MAX = 8 * 1024 * 1024
SCRATCH_PARENT = "/tmp"
SCRATCH_PREFIX = "opv."
DRIVER_SECONDS = 30
BUILD_SECONDS = 1800
SEED_TEST = "verify_app_seed_writes_a_store_the_app_loads"
SEED_PROJECT = "verify-demo"


class CannotStart(Exception):
    """Exit 2: nothing was driven."""


def say(message):
    print(message, file=sys.stderr)


# --- the steps a document gives -------------------------------------------------


def feature_ids():
    ids = []
    for line in (ROOT / "docs/features/index.yaml").read_text().splitlines():
        if line.startswith("  - id: "):
            ids.append(line[len("  - id: "):].strip())
    return ids


def steps_in(text, where):
    """The `verb words` lines of the section, or None when there is no section."""
    if SECTION not in text:
        return None
    body = text.split(SECTION, 1)[1]
    lines = []
    for line in body.splitlines()[1:]:
        if line.startswith("## "):
            break
        if line.startswith("    ") and line.strip():
            lines.append(line.strip())
    if len(lines) > STEP_LINES_MAX:
        raise CannotStart(f"{where}: {len(lines)} step lines, more than {STEP_LINES_MAX}")
    steps = []
    for number, line in enumerate(lines, 1):
        verb, _, words = line.partition(" ")
        words = words.strip()
        if verb not in VERBS or not words:
            raise CannotStart(f"{where}: step {number} is not `verb words`: {line!r}")
        if (number == 1) != (verb == "start"):
            raise CannotStart(f"{where}: step {number}: `start` is the first line and only the first")
        if verb == "start" and words not in SEEDS:
            raise CannotStart(f"{where}: step 1 is not `start empty` or `start project`: {line!r}")
        steps.append((verb, words))
    if not steps:
        raise CannotStart(f"{where}: the section has no step")
    return steps


def features_with_steps():
    found = []
    for feature in feature_ids():
        spec = ROOT / "docs/features" / feature / "spec.md"
        if spec.is_file() and SECTION in spec.read_text():
            found.append(feature)
    return found


# --- what the window says --------------------------------------------------------


def plain(text):
    """A name as a person reads it: icon glyphs and the space around them gone."""
    kept = "".join(c for c in (text or "") if not 0xE000 <= ord(c) <= 0xF8FF)
    return " ".join(kept.split())


def nodes_of(tree):
    stack = [tree]
    while stack:
        node = stack.pop()
        if isinstance(node, dict):
            if "role" in node:
                yield node
            stack.extend(reversed(node.get("children") or []))


def says(tree, words):
    return any(
        words in plain(node.get("name")) or words in plain(node.get("value"))
        for node in nodes_of(tree)
    )


def control_for(tree, words):
    """The ref of the one control answering to the words, or why there is none."""
    controls = [
        node
        for node in nodes_of(tree)
        if node.get("ref_id")
        and "Click" in (node.get("available_actions") or [])
        and node.get("role") != "statictext"
    ]
    exact = [node for node in controls if plain(node.get("name")) == words]
    partial = [node for node in controls if words in plain(node.get("name"))]
    for matches in (exact, partial):
        if len(matches) == 1:
            return matches[0]["ref_id"], None
        if len(matches) > 1:
            return None, f"{len(matches)} controls answer to {words!r}"
    return None, f"no control answers to {words!r}"


def judge(verb, tree, words):
    """(holds, detail). `click` returns the ref to press as its detail."""
    if verb == "see":
        return (True, "") if says(tree, words) else (False, f"{words!r} is not on the window")
    if verb == "gone":
        return (False, f"{words!r} is still on the window") if says(tree, words) else (True, "")
    if verb == "click":
        ref, why = control_for(tree, words)
        return (True, ref) if ref else (False, why)
    raise CannotStart(f"`{verb}` is not judged against a tree")


def tree_from(raw):
    if len(raw) > TREE_BYTES_MAX:
        return None, f"the window's tree is {len(raw)} bytes, more than {TREE_BYTES_MAX}"
    envelope = answer_of(raw)
    if not envelope.get("ok"):
        return None, f"the window could not be read: {envelope.get('error') or raw[:200]!r}"
    data = envelope.get("data")
    tree = data.get("tree") if isinstance(data, dict) else None
    if not isinstance(tree, dict):
        return None, "the window's answer holds no tree"
    return tree, None


# --- the instance ----------------------------------------------------------------


def launch_plan(root):
    return {
        "root": str(root),
        "environment": {"HOME": str(Path(root) / "h"), "TMUX_TMPDIR": str(Path(root) / "t")},
        "unset": ["TMUX", "TMUX_PANE"],
    }


def record_path(state):
    return state / "instance.json"


def read_record(state):
    try:
        return json.loads(record_path(state).read_text())
    except (OSError, ValueError):
        return None


def command_of(pid):
    result = subprocess.run(
        ["ps", "-o", "command=", "-p", str(pid)], capture_output=True, text=True, timeout=10
    )
    return result.stdout.strip()


def alive(record):
    """Alive, and still the binary the record names: a pid is reused."""
    command = command_of(record["pid"])
    return bool(command) and record["binary"] in command


def launch(state, binary, seed):
    binary = Path(binary).resolve()
    if not binary.is_file():
        raise CannotStart(f"{binary} is not built: run `cargo build --locked`")
    previous = read_record(state)
    if previous and alive(previous):
        raise CannotStart(
            f"an instance of this checkout is running (pid {previous['pid']}): "
            "run `python3 scripts/verify-app.py stop` first"
        )
    if previous:
        discard(state, previous)
    state.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix=SCRATCH_PREFIX, dir=SCRATCH_PARENT))
    plan = launch_plan(root)
    home = Path(plan["environment"]["HOME"])
    home.mkdir()
    Path(plan["environment"]["TMUX_TMPDIR"]).mkdir(mode=0o700)
    environment = {k: v for k, v in os.environ.items() if k not in plan["unset"]}
    environment.update(plan["environment"])
    try:
        if seed == "project":
            seed_project(root, home)
        log = open(root / "operon.log", "wb")
        process = subprocess.Popen(
            [str(binary)],
            env=environment,
            cwd=str(root),
            stdin=subprocess.DEVNULL,
            stdout=log,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
    except BaseException:
        shutil.rmtree(root, ignore_errors=True)
        raise
    record = {
        "root": str(root),
        "pid": process.pid,
        "binary": str(binary),
        "tmux_tmpdir": plan["environment"]["TMUX_TMPDIR"],
        "started": int(time.time()),
    }
    record_path(state).write_text(json.dumps(record))
    return record


def seed_project(root, home):
    project = root / SEED_PROJECT
    subprocess.run(["git", "init", "-q", str(project)], check=True, timeout=60)
    environment = dict(os.environ)
    environment["OPERON_VERIFY_SEED_HOME"] = str(home)
    environment["OPERON_VERIFY_SEED_PROJECT"] = str(project)
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), SEED_TEST],
        env=environment,
        capture_output=True,
        text=True,
        timeout=BUILD_SECONDS,
    )
    if result.returncode != 0:
        raise CannotStart(f"the seed store could not be written:\n{result.stdout[-2000:]}{result.stderr[-2000:]}")


def discard(state, record):
    """Remove the scratch folder and the record. Never the evidence."""
    root = Path(record["root"])
    if root.parent == Path(SCRATCH_PARENT) and root.name.startswith(SCRATCH_PREFIX):
        shutil.rmtree(root, ignore_errors=True)
    try:
        record_path(state).unlink()
    except OSError:
        pass


def stop(state):
    record = read_record(state)
    if record is None:
        return 0
    command = command_of(record["pid"])
    if command and record["binary"] not in command:
        say(
            f"pid {record['pid']} is not the binary this checkout started "
            f"({record['binary']}); it is `{command}` and is left running. "
            f"Remove {record_path(state)} by hand once you have checked."
        )
        return 1
    if command:
        try:
            os.killpg(record["pid"], signal.SIGTERM)
        except OSError:
            pass
        for _ in range(50):
            if not command_of(record["pid"]):
                break
            time.sleep(0.1)
        else:
            try:
                os.killpg(record["pid"], signal.SIGKILL)
            except OSError:
                pass
    tmux_directory = record.get("tmux_tmpdir")
    if tmux_directory and shutil.which("tmux"):
        for socket in Path(tmux_directory).glob("tmux-*/default"):
            subprocess.run(
                ["tmux", "-S", str(socket), "kill-server"], capture_output=True, timeout=10
            )
    discard(state, record)
    return 0


# --- the tool that reads and presses the window -----------------------------------


def driver_command():
    named = os.environ.get("VERIFY_APP_DRIVER")
    if named:
        return named
    found = shutil.which("agent-desktop")
    if not found:
        raise CannotStart(
            "`agent-desktop` is not installed: it reads and presses the window. "
            "See https://agent-browser.dev/desktop, then run `agent-desktop permissions --guide`"
        )
    return found


def drive(record, *arguments):
    environment = dict(os.environ)
    environment["VERIFY_APP_PID"] = str(record["pid"])
    try:
        # agent-desktop refuses a temporary directory other users could write.
        private = subprocess.run(
            ["getconf", "DARWIN_USER_TEMP_DIR"], capture_output=True, text=True, timeout=10
        ).stdout.strip()
        if private:
            environment["TMPDIR"] = private
    except OSError:
        pass
    try:
        result = subprocess.run(
            [driver_command(), *arguments],
            env=environment,
            capture_output=True,
            timeout=DRIVER_SECONDS,
        )
    except (subprocess.TimeoutExpired, OSError) as error:
        say(f"the driver did not answer `{arguments[0]}`: {error}")
        return b""
    return result.stdout


def answer_of(raw):
    """The driver's envelope, or an empty one when it is not a JSON object."""
    try:
        envelope = json.loads(raw)
    except ValueError:
        return {}
    return envelope if isinstance(envelope, dict) else {}


def window_of(record):
    windows = answer_of(drive(record, "list-windows")).get("data")
    if not isinstance(windows, list):
        return None
    own = [w for w in windows if isinstance(w, dict) and w.get("pid") == record["pid"]]
    # A process also owns helper windows nobody sees; the one a person uses is
    # visible and answers to accessibility. The stand-in driver says neither.
    shown = [w for w in own if w.get("visible", True) and w.get("accessible", True)]

    def area(window):
        bounds = window.get("bounds", {})
        return bounds.get("width", 0) * bounds.get("height", 0)

    shown.sort(key=area, reverse=True)
    return shown[0]["id"] if shown else None


def read_tree(record):
    window = window_of(record)
    if window is None:
        return None, "the application has no window"
    return tree_from(drive(record, "snapshot", "--window-id", window, "--compact"))


def picture(record, path):
    window = window_of(record)
    if window is None:
        return False
    answer = drive(record, "screenshot", "--window-id", window, str(path))
    if not Path(path).is_file():
        say(f"screenshot of {window}: {answer[:300]!r}")
        return False
    return True


def doctor(state, wait_seconds, check_build=True):
    """Each line is a fact and, when it fails, what to do. Returns the failures."""
    failures = []
    record = read_record(state)
    if record is None or not alive(record):
        return ["no instance of this checkout is running: `python3 scripts/verify-app.py launch`"]
    if check_build:
        binary = Path(record["binary"])
        # src/tests.rs is behind #[cfg(test)]: it is not in the binary.
        sources = (p for p in (ROOT / "src").rglob("*.rs") if p.name != "tests.rs")
        newest = max((p.stat().st_mtime for p in sources), default=0)
        if binary.stat().st_mtime < newest:
            failures.append("the binary is older than src/: `cargo build --locked`, then launch again")
    status = answer_of(drive(record, "status")).get("data")
    try:
        state_of = status["permissions"]["accessibility"]["state"]
    except (KeyError, TypeError):
        state_of = None
        failures.append("`agent-desktop status` did not answer")
    if state_of is not None and state_of != "granted":
        failures.append(
            f"accessibility permission is `{state_of}`: run `agent-desktop permissions --guide`"
        )
    deadline = time.time() + wait_seconds
    while True:
        tree, why = read_tree(record)
        if tree is not None or time.time() >= deadline:
            break
        time.sleep(0.5)
    if tree is None:
        failures.append(f"the window does not answer: {why}")
    return failures


# --- a run -------------------------------------------------------------------------


def run(state, label, steps, binary, build, wait_seconds):
    if build:
        say("building target/debug/operon ...")
        built = subprocess.run(
            ["cargo", "build", "--locked", "--manifest-path", str(ROOT / "Cargo.toml")],
            timeout=BUILD_SECONDS,
        )
        if built.returncode != 0:
            raise CannotStart("`cargo build --locked` failed")
    evidence = state / f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{label}"
    record = launch(state, binary, steps[0][1])
    evidence.mkdir(parents=True)

    def finish():
        """The one way out: keep the log, stop what was started, name the evidence."""
        try:
            shutil.copyfile(Path(record["root"]) / "operon.log", evidence / "operon.log")
        except OSError:
            pass
        stop(state)
        print(evidence)

    def interrupted(number, _frame):
        sys.exit(128 + number)

    signal.signal(signal.SIGINT, interrupted)
    signal.signal(signal.SIGTERM, interrupted)

    # Pass, fail, a signal, or an answer nobody foresaw: nothing this run
    # started stays on the person's screen.
    try:
        return follow(state, record, steps, evidence, build, wait_seconds)
    finally:
        finish()


def shown(tree):
    """What a person could tell apart on the window: refs change on every read."""
    return [(n.get("role"), plain(n.get("name")), plain(n.get("value"))) for n in nodes_of(tree)]


def bring_forward(record):
    window = window_of(record)
    if window is not None:
        drive(record, "focus-window", "--window-id", window)


def press(record, words, tree, wait_seconds):
    """Press the control and watch the window answer. A press to a window that
    is not drawing looks delivered to the tool that sent it and waits unseen, so
    delivery is judged by what the window then shows; when nothing changed, the
    window is brought forward and the press sent once more, which is safe
    exactly because nothing changed. A press the tool says it did not deliver
    (the control was redrawn between the read and the press) is sent again on a
    fresh read, as the tool's own answer says is safe."""
    before = shown(tree)
    undelivered = 0
    attempt = 0
    while attempt < 2:
        holds, detail = judge("click", tree, words)
        if not holds:
            return False, detail
        pressed = answer_of(drive(record, "click", detail))
        if not pressed.get("ok"):
            error = pressed.get("error")
            error = error if isinstance(error, dict) else {"message": error}
            safe = (error.get("disposition") or {}).get("delivery") == "not_delivered"
            undelivered += 1
            time.sleep(0.3)
            fresh, _why = read_tree(record)
            # The tool's word against the window's: a press it calls refused
            # has landed when the window answered it (seen live: the palette
            # row is gone because the page it opens is already drawn).
            if fresh is not None and shown(fresh) != before:
                return True, ""
            if not safe or undelivered > 3:
                return False, f"the press was refused: {error.get('code')} {error.get('message')}"
            tree = fresh if fresh is not None else tree
            continue
        deadline = time.time() + min(wait_seconds, 2.0)
        while True:
            time.sleep(0.2)
            after, _why = read_tree(record)
            if after is not None and shown(after) != before:
                return True, ""
            if time.time() >= deadline:
                break
        if after is not None:
            tree = after
        attempt += 1
        if attempt < 2:
            bring_forward(record)
    return False, "the press changed nothing on the window (is another window kept in front of it?)"


def follow(state, record, steps, evidence, build, wait_seconds):
    failures = doctor(state, max(wait_seconds, 1) * 4, check_build=build)
    if failures:
        raise CannotStart("; ".join(failures))

    bring_forward(record)
    verdicts = open(evidence / "steps.txt", "w")
    for number, (verb, words) in enumerate(steps, 1):
        holds, detail = True, ""
        if verb == "shot":
            # The system's capture times out now and then on a busy window, and
            # one attempt can outlast the wait: three tries, whatever the clock says.
            for _ in range(3):
                holds = picture(record, evidence / f"{number:02d}-{words}.png")
                if holds:
                    break
                time.sleep(0.5)
            detail = "" if holds else "the window could not be pictured"
        elif verb != "start":
            deadline = time.time() + wait_seconds
            while True:
                tree, why = read_tree(record)
                holds, detail = (False, why) if tree is None else judge(verb, tree, words)
                if holds or time.time() >= deadline:
                    break
                time.sleep(0.5)
            if tree is not None:
                (evidence / f"{number:02d}-tree.json").write_text(json.dumps(tree, ensure_ascii=False))
            if holds and verb == "click":
                holds, detail = press(record, words, tree, wait_seconds)
        if not holds:
            picture(record, evidence / f"{number:02d}-failed.png")
            verdicts.write(f"FAILED {verb} {words}: {detail}\n")
            verdicts.close()
            say(f"step {number} did not hold: {verb} {words}: {detail}")
            return 1
        verdicts.write(f"ok {verb} {words}\n")
        verdicts.flush()
    verdicts.close()
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--state", type=Path, default=ROOT / "target/verify", help="where the instance record and the evidence go")
    commands = parser.add_subparsers(dest="command", required=True)
    launching = commands.add_parser("launch")
    launching.add_argument("--seed", choices=SEEDS, default="empty")
    launching.add_argument("--plan", action="store_true", help="print the launch and start nothing")
    launching.add_argument("--binary", default=str(ROOT / "target/debug/operon"))
    commands.add_parser("doctor")
    commands.add_parser("stop")
    commands.add_parser("tree")
    commands.add_parser("shot").add_argument("name")
    judging = commands.add_parser("eval-step")
    judging.add_argument("verb", choices=("click", "see", "gone"))
    judging.add_argument("tree")
    judging.add_argument("words")
    running = commands.add_parser("run")
    running.add_argument("feature", nargs="?")
    running.add_argument("--list", action="store_true")
    running.add_argument("--steps", type=Path, help="a document other than the feature's own")
    running.add_argument("--no-build", action="store_true")
    running.add_argument("--binary", help="a stand-in for target/debug/operon; implies --no-build")
    running.add_argument("--wait-seconds", type=float, default=5.0)
    arguments = parser.parse_args()
    state = arguments.state.resolve()

    if arguments.command == "launch":
        if arguments.plan:
            # The name mkdtemp would give, without making it.
            print(json.dumps(launch_plan(Path(SCRATCH_PARENT) / f"{SCRATCH_PREFIX}xxxxxxxx")))
            return 0
        record = launch(state, arguments.binary, arguments.seed)
        print(json.dumps(record))
        return 0
    if arguments.command == "stop":
        return stop(state)
    if arguments.command == "doctor":
        failures = doctor(state, 5)
        for failure in failures:
            say(failure)
        return 2 if failures else 0
    if arguments.command == "eval-step":
        tree, why = tree_from(Path(arguments.tree).read_bytes())
        holds, detail = (False, why) if tree is None else judge(arguments.verb, tree, arguments.words)
        if holds and detail:
            print(detail)
        if not holds:
            say(detail)
        return 0 if holds else 1
    if arguments.command in ("tree", "shot"):
        record = read_record(state)
        if record is None or not alive(record):
            raise CannotStart("no instance of this checkout is running")
        if arguments.command == "tree":
            tree, why = read_tree(record)
            if tree is None:
                raise CannotStart(why)
            for node in nodes_of(tree):
                name = plain(node.get("name")) or plain(node.get("value"))
                if name:
                    print(f"{node.get('role')}\t{'control' if node.get('ref_id') else '-'}\t{name}")
            return 0
        state.mkdir(parents=True, exist_ok=True)
        target = state / f"{arguments.name}.png"
        if not picture(record, target):
            raise CannotStart("the window could not be pictured")
        print(target)
        return 0

    if arguments.list:
        for feature in features_with_steps():
            print(feature)
        return 0
    if arguments.steps:
        label, document = arguments.steps.stem, arguments.steps
    elif arguments.feature:
        if arguments.feature not in feature_ids():
            raise CannotStart(f"`{arguments.feature}` is not a feature of docs/features/index.yaml: see `run --list`")
        label, document = arguments.feature, ROOT / "docs/features" / arguments.feature / "spec.md"
    else:
        raise CannotStart("name a feature, or `run --list`")
    steps = steps_in(document.read_text(), str(document))
    if steps is None:
        raise CannotStart(f"{document} has no `{SECTION}` section: see `run --list`")
    binary = arguments.binary or str(ROOT / "target/debug/operon")
    build = not (arguments.no_build or arguments.binary)
    return run(state, label, steps, binary, build, arguments.wait_seconds)


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    try:
        sys.exit(main())
    except CannotStart as error:
        say(f"verify-app: {error}")
        sys.exit(2)
