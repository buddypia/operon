#!/usr/bin/env python3
"""Photograph every screen of docs/features/flow.yaml in the running application.

Change 316. The product map drew each screen as a box with its name; this takes
a picture of each one as the application draws it, apart from the person's data,
so the map can show the real screen and where on it each way out is pressed.

A screen of the flow carries `reach` — the words to press, `;`-separated, from a
fresh launch with the demo project (`-` for none) — or `noshot`, a Japanese
reason it cannot be photographed. For each `reach` screen this launches the
application through scripts/verify-app.py (change 314: its own home and tmux,
never the person's), presses the words with that runner's `press`, reads the
window's tree with the bounds of every control, photographs the window alone,
and stops the application. Pictures and entries go to docs/images/screens/:

    <id>.jpg        the window at the display's scale, JPEG quality 72
    screens.json    per screen: picture, window size in points, scale, the
                    controls (name and bounds in points from the window's
                    top-left corner), the words on the window, when and at
                    which commit it was taken

    python3 scripts/capture-screens.py                 build, then every screen
    python3 scripts/capture-screens.py --only ID ...   those screens
    python3 scripts/capture-screens.py --plan          print what would be pressed

Exit 0 every screen taken; 1 a screen could not be reached (its earlier picture
and entry are kept, and the run goes on to the next); 2 nothing could start.

The window appears on the person's screen while it runs, about ten seconds a
screen, and is brought to the front, as with `verify-app.py run`. Controls are
pressed through the accessibility tree: the pointer is not moved and nothing is
typed. `CAPTURE_PICTURE` names a stand-in for the picture tools (called as
`tool <window-number> <out.jpg>`), and `VERIFY_APP_DRIVER` one for the window
tool, so the suite can run the whole path without a window.
"""

import argparse
import datetime
import importlib.util
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FLOW = ROOT / "docs/features/flow.yaml"
OUT = ROOT / "docs/images/screens"
INDEX = "screens.json"
PICTURE_BYTES = 600 * 1024
QUALITY = "72"
TOOL_SECONDS = 30
BUILD_SECONDS = 1800
SCREEN_SECONDS = 120
RUN_SECONDS = 60 * 60
SETTLE_SECONDS = 3.0
CONTROLS = 300
WORDS = 400


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# One reader of the flat files and one judge of a control's name: the map and the
# runner's, not a third copy (identifier SSOT).
runner = load("verify_app", "verify-app.py")
mapper = load("change_map", "change-map.py")


class CannotStart(Exception):
    pass


def say(message):
    print(f"capture-screens: {message}", file=sys.stderr)


def plan_of(flow_text):
    """(screens to take, screens with a reason not to, problems). Each screen
    to take is {"id", "press": [words]}."""
    take, skip, problems = [], {}, []
    for screen in mapper.parse_flat(flow_text, "screens", "id"):
        sid = screen.get("id", "")
        reach, noshot = screen.get("reach"), screen.get("noshot")
        if (reach is None) == (noshot is None):
            problems.append(f"{sid}: needs exactly one of `reach` and `noshot`")
            continue
        if noshot is not None:
            skip[sid] = noshot
            continue
        words = [] if reach.strip() == "-" else [w.strip() for w in reach.split(";")]
        if any(not w for w in words):
            problems.append(f"{sid}: `reach` has an empty word")
            continue
        take.append({"id": sid, "press": words})
    return take, skip, problems


def jpeg_size(data):
    """(width, height) from a JPEG's frame header, or None."""
    if data[:2] != b"\xff\xd8":
        return None
    at = 2
    while at + 4 <= len(data):
        if data[at] != 0xFF:
            return None
        marker = data[at + 1]
        length = int.from_bytes(data[at + 2:at + 4], "big")
        if 0xC0 <= marker <= 0xCF and marker not in (0xC4, 0xC8, 0xCC):
            if at + 9 > len(data):
                return None
            height = int.from_bytes(data[at + 5:at + 7], "big")
            width = int.from_bytes(data[at + 7:at + 9], "big")
            return width, height
        at += 2 + length
    return None


def tool(arguments):
    try:
        done = subprocess.run(arguments, capture_output=True, timeout=TOOL_SECONDS)
    except (subprocess.TimeoutExpired, OSError) as error:
        return False, str(error)
    return done.returncode == 0, (done.stderr or done.stdout)[-400:].decode(errors="replace")


def photograph(number, destination):
    """The window alone, without its shadow, as a JPEG at `destination`."""
    stand_in = os.environ.get("CAPTURE_PICTURE")
    if stand_in:
        return tool([stand_in, number, str(destination)])
    raw = destination.with_suffix(".png")
    try:
        ok, why = tool(["screencapture", "-x", "-o", f"-l{number}", str(raw)])
        if not ok or not raw.is_file():
            return False, f"screencapture: {why}"
        return tool(["sips", "-s", "format", "jpeg", "-s", "formatOptions", QUALITY,
                     str(raw), "--out", str(destination)])
    finally:
        raw.unlink(missing_ok=True)


def bounds_of(node):
    b = node.get("bounds") or {}
    try:
        return float(b["x"]), float(b["y"]), float(b["width"]), float(b["height"])
    except (KeyError, TypeError, ValueError):
        return None


def read_window(record):
    """(window id, tree with bounds) or raise."""
    window = runner.window_of(record)
    if window is None:
        raise RuntimeError("the application has no window")
    raw = runner.drive(record, "snapshot", "--window-id", window, "--compact",
                       "--include-bounds", "--max-depth", "40")
    tree, why = runner.tree_from(raw)
    if tree is None:
        raise RuntimeError(why)
    return window, tree


def entry_of(tree):
    """Window size, controls and words, all in points from the window's corner."""
    frame = bounds_of(tree)
    if frame is None or frame[2] <= 0 or frame[3] <= 0:
        raise RuntimeError("the window's tree gives no size")
    wx, wy, ww, wh = frame
    controls, words = [], []
    for node in runner.nodes_of(tree):
        name = runner.plain(node.get("name"))
        for word in (name, runner.plain(node.get("value"))):
            if word and word not in words and len(words) < WORDS:
                words.append(word)
        if not (name and node.get("ref_id") and "Click" in (node.get("available_actions") or [])
                and node.get("role") != "statictext"):
            continue
        box = bounds_of(node)
        if box is None or box[2] <= 0 or box[3] <= 0:
            continue
        x, y = box[0] - wx, box[1] - wy
        # Inside the window only: a control scrolled out of view has bounds
        # past the edge, and an area there would sit on nothing.
        if x < 0 or y < 0 or x + box[2] > ww + 0.5 or y + box[3] > wh + 0.5:
            continue
        if len(controls) < CONTROLS:
            controls.append({"name": name, "x": round(x, 1), "y": round(y, 1),
                             "w": round(box[2], 1), "h": round(box[3], 1)})
    return {"window": {"w": ww, "h": wh}, "controls": controls, "words": words}


def settle(record):
    """Wait until two reads 300 ms apart agree, at most SETTLE_SECONDS."""
    deadline = time.time() + SETTLE_SECONDS
    before = None
    while time.time() < deadline:
        tree, _why = runner.read_tree(record)
        now = runner.shown(tree) if tree is not None else None
        if now is not None and now == before:
            return
        before = now
        time.sleep(0.3)


def commit_of():
    try:
        done = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"],
                              capture_output=True, text=True, timeout=10)
        return done.stdout.strip() or "unknown"
    except (subprocess.TimeoutExpired, OSError):
        return "unknown"


def read_index(out):
    try:
        index = json.loads((out / INDEX).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {"screens": {}}
    if not isinstance(index, dict) or not isinstance(index.get("screens"), dict):
        return {"screens": {}}
    return index


def write_index(out, index):
    temporary = out / f".{INDEX}.tmp"
    temporary.write_text(json.dumps(index, ensure_ascii=False, indent=1, sort_keys=True) + "\n",
                         encoding="utf-8")
    os.replace(temporary, out / INDEX)


def take(screen, state, binary, seed, out, wait_seconds, commit):
    """Launch, press, read, photograph, stop. Returns the entry or raises."""
    started = time.time()
    record = runner.launch(state, binary, seed)
    # Not a dot file: screencapture refuses to write one.
    temporary = out / f"{screen['id']}.part.jpg"
    try:
        deadline = time.time() + max(wait_seconds, 1.0) * 4
        while runner.read_tree(record)[0] is None:
            if time.time() > deadline:
                raise RuntimeError("the window did not answer")
            time.sleep(0.5)
        runner.bring_forward(record)
        for word in screen["press"]:
            if time.time() - started > SCREEN_SECONDS:
                raise RuntimeError(f"took longer than {SCREEN_SECONDS} s")
            tree, why = runner.read_tree(record)
            if tree is None:
                raise RuntimeError(why)
            pressed, why = runner.press(record, word, tree, wait_seconds)
            if not pressed:
                raise RuntimeError(f"press {word!r}: {why}")
        settle(record)
        window, tree = read_window(record)
        entry = entry_of(tree)
        number = "".join(c for c in window if c.isdigit())
        if not number:
            raise RuntimeError(f"the window id {window!r} holds no number")
        ok, why = photograph(number, temporary)
        if not ok or not temporary.is_file():
            raise RuntimeError(f"the picture could not be taken: {why}")
        data = temporary.read_bytes()
        size = jpeg_size(data)
        if size is None:
            raise RuntimeError("the picture is not a JPEG")
        if len(data) > PICTURE_BYTES:
            raise RuntimeError(f"the picture is {len(data)} bytes, more than {PICTURE_BYTES}")
        scale = size[0] / entry["window"]["w"]
        if abs(size[1] - entry["window"]["h"] * scale) > 1:
            raise RuntimeError(f"the picture is {size[0]}x{size[1]}, not the window's shape")
        name = f"{screen['id']}.jpg"
        os.replace(temporary, out / name)
        entry.update({
            "picture": name,
            "scale": round(scale, 3),
            "taken": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "commit": commit,
            "press": screen["press"],
        })
        return entry
    finally:
        temporary.unlink(missing_ok=True)
        runner.stop(state)


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--plan", action="store_true", help="print what would be pressed; start nothing")
    parser.add_argument("--only", nargs="+", metavar="SCREEN", help="these screens only")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--binary", help="a stand-in for target/debug/operon; implies --no-build")
    parser.add_argument("--seed", choices=runner.SEEDS, default="project")
    parser.add_argument("--flow", type=Path, default=FLOW)
    parser.add_argument("--out", type=Path, default=OUT)
    parser.add_argument("--state", type=Path, default=ROOT / "target/capture")
    parser.add_argument("--wait-seconds", type=float, default=3.0)
    arguments = parser.parse_args()

    try:
        flow_text = arguments.flow.read_text(encoding="utf-8")
    except OSError as error:
        say(f"{arguments.flow} cannot be read: {error}")
        return 2
    screens, skipped, problems = plan_of(flow_text)
    if problems:
        for problem in problems:
            say(problem)
        return 2
    if arguments.only:
        known = {s["id"] for s in screens}
        unknown = [sid for sid in arguments.only if sid not in known]
        if unknown:
            say(f"not a screen with `reach`: {' '.join(unknown)}")
            return 2
        screens = [s for s in screens if s["id"] in arguments.only]
    if arguments.plan:
        print(json.dumps({"screens": screens, "noshot": skipped}, ensure_ascii=False, indent=1))
        return 0

    binary = Path(arguments.binary or ROOT / "target/debug/operon")
    if not arguments.binary and not arguments.no_build:
        say("building target/debug/operon ...")
        try:
            built = subprocess.run(
                ["cargo", "build", "--locked", "--manifest-path", str(ROOT / "Cargo.toml")],
                timeout=BUILD_SECONDS)
        except subprocess.TimeoutExpired:
            say("`cargo build --locked` took too long")
            return 2
        if built.returncode != 0:
            say("`cargo build --locked` failed")
            return 2
    if not binary.is_file():
        say(f"{binary} is not built: run `cargo build --locked`")
        return 2

    def interrupted(number, _frame):
        sys.exit(128 + number)

    signal.signal(signal.SIGINT, interrupted)
    signal.signal(signal.SIGTERM, interrupted)

    out = arguments.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    state = arguments.state.resolve()
    index = read_index(out)
    commit = commit_of()
    failures = []
    run_deadline = time.time() + RUN_SECONDS
    try:
        for screen in screens:
            if time.time() > run_deadline:
                failures.append((screen["id"], f"the run passed {RUN_SECONDS // 60} min"))
                continue
            try:
                index["screens"][screen["id"]] = take(
                    screen, state, binary, arguments.seed, out, arguments.wait_seconds, commit)
                write_index(out, index)
                print(f"took {screen['id']}")
            except runner.CannotStart as error:
                say(str(error))
                return 2
            except RuntimeError as error:
                failures.append((screen["id"], str(error)))
                say(f"{screen['id']}: {error} (its earlier picture is kept)")
        if not arguments.only:
            # A screen that left the flow, or now has `noshot`, leaves the index.
            wanted = {s["id"] for s in screens}
            for gone in [sid for sid in index["screens"] if sid not in wanted]:
                picture = index["screens"].pop(gone).get("picture")
                if isinstance(picture, str) and "/" not in picture:
                    (out / picture).unlink(missing_ok=True)
            write_index(out, index)
    finally:
        runner.stop(state)
    if failures:
        for sid, why in failures:
            say(f"not taken: {sid}: {why}")
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except runner.CannotStart as error:
        say(str(error))
        sys.exit(2)
