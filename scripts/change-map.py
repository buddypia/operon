#!/usr/bin/env python3
"""Every change at a glance, and its documents on demand. Change 312.

    python3 scripts/change-map.py               summary, and .tmp/change-map.html
    python3 scripts/change-map.py --find WORD   the changes of an area, or with
                                                WORD in their name or title
    python3 scripts/change-map.py --json        the model, without documents

The map is generated from the records the pipeline already keeps — each
change's state.yaml, its documents, review.yaml, approvals.log — and from
nothing else. It writes nothing back, keeps no ledger, and its page is not
committed: a second record of a change's position is the thing
.claude/skills/feature-pilot/SKILL.md forbids, and a generated view of the
first record is not one.

A change under way lives in another worktree until it lands, so the other
working copies are read too: one `git worktree list`, ten seconds, 64 KiB.
Ceilings: 256 KiB per document, 40 records per change, 64 working copies.

exit 0 written or printed, 2 usage or no docs/sdlc/changes
"""
import argparse
import datetime
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

DOC_BYTES = 256 * 1024
DOCS_PER_CHANGE = 40
WORKTREES = 64
GIT_SECONDS = 10
GIT_BYTES = 64 * 1024

STAGES = ("plan", "design", "build", "test", "deploy", "maintain")

# Where each status is drawn. `change_map_knows_every_status_the_pipeline_
# defines` holds these keys to the set in docs/sdlc/templates/state.yaml.
GROUPS = {
    "awaiting-user": "needs",
    "blocked": "needs",
    "failed": "needs",
    "planning": "flight",
    "ready": "flight",
    "in-progress": "flight",
    "reviewing": "flight",
    "done": "done",
    "archived": "archived",
}
GROUP_ORDER = ("needs", "flight", "done", "archived")

# Areas that are not a module of the crate. A module's area is its own name.
PIPELINE_PREFIXES = (
    "scripts/", ".claude/", ".cli/", ".githooks/", ".github/", ".codex/",
    "docs/sdlc/", "evals/",
)
PIPELINE_FILES = ("Makefile", "AGENTS.md", "CLAUDE.md", "REVIEW.md")
AREA_LABELS = {
    "pipeline": "開発プロセス",
    "dependencies": "依存関係",
    "docs": "文書",
    "unfiled": "未分類",
}

# The order a person reads a change in; anything else follows by name.
DOC_ORDER = (
    "state.yaml", "intent.md", "bug.md", "spec.md", "screen.md", "plan.md",
    "handoff.md", "review.yaml", "verification.yaml", "approvals.log",
)
DOC_SUFFIXES = (".md", ".yaml", ".log")
REPOSITORY_PATH = (r"(?<![\w./-])(?:(?:src|scripts|\.claude|\.cli|\.githooks|\.github|evals|docs)"
                   r"/[\w./-]+)")


def read_text(path, limit=DOC_BYTES):
    """The file's text, cut at `limit` bytes, and whether it was cut."""
    try:
        with open(path, "rb") as handle:
            data = handle.read(limit + 1)
    except OSError:
        return None, False
    cut = len(data) > limit
    return data[:limit].decode("utf-8", errors="ignore"), cut


def block(text, name):
    """The `  key: "value"` rows directly under top-level `name:`. A value may
    run over several lines until its closing quote, as the template's does."""
    rows, inside, pending = [], False, None
    for line in text.splitlines():
        if pending is not None:
            piece = line.strip()
            if piece.endswith('"'):
                rows.append((pending[0], (pending[1] + " " + piece[:-1]).strip()))
                pending = None
            else:
                pending = (pending[0], pending[1] + " " + piece)
            continue
        if re.match(r"^[A-Za-z_][\w-]*:", line):
            inside = line.startswith(name + ":")
            continue
        if not inside:
            continue
        found = re.match(r'^  ([\w-]+):\s*"(.*)$', line)
        if not found:
            continue
        key, rest = found.group(1), found.group(2).rstrip()
        if rest.endswith('"'):
            rows.append((key, rest[:-1]))
        else:
            pending = (key, rest)
    return rows


def route_stages(root):
    text, _ = read_text(root / "docs/sdlc/routes.yaml")
    routes = {}
    for key, value in block(text or "", "routes"):
        stages = [stage for stage in value.split(" ")[0].split(",") if stage in STAGES]
        routes[key] = stages
    return routes


def template_statuses(root):
    text, _ = read_text(root / "docs/sdlc/templates/state.yaml")
    for key, value in block(text or "", "sets"):
        if key == "status":
            return value.split()
    return []


def module_descriptions(root):
    """`src/<name>.rs` — what CLAUDE.md's module map says it is."""
    text, _ = read_text(root / "CLAUDE.md")
    found = {}
    lines = (text or "").splitlines()
    for index, line in enumerate(lines):
        match = re.match(r"^\s*- `src/([\w]+)(?:\.rs|/)`[^—]*— (.+)$", line)
        if not match or match.group(1) in found:
            continue
        # The entry runs on over indented lines; its first sentence is enough.
        said = match.group(2)
        for more in lines[index + 1:index + 6]:
            if not more.startswith("    ") or more.lstrip().startswith("- "):
                break
            said += " " + more.strip()
        said = re.sub(r"`", "", said).strip()
        found[match.group(1)] = re.split(r"(?<=\.)\s", said, maxsplit=1)[0]
    return found


def section(text, heading):
    """The lines under `## heading`, up to the next `## `."""
    lines, inside = [], False
    for line in text.splitlines():
        if line.startswith("## "):
            inside = line[3:].strip() == heading
            continue
        if inside:
            lines.append(line)
    return lines


def looks_like_path(token):
    return bool(re.match(r"^[\w.][\w./-]*$", token)) and (
        "/" in token or re.search(r"\.[a-z]{1,5}$", token) is not None)


def area_of(path, change):
    # The test file, the change's own records, and the sweep runner every
    # mutations.py names say nothing about what a change is about.
    if (path in ("src/tests.rs", "scripts/mutation_sweep.py")
            or path.startswith("docs/sdlc/changes/" + change)):
        return None
    module = re.match(r"^src/(\w+?)(?:\.rs|/|$)", path)
    if module:
        return module.group(1)
    if path.startswith(PIPELINE_PREFIXES) or path in PIPELINE_FILES:
        return "pipeline"
    if path in ("Cargo.toml", "Cargo.lock"):
        return "dependencies"
    return "docs"


def areas_of(change, docs):
    """Every area the change's plan names, most-named first; the intent's
    guess when there is no plan; `unfiled` when neither says."""
    for name, heading in (("plan.md", "Files that change"),
                          ("intent.md", "Systems likely affected")):
        lines = section(docs.get(name, ""), heading)
        tokens = []
        for line in lines:
            tokens.extend(re.findall(r"`([^`]+)`", line))
            if line.startswith("|"):
                cells = line.split("|")
                if len(cells) > 1:
                    tokens.extend(cells[1].replace("`", " ").replace(",", " ").split())
        counts = {}
        for token in tokens:
            token = token.strip().rstrip(".,")
            if not looks_like_path(token):
                continue
            area = area_of(token, change)
            if area:
                counts[area] = counts.get(area, 0) + 1
        if counts:
            return ranked(counts)
    # A bug fix has neither section: its report is in state.yaml and what it
    # touched is what its sweep mutates. Any repository path those name.
    counts = {}
    for name in ("state.yaml", "bug.md", "plan.md", "spec.md", "verification.yaml",
                 "mutations.py"):
        text = docs.get(name, "")
        if name == "state.yaml":
            # The report is the comment; the contract names every gate script.
            text = "\n".join(line for line in text.splitlines() if line.startswith("#"))
        for token in re.findall(REPOSITORY_PATH, text):
            # Every state file's header points at its template.
            if token.startswith("docs/sdlc/templates/"):
                continue
            area = area_of(token.rstrip(".,"), change)
            if area:
                counts[area] = counts.get(area, 0) + 1
    return ranked(counts) if counts else ["unfiled"]


def ranked(counts):
    order = list(counts)
    return sorted(order, key=lambda area: (-counts[area], order.index(area)))


def title_of(change, docs, state_text):
    for name, prefix in (("intent.md", "Intent:"), ("bug.md", ""), ("spec.md", "Spec:"),
                         ("plan.md", "Plan:")):
        for line in docs.get(name, "").splitlines():
            if line.startswith("# "):
                title = line[2:].strip()
                if prefix and title.startswith(prefix):
                    title = title[len(prefix):].strip()
                if title:
                    return title
                break
    for line in state_text.splitlines():
        match = re.match(r"^# Change \d+\.\s*(.+)$", line)
        if match:
            return match.group(1).strip()
    return change.split("-", 1)[-1].replace("-", " ")


def findings_of(text):
    """`carried:` items of review.yaml, each with its `person:` line."""
    found, inside = [], False
    for line in text.splitlines():
        if re.match(r"^[A-Za-z_][\w-]*:", line):
            inside = line.startswith("carried:")
            continue
        if not inside:
            continue
        if line.startswith("  - "):
            found.append({"finding": line[4:].strip(), "person": ""})
        elif found:
            person = re.match(r'^\s+person:\s*"(.*)"\s*$', line)
            if person:
                found[-1]["person"] = person.group(1)
    return found


def approvals_of(text):
    rows = []
    for line in text.splitlines():
        parts = line.split(" ", 6)
        if len(parts) >= 6:
            rows.append({"at": parts[0], "by": parts[1], "category": parts[2],
                         "artifact": parts[3], "verdict": parts[5],
                         "reason": parts[6] if len(parts) > 6 else ""})
    return rows


def read_change(directory, where, statuses):
    name = directory.name
    names = sorted(
        (entry.name for entry in directory.iterdir()
         if entry.is_file() and entry.name.endswith(DOC_SUFFIXES)),
        key=lambda doc: (DOC_ORDER.index(doc) if doc in DOC_ORDER else len(DOC_ORDER), doc))
    carried, omitted = names[:DOCS_PER_CHANGE], names[DOCS_PER_CHANGE:]
    docs, texts = [], {}
    for doc in carried:
        text, cut = read_text(directory / doc)
        if text is None:
            omitted.append(doc)
            continue
        docs.append({"name": doc, "text": text, "truncated": cut})
        texts[doc] = text
    sweep, _ = read_text(directory / "mutations.py")
    areas = areas_of(name, dict(texts, **{"mutations.py": sweep or ""}))
    state_text = texts.get("state.yaml", "")
    state = dict(block(state_text, "state"))
    resume = re.search(r'^resume:\s*"(.*)"\s*$', state_text, re.M)
    status = state.get("status", "")
    problem = None
    if "state.yaml" not in texts:
        problem = "state.yaml がありません"
    elif not status:
        problem = "state.yaml に status がありません"
    elif status not in statuses or status not in GROUPS:
        problem = "status「%s」は定義されていません" % status
    contract = []
    for key, value in block(state_text, "contract"):
        parts = value.split(" ", 2)
        contract.append({"key": key, "kind": parts[0],
                         "verdict": parts[1] if len(parts) > 1 else "",
                         "text": parts[2] if len(parts) > 2 else ""})
    number = re.match(r"^(\d+)", name)
    return {
        "name": name,
        "number": number.group(1) if number else "",
        "title": title_of(name, texts, state_text),
        "route": state.get("route", ""),
        "stage": state.get("stage", ""),
        "status": status,
        "group": "needs" if problem else GROUPS[status],
        "problem": problem,
        "attempts": state.get("attempts", ""),
        "questions": state.get("questions", ""),
        "autonomy": state.get("autonomy", ""),
        "risk": state.get("risk", ""),
        "readiness": state.get("readiness", ""),
        "screen": state.get("screen", ""),
        "resume": resume.group(1) if resume else "",
        "contract": contract,
        "where": where,
        "areas": areas,
        "findings": findings_of(texts.get("review.yaml", "")),
        "approvals": approvals_of(texts.get("approvals.log", "")),
        "omitted": omitted,
        "docs": docs,
    }


def other_worktrees(root):
    """(path, branch) of every other working copy, those on a branch first; or
    None when git cannot say. GIT_* is dropped so a session's exported
    repository is not the one that answers."""
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "worktree", "list", "--porcelain"],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=environment,
            timeout=GIT_SECONDS)
    except (OSError, subprocess.SubprocessError):
        return None
    if result.returncode != 0 or len(result.stdout) > GIT_BYTES:
        return None
    here = os.path.realpath(root)
    trees, path, branch = [], None, ""
    for line in result.stdout.decode("utf-8", errors="replace").splitlines() + [""]:
        if line.startswith("worktree "):
            path, branch = line[len("worktree "):], ""
        elif line.startswith("branch "):
            branch = line[len("branch "):].replace("refs/heads/", "", 1)
        elif not line and path:
            if os.path.realpath(path) != here:
                trees.append((path, branch))
            path = None
    trees.sort(key=lambda tree: tree[1] == "")
    return trees[:WORKTREES]


def change_directories(root):
    base = Path(root) / "docs/sdlc/changes"
    try:
        return sorted(entry for entry in base.iterdir() if entry.is_dir())
    except OSError:
        return []


def build(root):
    statuses = template_statuses(root) or list(GROUPS)
    changes = {}
    for directory in change_directories(root):
        changes[directory.name] = read_change(
            directory, {"here": True, "path": str(root), "branch": ""}, statuses)
    trees = other_worktrees(root)
    for path, branch in trees or []:
        for directory in change_directories(path):
            if directory.name not in changes:
                changes[directory.name] = read_change(
                    directory, {"here": False, "path": path, "branch": branch}, statuses)
    ordered = sorted(
        changes.values(),
        key=lambda change: (GROUP_ORDER.index(change["group"]),
                            change["name"] if change["group"] == "needs" else ""))
    # Newest first inside the groups a person scans as history.
    ordered = ([c for c in ordered if c["group"] == "needs"]
               + sorted((c for c in ordered if c["group"] == "flight"),
                        key=lambda c: c["name"], reverse=True)
               + sorted((c for c in ordered if c["group"] in ("done", "archived")),
                        key=lambda c: c["name"], reverse=True))
    counts = {group: sum(1 for c in ordered if c["group"] == group) for group in GROUP_ORDER}
    counts["total"] = len(ordered)
    descriptions = module_descriptions(root)
    filed = {}
    for change in ordered:
        for area in change["areas"]:
            filed.setdefault(area, []).append(change["name"])
    last = ("docs", "dependencies", "unfiled")
    areas = [
        {"name": area, "label": AREA_LABELS.get(area, area),
         "description": descriptions.get(area, ""), "changes": names}
        for area, names in sorted(
            filed.items(), key=lambda item: (item[0] in last, -len(item[1]), item[0]))
    ]
    return {
        "generated": datetime.datetime.now().strftime("%Y-%m-%d %H:%M"),
        "root": str(root),
        "worktrees_read": trees is not None,
        "statuses": {status: GROUPS[status] for status in statuses if status in GROUPS},
        "stages": list(STAGES),
        "routes": route_stages(root),
        "counts": counts,
        "areas": areas,
        "changes": ordered,
    }


def headline(model):
    counts = model["counts"]
    return "変更 %d 件 — 要対応 %d · 進行中 %d · 完了 %d · 保管 %d" % (
        counts["total"], counts["needs"], counts["flight"], counts["done"],
        counts["archived"])


def line_of(change):
    place = change["where"]["branch"] or ("" if change["where"]["here"] else change["where"]["path"])
    return "  %s  %s  %s  %s" % (
        change["name"], change["status"] or "?", change["stage"] or "?", place)


def summary(model, out):
    lines = [headline(model)]
    if not model["worktrees_read"]:
        lines.append("他の作業コピーを読めませんでした（このチェックアウトの記録だけです）")
    for group, label in (("needs", "要対応"), ("flight", "進行中")):
        lines.append(label)
        listed = [c for c in model["changes"] if c["group"] == group]
        if not listed:
            lines.append("  なし")
        for change in listed:
            lines.append(line_of(change).rstrip())
            if change["problem"]:
                lines.append("    記録を読めません: " + change["problem"])
            elif change["resume"]:
                lines.append("    次にすること: " + change["resume"])
    lines.append("地図: %s" % out)
    return "\n".join(lines) + "\n"


def find(model, word):
    word = word.lower()
    lines = []
    for change in model["changes"]:
        labels = [AREA_LABELS.get(area, area) for area in change["areas"]]
        if (word in change["name"].lower() or word in change["title"].lower()
                or word in (area.lower() for area in change["areas"])
                or word in labels):
            lines.append("%s  %s  %s  %s  — %s" % (
                change["name"], change["status"] or "?", change["stage"] or "?",
                ",".join(labels), change["title"]))
    return "\n".join(lines) + "\n" if lines else "該当する変更はありません\n"


def data_block(model):
    """JSON that no document's text can end: every character HTML or a script
    parser gives meaning to is written as an escape."""
    text = json.dumps(model, ensure_ascii=False, separators=(",", ":"))
    for raw, escaped in (("&", "\\u0026"), ("<", "\\u003c"), (">", "\\u003e"),
                         ("\u2028", "\\u2028"), ("\u2029", "\\u2029")):
        text = text.replace(raw, escaped)
    return text


def write_page(model, out):
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    page = PAGE.replace("__HEADLINE__", headline(model)).replace("__DATA__", data_block(model))
    # Beside the target and renamed, so an interrupted run leaves the old page.
    handle, temporary = tempfile.mkstemp(dir=str(out.parent), prefix=".change-map-")
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as stream:
            stream.write(page)
        os.replace(temporary, out)
    except BaseException:
        try:
            os.unlink(temporary)
        except OSError:
            pass
        raise


def main():
    parser = argparse.ArgumentParser(description="Every change at a glance.")
    parser.add_argument("--root", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--out")
    parser.add_argument("--find", metavar="WORD")
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args()
    root = Path(arguments.root).resolve()
    if not (root / "docs/sdlc/changes").is_dir():
        sys.stderr.write("docs/sdlc/changes がありません: %s\n" % root)
        return 2
    model = build(root)
    if arguments.find is not None:
        sys.stdout.write(find(model, arguments.find))
        return 0
    if arguments.json:
        for change in model["changes"]:
            change["records"] = [doc["name"] for doc in change.pop("docs")]
        json.dump(model, sys.stdout, ensure_ascii=False)
        sys.stdout.write("\n")
        return 0
    out = Path(arguments.out) if arguments.out else root / ".tmp/change-map.html"
    write_page(model, out)
    sys.stdout.write(summary(model, out))
    return 0


PAGE = r"""<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Operon 変更地図</title>
<style>
:root {
  --bg: #f6f5f1; --panel: #ffffff; --ink: #1d1c1a; --soft: #5c5a55; --line: #dcd9d1;
  --needs: #a8320a; --needs-bg: #fdeee6; --flight: #1a55a8; --flight-bg: #e8f0fb;
  --done: #27693a; --done-bg: #e7f3ea; --rest: #5c5a55; --rest-bg: #ecebe6;
  --code: #f0eee8; --accent: #1a55a8; --shadow: 0 12px 40px rgba(0,0,0,.18);
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #151514; --panel: #1f1f1d; --ink: #ecebe6; --soft: #a9a69e; --line: #37362f;
    --needs: #ffa57e; --needs-bg: #3a1e12; --flight: #8fbcff; --flight-bg: #14263f;
    --done: #86d39b; --done-bg: #14301c; --rest: #a9a69e; --rest-bg: #2a2925;
    --code: #2a2925; --accent: #8fbcff; --shadow: 0 12px 40px rgba(0,0,0,.6);
  }
}
* { box-sizing: border-box; }
html { font-size: 15px; }
body { margin: 0; background: var(--bg); color: var(--ink); line-height: 1.6;
  font-family: -apple-system, BlinkMacSystemFont, "Hiragino Sans", "Yu Gothic UI", "Noto Sans JP", sans-serif; }
button { font: inherit; color: inherit; cursor: pointer; }
header.top { position: sticky; top: 0; z-index: 5; background: var(--bg);
  border-bottom: 1px solid var(--line); padding: 14px 28px 12px; }
header.top h1 { font-size: 1.15rem; margin: 0; display: inline; letter-spacing: .02em; }
header.top .counts { margin-left: 14px; color: var(--soft); }
header.top .row { display: flex; gap: 16px; align-items: center; margin-top: 10px; flex-wrap: wrap; }
header.top input { flex: 1 1 320px; max-width: 520px; padding: 7px 12px; border-radius: 8px;
  border: 1px solid var(--line); background: var(--panel); color: var(--ink); font: inherit; }
header.top .meta { color: var(--soft); font-size: .86rem; margin-left: auto; }
.warn { color: var(--needs); }
main { padding: 8px 28px 80px; max-width: 1500px; margin: 0 auto; }
section { margin-top: 26px; }
section > h2 { font-size: 1rem; margin: 0 0 10px; display: flex; gap: 10px; align-items: baseline; }
section > h2 small { color: var(--soft); font-weight: 400; }
.empty { color: var(--soft); padding: 10px 2px; }
.pill { display: inline-block; padding: 1px 9px; border-radius: 999px; font-size: .8rem;
  white-space: nowrap; background: var(--rest-bg); color: var(--rest); }
.pill.needs { background: var(--needs-bg); color: var(--needs); }
.pill.flight { background: var(--flight-bg); color: var(--flight); }
.pill.done { background: var(--done-bg); color: var(--done); }
.cards { display: grid; gap: 10px; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); }
.card { text-align: left; background: var(--panel); border: 1px solid var(--line);
  border-left: 4px solid var(--rest); border-radius: 10px; padding: 12px 14px; width: 100%; }
.card.needs { border-left-color: var(--needs); }
.card.flight { border-left-color: var(--flight); }
.card:hover, .chip:hover, tr.pick:hover { outline: 2px solid var(--accent); outline-offset: 1px; }
.card .head { display: flex; gap: 8px; align-items: baseline; flex-wrap: wrap; }
.card .num { font-weight: 700; font-variant-numeric: tabular-nums; }
.card .title { font-weight: 600; flex: 1 1 200px; }
.card .sub { color: var(--soft); font-size: .86rem; margin-top: 2px; overflow-wrap: anywhere; }
.card .next { margin-top: 8px; font-size: .92rem; }
.card .next b { color: var(--soft); font-weight: 600; }
.track { display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 8px; }
.track .col { background: var(--panel); border: 1px solid var(--line); border-radius: 10px;
  padding: 8px; min-height: 84px; }
.track .col h3 { margin: 0 0 8px; font-size: .84rem; color: var(--soft); font-weight: 600; }
.track .card { padding: 8px 10px; margin-bottom: 6px; }
.area { display: grid; grid-template-columns: 250px 190px 1fr; gap: 14px; align-items: start;
  padding: 9px 0; border-top: 1px solid var(--line); }
.area .name { font-weight: 600; }
.area .desc { color: var(--soft); font-size: .82rem; line-height: 1.4; }
.bar { display: flex; align-items: center; gap: 8px; color: var(--soft); font-size: .86rem;
  font-variant-numeric: tabular-nums; }
.bar i { display: block; height: 8px; border-radius: 4px; background: var(--done); min-width: 3px; }
.chips { display: flex; flex-wrap: wrap; gap: 5px; }
.chip { border: 1px solid var(--line); background: var(--panel); border-radius: 6px;
  padding: 0 7px; font-size: .84rem; font-variant-numeric: tabular-nums; }
.chip.needs { background: var(--needs-bg); color: var(--needs); border-color: var(--needs); }
.chip.flight { background: var(--flight-bg); color: var(--flight); border-color: var(--flight); }
.chip.archived { color: var(--soft); text-decoration: line-through; }
.chip.more { border-style: dashed; color: var(--soft); }
.filters { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 8px; }
.filters button { border: 1px solid var(--line); background: var(--panel); border-radius: 999px;
  padding: 2px 12px; font-size: .86rem; }
.filters button[aria-pressed="true"] { background: var(--ink); color: var(--bg); border-color: var(--ink); }
table.all { width: 100%; border-collapse: collapse; background: var(--panel);
  border: 1px solid var(--line); border-radius: 10px; overflow: hidden; }
table.all td, table.all th { padding: 6px 10px; border-top: 1px solid var(--line);
  text-align: left; vertical-align: top; font-size: .9rem; }
table.all th { border-top: 0; color: var(--soft); font-weight: 600; font-size: .82rem; }
table.all td.num { font-weight: 700; font-variant-numeric: tabular-nums; white-space: nowrap; }
tr.pick { cursor: pointer; }
#shade { position: fixed; inset: 0; background: rgba(0,0,0,.35); z-index: 9; }
#detail { position: fixed; top: 0; right: 0; bottom: 0; width: min(980px, 100%); z-index: 10;
  background: var(--panel); box-shadow: var(--shadow); display: flex; flex-direction: column; }
#detail[hidden], #shade[hidden] { display: none; }
#detail .dhead { padding: 16px 22px 0; border-bottom: 1px solid var(--line); }
#detail .dtitle { display: flex; gap: 10px; align-items: flex-start; }
#detail h2 { margin: 0; font-size: 1.12rem; flex: 1; line-height: 1.4; }
#detail .nav button { border: 1px solid var(--line); background: var(--bg); border-radius: 7px;
  padding: 3px 11px; margin-left: 4px; white-space: nowrap; }
#detail .facts { color: var(--soft); font-size: .88rem; margin: 6px 0 8px; display: flex;
  gap: 8px; flex-wrap: wrap; align-items: center; }
.steps { display: flex; gap: 0; margin: 4px 0 8px; flex-wrap: wrap; align-items: center;
  font-size: .86rem; color: var(--soft); }
.steps span { padding: 2px 10px; border: 1px solid var(--line); border-radius: 999px; }
.steps span.past { background: var(--done-bg); color: var(--done); border-color: transparent; }
.steps span.now { background: var(--ink); color: var(--bg); border-color: var(--ink); font-weight: 700; }
.steps em { width: 14px; height: 1px; background: var(--line); }
.tabs { display: flex; gap: 2px; overflow-x: auto; margin-top: 6px; }
.tabs button { border: 0; background: none; padding: 7px 12px; border-bottom: 3px solid transparent;
  white-space: nowrap; color: var(--soft); }
.tabs button[aria-selected="true"] { color: var(--ink); border-bottom-color: var(--accent); font-weight: 600; }
#body { padding: 18px 26px 60px; overflow: auto; flex: 1; }
.tools { display: flex; justify-content: flex-end; gap: 8px; margin-bottom: 4px; color: var(--soft);
  font-size: .84rem; align-items: center; }
.tools button { border: 1px solid var(--line); background: var(--bg); border-radius: 7px; padding: 1px 10px; }
.md { max-width: 820px; overflow-wrap: anywhere; }
.md h1 { font-size: 1.3rem; margin: 0 0 12px; line-height: 1.4; }
.md h2 { font-size: 1.08rem; margin: 26px 0 8px; padding-bottom: 4px; border-bottom: 1px solid var(--line); }
.md h3, .md h4 { font-size: 1rem; margin: 18px 0 6px; }
.md p { margin: 8px 0; }
.md ul, .md ol { margin: 8px 0; padding-left: 24px; }
.md li { margin: 3px 0; }
.md code { background: var(--code); border-radius: 4px; padding: 1px 5px; font-size: .88em;
  font-family: ui-monospace, "SF Mono", Menlo, monospace; }
.md pre, pre.raw { background: var(--code); border-radius: 8px; padding: 12px 14px; overflow: auto;
  font-size: .84rem; line-height: 1.45; font-family: ui-monospace, "SF Mono", Menlo, "Osaka-Mono", monospace; }
.md pre code { background: none; padding: 0; font-size: inherit; }
pre.raw { white-space: pre-wrap; overflow-wrap: anywhere; }
.md table { border-collapse: collapse; margin: 10px 0; font-size: .9rem; display: block; overflow-x: auto; }
.md th, .md td { border: 1px solid var(--line); padding: 5px 9px; text-align: left; vertical-align: top; }
.md th { background: var(--bg); }
.md blockquote { margin: 10px 0; padding: 4px 14px; border-left: 3px solid var(--line); color: var(--soft); }
.md a { color: var(--accent); }
.kv { display: grid; grid-template-columns: 150px 1fr; gap: 6px 14px; max-width: 820px; }
.kv dt { color: var(--soft); } .kv dd { margin: 0; overflow-wrap: anywhere; }
.box { border: 1px solid var(--line); border-radius: 10px; padding: 12px 16px; margin: 12px 0; max-width: 820px; }
.box.needs { border-color: var(--needs); background: var(--needs-bg); }
.box h3 { margin: 0 0 6px; font-size: .95rem; }
ul.check { list-style: none; padding: 0; margin: 0; }
ul.check li { display: grid; grid-template-columns: 54px 1fr; gap: 10px; padding: 6px 0;
  border-top: 1px solid var(--line); }
ul.check li:first-child { border-top: 0; }
ul.check .key { font-weight: 600; } ul.check .what { color: var(--soft); font-size: .88rem; }
.finding { border-top: 1px solid var(--line); padding: 12px 0; max-width: 820px; }
.finding .person { font-size: 1rem; } .finding .raw { color: var(--soft); font-size: .84rem; margin-top: 4px; }
@media (max-width: 900px) {
  .area { grid-template-columns: 1fr; gap: 4px; } .track { grid-template-columns: repeat(2, 1fr); }
  main, header.top { padding-left: 14px; padding-right: 14px; }
}
</style>
</head>
<body>
<header class="top">
  <h1>Operon 変更地図</h1><span class="counts">__HEADLINE__</span>
  <div class="row">
    <input id="q" type="search" placeholder="検索: 番号・題・領域・状態" aria-label="変更を検索">
    <span class="meta" id="meta"></span>
  </div>
</header>
<main>
  <section id="needs"><h2>あなたの判断を待っています <small id="needs-n"></small></h2><div id="needs-list"></div></section>
  <section id="flight"><h2>進行中 <small id="flight-n"></small></h2><div id="flight-list"></div></section>
  <section id="areas"><h2>領域ごとの変更 <small>どの部分に、これまで何をしてきたか</small></h2><div id="area-list"></div></section>
  <section id="all"><h2>すべての変更 <small id="all-n"></small></h2>
    <div class="filters" id="filters"></div><div id="all-list"></div></section>
</main>
<div id="shade" hidden></div>
<aside id="detail" hidden aria-label="変更の詳細">
  <div class="dhead">
    <div class="dtitle"><h2 id="d-title"></h2>
      <div class="nav"><button id="d-prev">前へ</button><button id="d-next">次へ</button><button id="d-close">閉じる</button></div></div>
    <div class="facts" id="d-facts"></div>
    <div class="steps" id="d-steps"></div>
    <div class="tabs" id="d-tabs" role="tablist"></div>
  </div>
  <div id="body"></div>
</aside>
<script id="map-data" type="application/json">__DATA__</script>
<script>
"use strict";
const M = JSON.parse(document.getElementById("map-data").textContent);
const STATUS = { "planning": "計画中", "ready": "着手可", "in-progress": "実装中", "reviewing": "検証中",
  "done": "完了", "awaiting-user": "判断待ち", "blocked": "停止中", "failed": "失敗", "archived": "保管" };
const STAGE = { plan: "計画", design: "設計", build: "実装", test: "テスト", deploy: "出荷", maintain: "保守" };
const GROUP = { needs: "要対応", flight: "進行中", done: "完了", archived: "保管" };
const TAB = { "intent.md": "意図", "spec.md": "仕様", "plan.md": "計画", "screen.md": "画面案",
  "bug.md": "不具合の報告", "handoff.md": "引き継ぎ", "state.yaml": "状態の記録", "review.yaml": "レビューの記録",
  "verification.yaml": "検証の記録" };
const VERDICT = { passed: "済", pending: "未", waived: "免除" };
const byName = new Map(M.changes.map(c => [c.name, c]));
const $ = id => document.getElementById(id);

function esc(text) {
  return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}
function statusWord(c) { return c.problem ? "読めない記録" : (STATUS[c.status] || c.status); }
function pill(c) { return '<span class="pill ' + c.group + '">' + esc(statusWord(c)) + "</span>"; }
function place(c) { return c.where.branch || (c.where.here ? "" : c.where.path); }
function areaLabel(name) { const a = M.areas.find(x => x.name === name); return a ? a.label : name; }
function haystack(c) {
  return [c.name, c.title, statusWord(c), c.status, c.route, STAGE[c.stage] || "", c.stage,
    c.areas.map(areaLabel).join(" "), c.areas.join(" "), place(c)].join(" ").toLowerCase();
}

function card(c, small) {
  let h = '<button class="card ' + c.group + '" data-open="' + esc(c.name) + '"><div class="head">' +
    '<span class="num">' + esc(c.number) + '</span><span class="title">' + esc(c.title) + "</span>" + pill(c) + "</div>";
  const sub = [STAGE[c.stage] || c.stage, c.route, place(c)].filter(Boolean).join(" · ");
  if (!small) h += '<div class="sub">' + esc(sub) + "</div>";
  else if (place(c)) h += '<div class="sub">' + esc(place(c)) + "</div>";
  if (c.problem) h += '<div class="next"><b>記録を読めません: </b>' + esc(c.problem) + "</div>";
  else if (c.resume && !small) h += '<div class="next"><b>次にすること: </b>' + esc(c.resume) + "</div>";
  return h + "</button>";
}

let query = "";
let groupFilter = "";
let expanded = new Set();
let listed = [];

function matches(c) { return !query || haystack(c).includes(query); }

function renderOverview() {
  const needs = M.changes.filter(c => c.group === "needs");
  $("needs-n").textContent = "(" + needs.length + ")";
  $("needs-list").innerHTML = needs.length
    ? '<div class="cards">' + needs.map(c => card(c)).join("") + "</div>"
    : '<div class="empty">判断を待っている変更はありません</div>';
  const flight = M.changes.filter(c => c.group === "flight");
  $("flight-n").textContent = "(" + flight.length + ")";
  if (!flight.length) $("flight-list").innerHTML = '<div class="empty">進行中の変更はありません</div>';
  else $("flight-list").innerHTML = '<div class="track">' + M.stages.map(s =>
    '<div class="col"><h3>' + STAGE[s] + "</h3>" +
    flight.filter(c => c.stage === s).map(c => card(c, true)).join("") + "</div>").join("") + "</div>" +
    flight.filter(c => !M.stages.includes(c.stage)).map(c => card(c)).join("");
  const most = Math.max(1, ...M.areas.map(a => a.changes.length));
  $("area-list").innerHTML = M.areas.map(a => {
    const all = a.changes.map(n => byName.get(n)).filter(matches);
    if (query && !all.length) return "";
    const open = expanded.has(a.name) || query;
    const shown = open ? all : all.slice(0, 16);
    let chips = shown.map(c => '<button class="chip ' + c.group + '" data-open="' + esc(c.name) +
      '" title="' + esc(c.title + " — " + statusWord(c)) + '">' + esc(c.number) + "</button>").join("");
    if (all.length > shown.length) chips += '<button class="chip more" data-more="' + esc(a.name) +
      '">さらに ' + (all.length - shown.length) + "</button>";
    return '<div class="area"><div><div class="name">' + esc(a.label) + '</div><div class="desc">' +
      esc(a.description) + '</div></div><div class="bar"><i style="width:' +
      Math.round(150 * a.changes.length / most) + 'px"></i>' + a.changes.length +
      '</div><div class="chips">' + chips + "</div></div>";
  }).join("") || '<div class="empty">該当する変更はありません</div>';
  $("filters").innerHTML = [["", "すべて"]].concat(Object.entries(GROUP)).map(([key, label]) =>
    '<button data-group="' + key + '" aria-pressed="' + (groupFilter === key) + '">' + label +
    (key ? " " + M.counts[key] : "") + "</button>").join("");
  listed = M.changes.filter(c => matches(c) && (!groupFilter || c.group === groupFilter));
  $("all-n").textContent = "(" + listed.length + " / " + M.counts.total + ")";
  $("all-list").innerHTML = listed.length ? '<table class="all"><thead><tr><th>番号</th><th>題</th><th>状態</th>' +
    "<th>段階</th><th>経路</th><th>領域</th></tr></thead><tbody>" + listed.map(c =>
      '<tr class="pick" data-open="' + esc(c.name) + '"><td class="num">' + esc(c.number) + "</td><td>" +
      esc(c.title) + "</td><td>" + pill(c) + "</td><td>" + esc(STAGE[c.stage] || c.stage) + "</td><td>" +
      esc(c.route) + "</td><td>" + esc(c.areas.map(areaLabel).join("、")) + "</td></tr>").join("") +
    "</tbody></table>" : '<div class="empty">該当する変更はありません</div>';
}

// Markdown, escape first: every character of a document passes through esc()
// before any markup is added around it.
function inline(text) {
  const codes = [];
  let s = text.replace(/`([^`]+)`/g, (_, code) => { codes.push(code); return "\u0000" + (codes.length - 1) + "\u0000"; });
  s = esc(s);
  s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/(^|[\s(])\*([^*\s][^*]*)\*(?=[\s).,;:!?]|$)/g, "$1<em>$2</em>");
  s = s.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>');
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => "<code>" + esc(codes[Number(i)]) + "</code>");
}
function cells(line) {
  let s = line.trim();
  if (s.startsWith("|")) s = s.slice(1);
  if (s.endsWith("|")) s = s.slice(0, -1);
  const out = []; let cur = ""; let tick = false;
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === "`") tick = !tick;
    if (ch === "\\" && s[i + 1] === "|") { cur += "|"; i++; continue; }
    if (ch === "|" && !tick) { out.push(cur.trim()); cur = ""; continue; }
    cur += ch;
  }
  out.push(cur.trim());
  return out;
}
function markdown(text) {
  const lines = text.split("\n");
  const out = [];
  let i = 0;
  const listItem = /^(\s*)([-*]|\d+[.)])\s+(.*)$/;
  while (i < lines.length) {
    const line = lines[i];
    const fence = line.match(/^\s*(```|~~~)/);
    if (fence) {
      const buf = []; i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence[1])) { buf.push(lines[i]); i++; }
      i++;
      out.push("<pre><code>" + esc(buf.join("\n")) + "</code></pre>");
      continue;
    }
    if (!line.trim()) { i++; continue; }
    const head = line.match(/^(#{1,6})\s+(.*)$/);
    if (head) { const n = head[1].length; out.push("<h" + n + ">" + inline(head[2]) + "</h" + n + ">"); i++; continue; }
    if (/^\s*([-*_])\s*(\1\s*){2,}$/.test(line)) { out.push("<hr>"); i++; continue; }
    if (line.trim().startsWith("|") && i + 1 < lines.length && /^\s*\|?\s*:?-{2,}/.test(lines[i + 1])) {
      const header = cells(line); i += 2;
      let t = "<table><thead><tr>" + header.map(c => "<th>" + inline(c) + "</th>").join("") + "</tr></thead><tbody>";
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        t += "<tr>" + cells(lines[i]).map(c => "<td>" + inline(c) + "</td>").join("") + "</tr>"; i++;
      }
      out.push(t + "</tbody></table>");
      continue;
    }
    if (line.startsWith(">")) {
      const buf = [];
      while (i < lines.length && lines[i].startsWith(">")) { buf.push(lines[i].replace(/^>\s?/, "")); i++; }
      out.push("<blockquote>" + markdown(buf.join("\n")) + "</blockquote>");
      continue;
    }
    if (listItem.test(line)) {
      const base = line.match(listItem)[1].length;
      const ordered = /\d/.test(line.match(listItem)[2]);
      let h = ordered ? "<ol>" : "<ul>";
      while (i < lines.length) {
        const m = lines[i].match(listItem);
        if (!m || m[1].length < base) break;
        if (m[1].length > base) { h += markdownNested(); continue; }
        let item = m[3]; i++;
        while (i < lines.length && lines[i].trim() && !listItem.test(lines[i]) &&
               /^\s+/.test(lines[i]) && !/^\s*(```|~~~)/.test(lines[i])) { item += " " + lines[i].trim(); i++; }
        h += "<li>" + inline(item);
        if (i < lines.length) {
          const next = lines[i].match(listItem);
          if (next && next[1].length > base) h += markdownNested();
        }
        h += "</li>";
        while (i < lines.length && !lines[i].trim() && i + 1 < lines.length && listItem.test(lines[i + 1]) &&
               lines[i + 1].match(listItem)[1].length >= base) i++;
      }
      out.push(h + (ordered ? "</ol>" : "</ul>"));
      continue;
    }
    const para = [line.trim()]; i++;
    while (i < lines.length && lines[i].trim() && !/^(#{1,6}\s|>|\s*(```|~~~)|\s*\|)/.test(lines[i]) &&
           !listItem.test(lines[i])) { para.push(lines[i].trim()); i++; }
    out.push("<p>" + inline(para.join(" ")) + "</p>");
  }
  return out.join("\n");

  function markdownNested() {
    const depth = lines[i].match(listItem)[1].length;
    const buf = [];
    while (i < lines.length && (!lines[i].trim() ? false :
           (lines[i].match(/^(\s*)/)[1].length >= depth))) { buf.push(lines[i].slice(depth)); i++; }
    return markdown(buf.join("\n"));
  }
}

let current = null;
let currentTab = "overview";
let rawView = false;

function tabsOf(c) {
  const tabs = [["overview", "概要"]];
  const named = ["intent.md", "bug.md", "spec.md", "plan.md", "screen.md", "handoff.md"];
  for (const n of named) if (c.docs.some(d => d.name === n)) tabs.push([n.replace(/\.md$/, ""), TAB[n]]);
  if (c.findings.length) tabs.push(["findings", "未解決の指摘 " + c.findings.length]);
  if (c.approvals.length) tabs.push(["approvals", "承認の記録"]);
  for (const d of c.docs) if (!named.includes(d.name) && d.name !== "approvals.log")
    tabs.push([d.name.replace(/\.md$/, ""), TAB[d.name] || d.name]);
  return tabs;
}
function docFor(c, tab) { return c.docs.find(d => d.name === tab || d.name === tab + ".md"); }

function overview(c) {
  let h = "";
  if (c.problem) h += '<div class="box needs"><h3>記録を読めません</h3>' + esc(c.problem) + "</div>";
  else if (c.group === "needs") h += '<div class="box needs"><h3>あなたの判断を待っています</h3>' + esc(c.resume) + "</div>";
  else if (c.resume) h += '<div class="box"><h3>' + (c.group === "flight" ? "次にすること" : "結果") + "</h3>" + esc(c.resume) + "</div>";
  const used = v => { const p = String(v).split(" "); return p.length === 2 ? p[0] + " / " + p[1] : v; };
  h += '<dl class="kv"><dt>状態</dt><dd>' + pill(c) + " <code>" + esc(c.status) + "</code></dd>" +
    "<dt>段階</dt><dd>" + esc(STAGE[c.stage] || c.stage) + " <code>" + esc(c.stage) + "</code></dd>" +
    "<dt>経路</dt><dd><code>" + esc(c.route) + "</code></dd>" +
    "<dt>リスク / 自律</dt><dd>" + esc(c.risk) + " / " + esc(c.autonomy) + "</dd>" +
    "<dt>やり直し</dt><dd>" + esc(used(c.attempts)) + "</dd><dt>質問</dt><dd>" + esc(used(c.questions)) + "</dd>" +
    "<dt>仕様の準備</dt><dd>" + esc(c.readiness) + "</dd><dt>画面の合意</dt><dd>" + esc(c.screen) + "</dd>" +
    "<dt>領域</dt><dd>" + esc(c.areas.map(areaLabel).join("、")) + "</dd>" +
    "<dt>場所</dt><dd>" + (c.where.here ? "このチェックアウト" : "別の作業コピー: " + esc(c.where.path)) +
    (c.where.branch ? " (" + esc(c.where.branch) + ")" : "") + "</dd>" +
    "<dt>記録</dt><dd><code>docs/sdlc/changes/" + esc(c.name) + "/</code></dd></dl>";
  if (c.contract.length) {
    h += '<div class="box"><h3>完了の条件</h3><ul class="check">' + c.contract.map(k =>
      "<li><span>" + '<span class="pill ' + (k.verdict === "pending" ? "needs" : k.verdict === "passed" ? "done" : "") + '">' +
      esc(VERDICT[k.verdict] || k.verdict) + '</span></span><span><span class="key">' + esc(k.key) +
      "</span> <small>(" + (k.kind === "human" ? "人が確認" : "自動") + ')</small><div class="what">' + esc(k.text) +
      "</div></span></li>").join("") + "</ul></div>";
  }
  if (c.omitted.length) h += '<p class="warn">載せていない記録: ' + esc(c.omitted.join(", ")) + "</p>";
  return h;
}

function renderDetail() {
  const c = current;
  $("d-title").textContent = c.number + "  " + c.title;
  $("d-facts").innerHTML = pill(c) + "<span>" + esc([c.route, c.risk ? "リスク " + c.risk : "", place(c)].filter(Boolean).join(" · ")) + "</span>";
  const route = (M.routes[c.route] || []).slice();
  if (c.stage && !route.includes(c.stage)) route.push(c.stage);
  const at = route.indexOf(c.stage);
  const closed = c.group === "done";
  $("d-steps").innerHTML = route.map((s, n) => '<span class="' + (closed || n < at ? "past" : n === at ? "now" : "") +
    '">' + STAGE[s] + "</span>").join("<em></em>");
  const tabs = tabsOf(c);
  if (!tabs.some(t => t[0] === currentTab)) currentTab = "overview";
  $("d-tabs").innerHTML = tabs.map(t => '<button role="tab" data-tab="' + esc(t[0]) + '" aria-selected="' +
    (t[0] === currentTab) + '">' + esc(t[1]) + "</button>").join("");
  const body = $("body");
  if (currentTab === "overview") body.innerHTML = overview(c);
  else if (currentTab === "findings") {
    body.innerHTML = "<p>レビューで見つかり、直さずに残してある点です。</p>" + c.findings.map(f =>
      '<div class="finding"><div class="person">' + (f.person ? esc(f.person) : "（人向けの説明がありません）") +
      '</div><div class="raw">' + esc(f.finding) + "</div></div>").join("");
  } else if (currentTab === "approvals") {
    body.innerHTML = '<div class="md"><table><thead><tr><th>日時 (UTC)</th><th>誰が</th><th>対象</th><th>判定</th><th>理由</th></tr></thead><tbody>' +
      c.approvals.map(a => "<tr><td>" + esc(a.at) + "</td><td>" + esc(a.by === "person" ? "人" : a.by === "auto" ? "評価役" : a.by) +
      "</td><td>" + esc(a.category + " " + a.artifact) + "</td><td>" + esc(a.verdict) + "</td><td>" + esc(a.reason) +
      "</td></tr>").join("") + "</tbody></table></div>";
  } else {
    const d = docFor(c, currentTab);
    const isMd = d.name.endsWith(".md");
    let h = '<div class="tools"><code>' + esc(d.name) + "</code>" + (isMd ? '<button id="raw-toggle">' +
      (rawView ? "整形して表示" : "原文") + "</button>" : "") + "</div>";
    if (d.truncated) h += '<p class="warn">長いため途中まで表示しています</p>';
    h += isMd && !rawView ? '<div class="md">' + markdown(d.text) + "</div>" : '<pre class="raw">' + esc(d.text) + "</pre>";
    body.innerHTML = h;
  }
  body.scrollTop = 0;
  const at2 = listed.indexOf(c);
  $("d-prev").disabled = at2 <= 0;
  $("d-next").disabled = at2 < 0 || at2 >= listed.length - 1;
}

function show(name, tab) {
  const c = byName.get(name);
  if (!c) return;
  current = c; currentTab = tab || "overview";
  $("detail").hidden = false; $("shade").hidden = false;
  renderDetail();
  const hash = "#" + (c.number || c.name) + (currentTab === "overview" ? "" : "/" + currentTab);
  if (location.hash !== hash) history.replaceState(null, "", hash);
  $("d-close").focus();
}
function hide() {
  current = null; $("detail").hidden = true; $("shade").hidden = true;
  history.replaceState(null, "", location.pathname + location.search);
}
function fromHash() {
  const m = decodeURIComponent(location.hash.slice(1)).match(/^([^/]+)(?:\/(.+))?$/);
  if (!m) return;
  const c = M.changes.find(x => x.number === m[1] || x.name === m[1]);
  if (c) show(c.name, m[2]);
}

document.addEventListener("click", event => {
  const target = event.target.closest("[data-open],[data-more],[data-group],[data-tab]");
  if (!target) return;
  if (target.dataset.open) show(target.dataset.open);
  else if (target.dataset.more) { expanded.add(target.dataset.more); renderOverview(); }
  else if (target.dataset.group !== undefined) { groupFilter = target.dataset.group; renderOverview(); }
  else if (target.dataset.tab) { show(current.name, target.dataset.tab); }
});
$("body").addEventListener("click", event => {
  if (event.target.id === "raw-toggle") { rawView = !rawView; renderDetail(); }
});
$("q").addEventListener("input", event => { query = event.target.value.trim().toLowerCase(); renderOverview(); });
$("shade").addEventListener("click", hide);
$("d-close").addEventListener("click", hide);
$("d-prev").addEventListener("click", () => { const n = listed.indexOf(current); if (n > 0) show(listed[n - 1].name, currentTab); });
$("d-next").addEventListener("click", () => { const n = listed.indexOf(current); if (n >= 0 && n < listed.length - 1) show(listed[n + 1].name, currentTab); });
document.addEventListener("keydown", event => { if (event.key === "Escape" && current) hide(); });
window.addEventListener("hashchange", fromHash);

$("meta").innerHTML = "生成 " + esc(M.generated) + (M.worktrees_read ? "" : ' · <span class="warn">他の作業コピーを読めませんでした</span>');
renderOverview();
fromHash();
</script>
</body>
</html>
"""


if __name__ == "__main__":
    sys.exit(main())
