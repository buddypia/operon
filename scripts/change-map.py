#!/usr/bin/env python3
"""Every change at a glance, and its documents on demand. Change 312.

    python3 scripts/change-map.py               summary, and .tmp/change-map.html
    python3 scripts/change-map.py --find WORD   the changes of an area, or with
                                                WORD in their name or title
    python3 scripts/change-map.py --json        the model, without documents
    python3 scripts/change-map.py --backlog     also read the backlog (GitHub,
                                                through scripts/backlog.py)
    python3 scripts/change-map.py --serve       the page on 127.0.0.1, rebuilt
                                                on every load (--port, --requests)

The map is generated from the records the pipeline already keeps — each
change's state.yaml, its documents, review.yaml, approvals.log — and from
nothing else. It writes nothing back, keeps no ledger, and its page is not
committed: a second record of a change's position is the thing
.claude/skills/feature-pilot/SKILL.md forbids, and a generated view of the
first record is not one.

A change under way lives in another worktree until it lands, so the other
working copies are read too: one `git worktree list`, ten seconds, 64 KiB.
Ceilings: 256 KiB per document, 40 records per change, 64 working copies.
The backlog is read only when asked: one child, 30 seconds, 1 MiB, 200 items.

exit 0 written or printed, 2 usage, no docs/sdlc/changes, or a port in use
"""
import argparse
import base64
import datetime
import errno
import http.server
import importlib.util
import json
import math
import os
import re
import select
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

DOC_BYTES = 256 * 1024
IMAGE_BYTES = 2 * 1024 * 1024
FEATURES = 64
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


def obligations_of(text):
    """How many obligations a change's verification.yaml lists."""
    count, inside = 0, False
    for line in text.splitlines():
        if re.match(r"^[A-Za-z_][\w-]*:", line):
            inside = line.startswith("obligations:")
            continue
        if inside and line.startswith("  - "):
            count += 1
    return count


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
        "features_named": state.get("features", ""),
        "plan_paths": plan_paths(name, texts),
        "resume": resume.group(1) if resume else "",
        "contract": contract,
        "where": where,
        "areas": areas,
        "findings": findings_of(texts.get("review.yaml", "")),
        "approvals": approvals_of(texts.get("approvals.log", "")),
        "obligations": obligations_of(texts.get("verification.yaml", "")),
        "omitted": omitted,
        "docs": docs,
    }


def parse_catalogue(text):
    """The catalogue's flat `- id:` entries and its screens, as
    docs/features/index.yaml describes itself; no YAML library."""
    entries, screens, inside = [], [], False
    for line in text.splitlines():
        if line.startswith("# Screens"):
            inside = True
            continue
        if inside:
            row = re.match(r"^#   (\S+)\s+(.*)$", line)
            if row:
                screens.append({"id": row.group(1), "label": row.group(2).strip()})
                continue
            inside = False
        if not line.strip() or line.startswith("#"):
            continue
        start = re.match(r"^  - id:\s*(\S+)\s*$", line)
        if start:
            entries.append({"id": start.group(1)})
            continue
        field = re.match(r"^    ([\w-]+):\s*(.*?)\s*$", line)
        if field and entries:
            entries[-1][field.group(1)] = field.group(2)
    return entries, screens


def manual_section(manual, heading):
    """The manual's text under the heading, to the next heading of its level
    or above."""
    lines, start, level = manual.splitlines(), None, 0
    for index, line in enumerate(lines):
        found = re.match(r"^(#+)\s+(.*?)\s*$", line)
        if found and found.group(2) == heading:
            start, level = index, len(found.group(1))
            break
    if start is None:
        return ""
    out = []
    for line in lines[start + 1:]:
        found = re.match(r"^(#+)\s", line)
        if found and len(found.group(1)) <= level:
            break
        out.append(line)
    return "\n".join(out).strip()


def keyword_hit(keyword, text):
    if re.search(r"[^\x00-\x7f]", keyword):
        return keyword in text
    # A whole word, or the word with an ending: `pr` is not the start of `problem`.
    return re.search(r"(?<![a-z0-9])" + re.escape(keyword.lower()) + r"(?:s|es|ed|ing)?(?![a-z0-9])", text) is not None


def read_features(root):
    """The feature layer: the catalogue, each feature's documents, the
    screens with their images. Nothing here reads a change."""
    text, _ = read_text(Path(root) / "docs/features/index.yaml")
    base = {"catalogue_error": "", "features": [], "features_omitted": [],
            "screens": [], "images": {}}
    if text is None:
        base["catalogue_error"] = "機能の一覧を読めません: docs/features/index.yaml がありません"
        return base
    entries, screens = parse_catalogue(text)
    if not entries:
        base["catalogue_error"] = "機能の一覧を読めません: 項目がありません"
        return base
    omitted = [entry["id"] for entry in entries[FEATURES:]]
    manual, _ = read_text(Path(root) / "docs/MANUAL.ja.md")
    images, features = {}, []
    for entry in entries[:FEATURES]:
        folder = Path(root) / "docs/features" / entry["id"]
        problems, documents = [], {}
        for name in ("brief.md", "spec.md"):
            body, cut = read_text(folder / name)
            if body is None:
                problems.append("文書がありません: " + name)
                documents[name] = None
            else:
                documents[name] = {"text": body, "truncated": cut}
        image = entry.get("image", "")
        if image:
            data = None
            try:
                with open(Path(root) / "docs/images" / image, "rb") as handle:
                    data = handle.read(IMAGE_BYTES + 1)
            except OSError:
                problems.append("画像がありません: " + image)
            if data is not None and len(data) > IMAGE_BYTES:
                problems.append("画像が大きすぎます: " + image)
                data = None
            if data is None:
                image = ""
            elif image not in images:
                kind = "jpeg" if image.lower().endswith((".jpg", ".jpeg")) else (
                    "webp" if image.lower().endswith(".webp") else "png")
                images[image] = "data:image/%s;base64,%s" % (
                    kind, base64.b64encode(data).decode("ascii"))
        features.append({
            "id": entry["id"],
            "title": entry.get("title", entry["id"]),
            "screen": entry.get("screen", ""),
            "domain": entry.get("domain", ""),
            "summary": entry.get("summary", ""),
            "paths": entry.get("paths", "").split(),
            "shared": entry.get("shared", "").split(),
            "keywords": entry.get("keywords", "").split(),
            "manual": entry.get("manual", ""),
            "manual_text": manual_section(manual or "", entry.get("manual", "")),
            "image": image,
            "brief": documents["brief.md"],
            "spec": documents["spec.md"],
            "problems": problems,
            "badge": "steady",
            "stage": "",
            "changes": [],
        })
    for feature in features:
        if feature["screen"] and feature["screen"] not in [s["id"] for s in screens]:
            feature["problems"].append("画面「%s」が一覧にありません" % feature["screen"])
    for screen in screens:
        members = [f for f in features if f["screen"] == screen["id"]]
        screen["features"] = [f["id"] for f in members]
        screen["image"] = next((f["image"] for f in members if f["image"]), "")
    base.update({"features": features, "features_omitted": omitted,
                 "screens": screens, "images": images})
    return base


def plan_paths(change, docs):
    """The repository paths a change's plan names, without the paths that say
    nothing about what it is about."""
    found = []
    for line in section(docs.get("plan.md", ""), "Files that change"):
        tokens = re.findall(r"`([^`]+)`", line)
        if line.startswith("|"):
            cells = line.split("|")
            if len(cells) > 1:
                tokens.append(cells[1].replace("`", "").strip())
        for token in tokens:
            token = token.strip().rstrip(".,")
            if (looks_like_path(token) and token != "src/tests.rs"
                    and not token.startswith("docs/sdlc/changes/" + change)):
                found.append(token)
    return found


def link_features(model_features, changes):
    """Tie each change to the features it touches, strongest link first:
    `recorded` (its state.yaml names the feature), `inferred` (its plan lists a
    file the feature claims, unless more than two features claim it), and
    `estimated` (a keyword of the feature in its name or title, only when
    nothing else tied it)."""
    claims = {}
    for feature in model_features:
        for path in feature["paths"]:
            claims.setdefault(path, []).append(feature["id"])
    by_id = {feature["id"]: feature for feature in model_features}
    unmapped = []
    for change in changes:
        links = {}
        named = change["features_named"].split()
        for word in named:
            if word in by_id:
                links[word] = "recorded"
        if named != ["none"]:
            for token in change["plan_paths"]:
                for path, owners in claims.items():
                    if len(owners) > 2:
                        continue
                    if token == path or (path.endswith("/") and token.startswith(path)):
                        for owner in owners:
                            links.setdefault(owner, "inferred")
            if not links:
                text = (change["name"] + " " + change["title"]).lower()
                for feature in model_features:
                    if any(keyword_hit(word, text) for word in feature["keywords"]):
                        links[feature["id"]] = "estimated"
        change["features"] = [{"id": key, "link": value} for key, value in links.items()]
        if not links and named != ["none"]:
            unmapped.append(change["name"])
        for key, value in links.items():
            by_id[key]["changes"].append({"name": change["name"], "link": value})
    lookup = {change["name"]: change for change in changes}
    for feature in model_features:
        feature["changes"].sort(key=lambda item: item["name"], reverse=True)
        open_ones = [lookup[c["name"]] for c in feature["changes"]
                     if c["link"] != "estimated" and lookup[c["name"]]["group"] in ("needs", "flight")]
        if any(c["group"] == "needs" for c in open_ones):
            feature["badge"] = "waiting"
        elif open_ones:
            feature["badge"] = "changing"
        if open_ones:
            feature["stage"] = open_ones[0]["stage"]
    return sorted(unmapped, reverse=True)


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


# Change 315: the domains, the glossary, the screen flow and the requirement
# board. Each comes from one flat file under docs/features/ (or, for the
# board, the specs' Behaviour lines), read here and checked by the suite.
FILE_BYTES = 256 * 1024
DOMAINS = 16
TERMS = 400
FLOW_SCREENS = 80
FLOW_WAYS = 300
CHORD_WORDS = (("Mod+", "⌘"), ("Shift+", "⇧"), ("Alt+", "⌥"), ("Ctrl+", "⌃"))


def parse_flat(text, list_name, opener):
    """The entries of `list_name:` in a flat file, each opened by
    `  - <opener>:` and continued by `    key: value` lines; no YAML library,
    the same reading as `parse_flat_list` in src/tests.rs."""
    entries, inside = [], False
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        if not line.startswith(" "):
            inside = line.rstrip() == list_name + ":"
            continue
        if not inside:
            continue
        start = re.match(r"^  - %s:\s*(.*?)\s*$" % re.escape(opener), line)
        if start:
            entries.append({opener: start.group(1)})
            continue
        field = re.match(r"^    ([\w-]+)\s*:\s*(.*?)\s*$", line)
        if field and entries:
            entries[-1][field.group(1)] = field.group(2)
    return entries


def read_flat_file(root, name, unreadable):
    """The file's text, or the reason it cannot be used, in the words the
    view shows."""
    text, cut = read_text(Path(root) / "docs/features" / name, FILE_BYTES)
    if text is None:
        return None, "%s: docs/features/%s がありません" % (unreadable, name)
    if cut:
        return None, "%s: docs/features/%s が 256 KiB を超えています" % (unreadable, name)
    return text, ""


def phrases(value):
    return [part.strip() for part in (value or "").split(" | ") if part.strip()]


def capped(entries, ceiling, noun):
    if len(entries) <= ceiling:
        return entries, ""
    return entries[:ceiling], "%sが多すぎます。%d 件のうち %d 件を表示しています" % (
        noun, len(entries), ceiling)


def module_lines(root):
    """Every `src/` path the module map of CLAUDE.md names, with the sentence
    that names it."""
    text, _ = read_text(Path(root) / "CLAUDE.md")
    text = text or ""
    start = text.find("## Shape of this codebase")
    end = text.find("## Policies", start)
    found = {}
    if start < 0:
        return found
    for line in text[start:end if end > 0 else len(text)].splitlines():
        for token in re.findall(r"`(src/[^`]+)`", line):
            if token in found:
                continue
            after = line.split("`" + token + "`", 1)[1]
            said = re.sub(r"`", "", after).strip(" —-,:")
            found[token] = re.split(r"(?<=\.)\s", said, maxsplit=1)[0]
    return found


def read_domains(root, features):
    out = {"domains": [], "domains_error": "", "domains_notice": ""}
    text, error = read_flat_file(root, "domains.yaml", "ドメインの定義を読めません")
    if text is None:
        out["domains_error"] = error
        return out
    entries = parse_flat(text, "domains", "id")
    if not entries:
        out["domains_error"] = "ドメインの定義を読めません: 項目がありません"
        return out
    entries, out["domains_notice"] = capped(entries, DOMAINS, "ドメイン")
    lines = module_lines(root)
    for entry in entries:
        out["domains"].append({
            "id": entry["id"],
            "title": entry.get("title", entry["id"]),
            "summary": entry.get("summary", ""),
            "owns": entry.get("owns", ""),
            "excludes": entry.get("excludes", ""),
            "adjacent": entry.get("adjacent", "").split(),
            "modules": [{"path": path, "about": lines.get(path, "")}
                        for path in entry.get("modules", "").split()],
            "features": [f["id"] for f in features if f.get("domain") == entry["id"]],
            "changes": [],
        })
    return out


def read_glossary(root, features):
    out = {"terms": [], "terms_error": "", "terms_notice": ""}
    text, error = read_flat_file(root, "glossary.yaml", "用語集を読めません")
    if text is None:
        out["terms_error"] = error
        return out
    entries = parse_flat(text, "terms", "id")
    if not entries:
        out["terms_error"] = "用語集を読めません: 項目がありません"
        return out
    entries, out["terms_notice"] = capped(entries, TERMS, "用語")
    domain_of = {f["id"]: f.get("domain", "") for f in features}
    for entry in entries:
        word = entry.get("term", "")
        out["terms"].append({
            "id": entry["id"],
            "term": word,
            "meaning": entry.get("meaning", ""),
            "feature": entry.get("feature", ""),
            "domain": domain_of.get(entry.get("feature", ""), ""),
            "en": entry.get("en", ""),
            "ui": phrases(entry.get("ui")),
            "code": entry.get("code", "").split(),
            "avoid": phrases(entry.get("avoid")),
            "see": entry.get("see", "").split(),
            "used_by": [f["id"] for f in features
                        if word and f.get("brief") and word in f["brief"]["text"]],
        })
    return out


def keymap_chords(root):
    """Each remappable action's default chord as a person reads it (⌘1)."""
    text, _ = read_text(Path(root) / "src/app/keymap.rs")
    chords = {}
    for action, default in re.findall(
            r'id:\s*"([^"]+)",\s*label:\s*"[^"]*",\s*default:\s*"([^"]+)"', text or ""):
        for word, mark in CHORD_WORDS:
            default = default.replace(word, mark)
        chords[action] = default
    return chords


def read_flow(root, features, screens_order, terms, images):
    out = {"flow": {"screens": [], "ways": [], "bands": [], "width": 0, "height": 0},
           "flow_error": "", "flow_notice": ""}
    text, error = read_flat_file(root, "flow.yaml", "画面の流れを読めません")
    if text is None:
        out["flow_error"] = error
        return out
    screens = parse_flat(text, "screens", "id")
    ways = parse_flat(text, "ways", "from")
    if not screens:
        out["flow_error"] = "画面の流れを読めません: 画面がありません"
        return out
    screens, notice_screens = capped(screens, FLOW_SCREENS, "画面")
    ways, notice_ways = capped(ways, FLOW_WAYS, "道")
    out["flow_notice"] = "。".join(n for n in (notice_screens, notice_ways) if n)
    known = {s["id"] for s in screens}
    chords = keymap_chords(root)
    carried_ways = []
    for way in ways:
        if way.get("from") not in known or way.get("to") not in known:
            continue
        key = way.get("key", "")
        carried_ways.append({
            "from": way["from"], "to": way["to"], "key": key or None,
            "chord": chords.get(key, ""), "label": way.get("label", ""),
            "click": way.get("click", ""), "note": way.get("note", ""),
            "press": way.get("press", ""), "unseen": way.get("unseen", ""),
        })
    # Breadth-first order from ホーム decides where a box sits in its band.
    order, queue = {}, ["home"] if "home" in known else []
    while queue:
        here = queue.pop(0)
        if here in order:
            continue
        order[here] = len(order)
        queue.extend(w["to"] for w in carried_ways if w["from"] == here)
    for s in screens:
        order.setdefault(s["id"], len(order))
    bands = [b for b in screens_order if any(s.get("screen") == b for s in screens)]
    bands += sorted({s.get("screen", "") for s in screens} - set(bands))
    box_w, box_h, gap_x, gap_y, left, per_row = 150, 54, 30, 46, 132, 5
    widest, top, placed_bands = 0, 24, []
    feature_terms = {}
    for term in terms:
        feature_terms.setdefault(term["feature"], []).append(term["id"])
    carried = []
    for band in bands:
        members = sorted((s for s in screens if s.get("screen", "") == band),
                         key=lambda s: order[s["id"]])
        widest = max(widest, min(len(members), per_row))
        rows = (len(members) + per_row - 1) // per_row
        placed_bands.append({"id": band, "y": top, "height": rows * box_h + (rows - 1) * 18})
        for index, s in enumerate(members):
            column, line = index % per_row, index // per_row
            on = s.get("features", "").split()
            image = s.get("image", "")
            if image and image not in images:
                image = ""
            carried.append({
                "id": s["id"], "title": s.get("title", s["id"]), "kind": s.get("kind", ""),
                "screen": band, "features": on, "parent": s.get("parent", ""),
                "page": s.get("page", ""), "tab": s.get("tab", ""), "image": image,
                "note": s.get("note", ""), "noshot": s.get("noshot", ""),
                "x": left + column * (box_w + gap_x), "y": top + line * (box_h + 18),
                "in": [i for i, w in enumerate(carried_ways) if w["to"] == s["id"]],
                "out": [i for i, w in enumerate(carried_ways) if w["from"] == s["id"]],
                "terms": sorted({t for f in on for t in feature_terms.get(f, [])}),
            })
        top += placed_bands[-1]["height"] + gap_y
    by_id = {s["id"]: s for s in carried}
    out["flow"] = {
        "screens": [by_id[s["id"]] for s in screens],
        "ways": carried_ways,
        "bands": placed_bands,
        "box": [box_w, box_h],
        "width": left + widest * (box_w + gap_x),
        "height": top,
    }
    return out


SHOTS_INDEX = "docs/images/screens/screens.json"
SHOTS_INDEX_BYTES = 4 * 1024 * 1024
SHOT_BYTES = 600 * 1024
SHOTS_BYTES = 16 * 1024 * 1024
SHOT_SCREENS = 80
SHOT_CONTROLS = 300
SHOT_WORDS = 400
ORDINAL = re.compile(r"^(.*\S)\s+#(\d+)$")


def finite(value):
    """A number a page can place by: not a bool, not NaN, not infinite."""
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return False
    try:
        return math.isfinite(value)
    except OverflowError:
        # An integer past a float's range is no place on a page.
        return False


def jpeg_size(data):
    """(width, height) from a JPEG's frame header, or None — the reading
    scripts/capture-screens.py writes with."""
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
            return (int.from_bytes(data[at + 7:at + 9], "big"),
                    int.from_bytes(data[at + 5:at + 7], "big"))
        at += 2 + length
    return None


def answering(controls, words):
    """The controls that answer to a way's words, in reading order, by
    `answering` in scripts/verify-app.py — the rule the runner presses with;
    `words #n` keeps only the n-th."""
    ordinal = ORDINAL.match(words)
    if ordinal:
        words = ordinal.group(1)
    found = [controls[at] for at in runner().answering([c["name"] for c in controls], words)]
    if ordinal:
        n = int(ordinal.group(2))
        return found[n - 1:n] if 0 < n <= len(found) else []
    return found


_RUNNER = []


def runner():
    """scripts/verify-app.py, loaded once, for its matching rule."""
    if not _RUNNER:
        spec = importlib.util.spec_from_file_location(
            "verify_app", Path(__file__).resolve().parent / "verify-app.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        _RUNNER.append(module)
    return _RUNNER[0]


def read_shots(root, flow, terms):
    """Join docs/images/screens/screens.json to the flow: each screen's
    picture (as a data URL in `shots`), its words and the terms they show,
    and each way's pressable area. Every reason a picture is not shown is
    said per screen; nothing here stops the map from being written."""
    out = {"shots": {}, "shots_error": "", "shots_notice": ""}
    screens = flow["screens"]
    for screen in screens:
        screen.update({"shot": None, "shot_problem": "", "words": [], "terms_on": []})
    for way in flow["ways"]:
        way.update({"area": None, "answers": None})
    path = Path(root) / SHOTS_INDEX
    text, cut = read_text(path, SHOTS_INDEX_BYTES)
    index = None
    if text is None:
        out["shots_error"] = "写真がまだありません。scripts/capture-screens.py で撮れます"
    elif cut:
        out["shots_error"] = "写真を読めませんでした: screens.json が 4 MiB を超えています"
    else:
        try:
            index = json.loads(text)
        except ValueError:
            index = None
        if not isinstance(index, dict) or not isinstance(index.get("screens"), dict):
            out["shots_error"] = "写真を読めませんでした: screens.json を読めません"
            index = None
    entries = index["screens"] if index else {}
    if len(entries) > SHOT_SCREENS:
        out["shots_notice"] = "写真が多すぎます。%d 画面のうち %d 画面を載せています" % (
            len(entries), SHOT_SCREENS)
    allowed = list(entries)[:SHOT_SCREENS]
    spent = 0
    by_id = {s["id"]: s for s in screens}
    for screen in screens:
        if screen.get("noshot"):
            screen["shot_problem"] = screen["noshot"]
            continue
        entry = entries.get(screen["id"]) if screen["id"] in allowed else None
        if not isinstance(entry, dict):
            if index is not None:
                screen["shot_problem"] = "この画面はまだ撮影されていません"
            continue
        picture = entry.get("picture")
        window = entry.get("window") or {}
        try:
            ww, wh = float(window["w"]), float(window["h"])
            scale = float(entry["scale"])
        except (KeyError, TypeError, ValueError):
            ww = wh = scale = math.nan
        # Python reads NaN and Infinity in JSON; neither is a size, and a NaN
        # passes every comparison below by failing it.
        if not all(math.isfinite(v) and v > 0 for v in (ww, wh, scale)):
            screen["shot_problem"] = "写真を読めませんでした: 記録に窓の大きさがありません"
            continue
        if not isinstance(picture, str) or "/" in picture or picture.startswith("."):
            screen["shot_problem"] = "写真を読めませんでした: 記録のファイル名が正しくありません"
            continue
        try:
            with open(path.parent / picture, "rb") as handle:
                data = handle.read(SHOT_BYTES + 1)
        except (OSError, ValueError):
            # ValueError: a NUL in the name, which `open` refuses before the disk.
            screen["shot_problem"] = "写真を読めませんでした: %s がありません" % picture.replace("\0", "")
            continue
        if len(data) > SHOT_BYTES:
            screen["shot_problem"] = "写真を読めませんでした: 600 KiB を超えています"
            continue
        size = jpeg_size(data)
        if size is None or abs(size[0] - ww * scale) > 1 or abs(size[1] - wh * scale) > 1:
            screen["shot_problem"] = "写真を読めませんでした: 大きさが記録と違います"
            continue
        if spent + len(data) > SHOTS_BYTES:
            screen["shot_problem"] = "写真を読めませんでした: 写真の合計が 16 MiB を超えます"
            continue
        spent += len(data)
        out["shots"][screen["id"]] = "data:image/jpeg;base64," + base64.b64encode(data).decode("ascii")
        words = [w for w in entry.get("words") or [] if isinstance(w, str)][:SHOT_WORDS]
        controls = [c for c in entry.get("controls") or []
                    if isinstance(c, dict) and isinstance(c.get("name"), str)
                    and all(finite(c.get(k)) for k in ("x", "y", "w", "h"))][:SHOT_CONTROLS]
        screen["shot"] = {"w": ww, "h": wh, "taken": str(entry.get("taken", "")),
                          "commit": str(entry.get("commit", ""))}
        screen["words"] = words
        screen["controls"] = controls
        screen["terms_on"] = [
            t["id"] for t in terms
            if any(word and any(word in said for said in words)
                   for word in [t["term"]] + t["ui"])]
    for way in flow["ways"]:
        screen = by_id.get(way["from"])
        if not screen or not screen.get("shot"):
            continue
        words = way.get("press") or way.get("label")
        if not words:
            continue
        found = answering(screen["controls"], words)
        way["answers"] = len(answering(screen["controls"], ORDINAL.sub(r"\1", words)))
        if len(found) == 1 and not way.get("unseen"):
            c = found[0]
            way["area"] = {k: c.get(k) for k in ("x", "y", "w", "h")}
    for screen in screens:
        screen.pop("controls", None)
    return out


def behaviour_items(spec):
    """The numbered items of a spec's `## Behaviour` section with their
    continuation lines, read as `behaviour_lines` in src/tests.rs reads them."""
    at = spec.find("## Behaviour")
    if at < 0:
        return []
    body = spec[at + len("## Behaviour"):]
    nxt = body.find("\n## ")
    body = body[:nxt] if nxt >= 0 else body
    items = []
    for line in body.splitlines():
        found = re.match(r"^(\d+)\. (.*)$", line)
        if found:
            items.append([int(found.group(1)), found.group(2)])
        elif items and line.startswith("   "):
            items[-1][1] += " " + line.strip()
    return items


TESTS_BYTES = 64 * 1024 * 1024


def test_names(root):
    """Every `fn` of src/tests.rs: a `Pinned by` name pins a line only when the
    test it names exists."""
    text, _ = read_text(Path(root) / "src/tests.rs", TESTS_BYTES)
    return set(re.findall(r"\bfn\s+([A-Za-z_]\w*)", text or ""))


def read_requirements(features, tests=None):
    rows = []
    for feature in features:
        spec = feature.get("spec")
        if not spec:
            continue
        for number, text in behaviour_items(spec["text"]):
            at = text.find("Pinned by")
            found = re.findall(r"`([^`]+)`", text[at:]) if at >= 0 else []
            said = (text[:at] if at >= 0 else text).strip()
            known = [name for name in found if tests is None or name in tests]
            rows.append({"feature": feature["id"], "domain": feature.get("domain", ""),
                         "number": number, "text": said, "tests": found,
                         "pinned": bool(known),
                         "missing": [name for name in found if name not in known]})
    return rows


def link_domains(domains, features):
    """A domain's changes are its features' changes, newest first."""
    by_feature = {f["id"]: f for f in features}
    for domain in domains:
        seen = {}
        for fid in domain["features"]:
            for link in by_feature[fid]["changes"]:
                if link["link"] != "estimated":
                    seen.setdefault(link["name"], link["link"])
        domain["changes"] = sorted(seen, reverse=True)


def build(root, backlog=None):
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
    layer = read_features(root)
    unmapped = link_features(layer["features"], ordered)
    domains = read_domains(root, layer["features"])
    link_domains(domains["domains"], layer["features"])
    glossary = read_glossary(root, layer["features"])
    flow = read_flow(root, layer["features"], [s["id"] for s in layer["screens"]],
                     glossary["terms"], layer["images"])
    shots = read_shots(root, flow["flow"], glossary["terms"])
    requirements = read_requirements(layer["features"], test_names(root))
    domain_by_id = {d["id"]: d for d in domains["domains"]}
    for feature in layer["features"]:
        feature["terms"] = [t["id"] for t in glossary["terms"] if t["feature"] == feature["id"]]
        feature["flow_screens"] = [s["id"] for s in flow["flow"]["screens"]
                                   if feature["id"] in s["features"]]
        owner = domain_by_id.get(feature.get("domain", ""))
        feature["modules"] = [m["path"] for m in owner["modules"]] if owner else []
        feature["requirements"] = [r["number"] for r in requirements if r["feature"] == feature["id"]]
    for domain in domains["domains"]:
        domain["screens"] = [s["id"] for s in flow["flow"]["screens"]
                             if set(s["features"]) & set(domain["features"])]
    for change in ordered:
        del change["plan_paths"]
    feature_depth(root, layer["features"], requirements, flow["flow"], ordered)
    backlog = backlog or {"asked": False, "items": [], "total": 0, "error": "", "notice": ""}
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
        "unmapped": unmapped,
        "catalogue_error": layer["catalogue_error"],
        "features": layer["features"],
        "features_omitted": layer["features_omitted"],
        "screens": layer["screens"],
        "images": layer["images"],
        "domains": domains["domains"],
        "domains_error": domains["domains_error"],
        "domains_notice": domains["domains_notice"],
        "terms": glossary["terms"],
        "terms_error": glossary["terms_error"],
        "terms_notice": glossary["terms_notice"],
        "flow": flow["flow"],
        "flow_error": flow["flow_error"],
        "flow_notice": flow["flow_notice"],
        "requirements": requirements,
        "shots": shots["shots"],
        "shots_error": shots["shots_error"],
        "shots_notice": shots["shots_notice"],
        "overview": overview_of(layer["features"], domains["domains"], flow["flow"],
                                requirements, glossary["terms"], counts),
        "status": status_of(ordered, layer["features"], domains["domains"], requirements),
        "backlog": backlog,
        "work": work_of(ordered, backlog),
        "process": read_process(root, ordered),
        "catalog": read_catalog(root),
    }


def overview_of(features, domains, flow, requirements, terms, counts):
    """The counts 概要 shows, each derived from the model it sums."""
    def share(rows):
        return {"requirements": len(rows), "pinned": sum(1 for r in rows if r["pinned"])}
    per_domain = []
    for domain in domains:
        rows = [r for r in requirements if r["domain"] == domain["id"]]
        per_domain.append(dict({"id": domain["id"], "features": len(domain["features"])}, **share(rows)))
    bare = {}
    for row in requirements:
        if not row["pinned"]:
            bare[row["feature"]] = bare.get(row["feature"], 0) + 1
    return dict({
        "features": len(features),
        "domains": len(domains),
        "screens": len(flow["screens"]),
        "photographed": sum(1 for s in flow["screens"] if s.get("shot")),
        "terms": len(terms),
        "changes": {group: counts[group] for group in GROUP_ORDER},
        "changes_total": counts["total"],
        "per_domain": per_domain,
        "least_pinned": [{"feature": f, "bare": n} for f, n in
                         sorted(bare.items(), key=lambda item: (-item[1], item[0]))[:5]],
    }, **share(requirements))


# Change 325: where work stands, a feature in depth, the work board, the
# pipeline and the catalogue. Each is derived from what `build` already holds
# or from the one file that defines it; nothing here is a second record.
ATTENTION_ROWS = 5
NEXT_ROWS = 8
LANDED_CARDS = 8
EVIDENCE_ROWS = 3
RAIL = ("brief", "spec", "screens", "requirements", "tests", "live")
LAYERS = ("draw", "text", "core", "test", "script", "doc")
TEXT_FILES = ("src/i18n.rs", "src/i18n_tables.rs", "src/theme.rs", "src/glyphs.rs")


def layer_of(path):
    """The first layer whose rule the path meets."""
    if path == "src/tests.rs":
        return "test"
    if path == "src/app.rs" or path.startswith(("src/app/", "src/ui/")):
        return "draw"
    if path in TEXT_FILES:
        return "text"
    if path.startswith("src/"):
        return "core"
    if path.startswith("docs/") or path.endswith(".md"):
        return "doc"
    return "script"


def is_open(change):
    return change["group"] in ("needs", "flight")


def status_of(changes, features, domains, requirements):
    """現況: counts, where each open change is, what a person has to handle,
    what is under way, each domain, each working copy."""
    opened = [c for c in changes if is_open(c)]
    by_badge = lambda rows: {badge: sum(1 for f in rows if f["badge"] == badge) for badge in BADGE}
    cards = []

    def card(kind, items):
        cards.append({"kind": kind, "total": len(items), "items": items[:ATTENTION_ROWS]})

    card("decide", [{"change": c["name"], "text": c["problem"] or c["resume"]}
                    for c in changes if c["group"] == "needs"])
    card("confirm", [{"change": c["name"], "key": k["key"], "text": k["text"]}
                     for c in opened for k in c["contract"]
                     if k["kind"] == "human" and k["verdict"] == "pending"])
    card("carried", [{"change": c["name"], "text": f["person"] or f["finding"]}
                     for c in changes if c["group"] == "done" for f in c["findings"]])
    card("gaps", [{"feature": f["id"],
                   "text": "、".join(f["problems"]) or "テストのある要件がありません"}
                  for f in features if f["problems"] or not f["pinned"]])
    flight = [c for c in changes if c["group"] == "flight"]
    branches = {}
    for c in opened:
        where = c["where"]
        key = (not where["here"], where["branch"], where["path"])
        branches.setdefault(key, []).append(c["name"])
    by_feature = {f["id"]: f for f in features}
    per_domain = []
    for domain in domains:
        members = [by_feature[fid] for fid in domain["features"] if fid in by_feature]
        rows = [r for r in requirements if r["domain"] == domain["id"]]
        per_domain.append(dict(by_badge(members), id=domain["id"], features=len(members),
                               requirements=len(rows),
                               pinned=sum(1 for r in rows if r["pinned"])))
    per_domain.sort(key=lambda d: -d["waiting"])
    return {
        "features": dict(by_badge(features), total=len(features)),
        "phases": [{"stage": stage, "changes": [c["name"] for c in opened if c["stage"] == stage]}
                   for stage in STAGES],
        "off_track": [c["name"] for c in opened if c["stage"] not in STAGES],
        "attention": cards,
        "next": [{"name": c["name"], "stage": c["stage"], "resume": c["resume"],
                  "passed": sum(1 for k in c["contract"] if k["verdict"] == "passed"),
                  "waived": sum(1 for k in c["contract"] if k["verdict"] == "waived"),
                  "total": len(c["contract"])} for c in flight[:NEXT_ROWS]],
        "next_total": len(flight),
        "branches": [{"here": not key[0], "branch": key[1], "path": key[2], "changes": names}
                     for key, names in sorted(branches.items())],
        "domains": per_domain,
    }


def feature_depth(root, features, requirements, flow, changes):
    """What a feature's page shows past its documents: how much of it is
    tested, which of its six steps exist, its code by layer, the change open
    on it and the evidence its latest changes landed on."""
    lines = module_lines(root)
    shot = {s["id"]: bool(s.get("shot")) for s in flow["screens"]}
    by_name = {c["name"]: c for c in changes}
    for feature in features:
        rows = [r for r in requirements if r["feature"] == feature["id"]]
        pinned = sum(1 for r in rows if r["pinned"])
        cover = ("empty" if not rows else "full" if pinned == len(rows)
                 else "partial" if pinned else "none")
        spec = feature["spec"]["text"] if feature.get("spec") else ""
        screens = [shot.get(sid, False) for sid in feature.get("flow_screens", [])]
        seen = [line for line in section(spec, "Seen in the running app") if line.strip()]
        state = {
            "brief": "done" if feature.get("brief") else "missing",
            "spec": "done" if feature.get("spec") else "missing",
            "screens": ("done" if screens and all(screens)
                        else "partial" if any(screens) else "missing"),
            "requirements": "done" if rows else "missing",
            "tests": {"full": "done", "partial": "partial"}.get(cover, "missing"),
            "live": "done" if seen else "missing",
        }
        files, symbols = [], {}
        for path in feature["paths"] + feature["shared"]:
            if path not in files:
                files.append(path)
        for line in section(spec, "Code and tests"):
            for token in re.findall(r"`([^`]+)`", line):
                path, _, symbol = token.partition("::")
                path = path.strip().rstrip(".,")
                if not looks_like_path(path) or "/" not in path:
                    continue
                if path not in files:
                    files.append(path)
                if symbol and symbol not in symbols.setdefault(path, []):
                    symbols[path].append(symbol)
        code = [{"layer": layer, "files": [
            {"path": path, "about": lines.get(path, ""), "symbols": symbols.get(path, [])}
            for path in files if layer_of(path) == layer]} for layer in LAYERS]
        tied = [by_name[link["name"]] for link in feature["changes"] if link["link"] != "estimated"]
        opened = [c for c in tied if is_open(c)]
        feature.update({
            "cover": cover,
            "pinned": pinned,
            "rail": [{"step": step, "state": state[step]} for step in RAIL],
            "code": [entry for entry in code if entry["files"]],
            "next": ({"change": opened[0]["name"], "resume": opened[0]["problem"] or opened[0]["resume"]}
                     if opened else None),
            "evidence": [{"change": c["name"], "obligations": c["obligations"],
                          "findings": len(c["findings"]), "approvals": len(c["approvals"])}
                         for c in tied if c["group"] == "done"][:EVIDENCE_ROWS],
        })


BACKLOG_SECONDS = 30
BACKLOG_BYTES = 1024 * 1024
BACKLOG_ITEMS = 200
BACKLOG_TEXT = 4000
BACKLOG_SCRIPT = "scripts/backlog.py"


def read_backlog(root, seconds=BACKLOG_SECONDS):
    """The backlog change 323 keeps on GitHub, through the mapped root's own
    script: one child, bounded in time and bytes. Every failure is a sentence
    for the board; none stops the map."""
    out = {"asked": True, "items": [], "total": 0, "error": "", "notice": ""}

    def failed(why):
        return dict(out, error="バックログを読めませんでした: " + why)

    script = Path(root) / BACKLOG_SCRIPT
    if not script.is_file():
        return failed(BACKLOG_SCRIPT + " がありません")
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    late = []
    with tempfile.TemporaryFile() as errors:
        try:
            child = subprocess.Popen(
                [sys.executable, str(script), "--timeout", str(seconds), "list", "--json"],
                cwd=str(root), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                stderr=errors, env=environment, start_new_session=True)
        except OSError:
            return failed("起動できません")

        def stop():
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except OSError:
                pass

        def overdue():
            late.append(True)
            stop()

        timer = threading.Timer(seconds, overdue)
        timer.start()
        try:
            # Read against the deadline, not to the end: a grandchild in a
            # session of its own keeps the pipe open past killpg.
            deadline = time.monotonic() + seconds
            chunks, size = [], 0
            while size <= BACKLOG_BYTES:
                left = deadline - time.monotonic()
                if left <= 0 or not select.select([child.stdout], [], [], left)[0]:
                    overdue()
                    break
                chunk = os.read(child.stdout.fileno(), 65536)
                if not chunk:
                    break
                chunks.append(chunk)
                size += len(chunk)
            data = b"".join(chunks)
            if len(data) > BACKLOG_BYTES:
                stop()
            child.stdout.close()
            code = child.wait()
        finally:
            timer.cancel()
        errors.seek(0)
        said = errors.read(2000).decode("utf-8", errors="replace").strip().splitlines()
    if late:
        return failed("%d 秒以内に応答がありません" % seconds)
    if len(data) > BACKLOG_BYTES:
        return failed("出力が 1 MiB を超えています")
    if code != 0:
        return failed("終了コード %d%s" % (code, "（%s）" % said[-1][:200] if said else ""))
    try:
        listed = json.loads(data.decode("utf-8", errors="replace"))
    except ValueError:
        listed = None
    if not isinstance(listed, list):
        return failed("一覧として読めません")
    listed = [item for item in listed if isinstance(item, dict)]
    out["total"] = len(listed)
    if len(listed) > BACKLOG_ITEMS:
        out["notice"] = "バックログが多すぎます。%d 件のうち %d 件を表示しています" % (
            len(listed), BACKLOG_ITEMS)
    seen = set()
    for item in listed[:BACKLOG_ITEMS]:
        number = item.get("number")
        # An issue is one card: a number repeated, or one that is not whole,
        # names none the board can tell apart.
        if not finite(number) or number != int(number) or int(number) in seen:
            continue
        seen.add(int(number))
        url = item.get("url")
        held = item.get("held_by")
        out["items"].append({
            "number": int(number),
            "title": str(item.get("title") or ""),
            "kind": str(item.get("kind") or ""),
            "score": item.get("score") if finite(item.get("score")) else None,
            "ice": [item.get(k) if finite(item.get(k)) else None
                    for k in ("impact", "confidence", "ease")],
            # A link is drawn only when it goes where the backlog lives.
            "url": url if isinstance(url, str) and url.startswith("https://github.com/") else "",
            "text": str(item.get("body") or "")[:BACKLOG_TEXT],
            "held": (str(held) if held else "unknown" if item.get("held_known") is False else ""),
            "held_since": str(item.get("held_since") or ""),
        })
    return out


def work_of(changes, backlog):
    """The board: every card in one column."""
    cards = []
    for item in backlog["items"]:
        held = item["held"] not in ("", "unknown")
        cards.append({"id": "issue:%d" % item["number"], "column": "held" if held else "backlog",
                      "owner": "ai" if held else "none", "title": item["title"],
                      "issue": item["number"], "kind": item["kind"], "score": item["score"],
                      "url": item["url"], "held": item["held"], "text": item["text"]})
    landed = 0
    for change in changes:
        if change["group"] == "done":
            landed += 1
            if landed > LANDED_CARDS:
                continue
        elif not is_open(change):
            continue
        column = {"needs": "you", "flight": "flight", "done": "done"}[change["group"]]
        cards.append({"id": "change:" + change["name"], "column": column,
                      "owner": {"you": "person", "flight": "ai"}.get(column, "none"),
                      "title": change["title"], "change": change["name"],
                      "stage": change["stage"],
                      "text": change["problem"] or change["resume"]})
    return {"columns": ["backlog", "held", "flight", "you", "done"], "cards": cards}


def read_process(root, changes):
    """The pipeline as its three tables define it."""
    out = {"stages": list(STAGES), "routes": [], "surfaces": [], "methods": [], "error": ""}
    unread = []

    def table(name):
        text, cut = read_text(Path(root) / name, FILE_BYTES)
        if text is None or cut:
            unread.append(name)
            return ""
        return text

    routes = table("docs/sdlc/routes.yaml")
    risk = table("docs/sdlc/risk.yaml")
    methods = table("docs/sdlc/verification-methods.yaml")
    asked_of_route = dict(block(methods, "routes"))
    asked_of_surface = dict(block(methods, "surfaces"))

    def named(value):
        return [word for word in (value or "").split(",") if word and word != "-"]

    took = {}
    for change in changes:
        took[change["route"]] = took.get(change["route"], 0) + 1
    for name, value in block(routes, "routes"):
        parts = value.split()
        if len(parts) != 5:
            continue
        out["routes"].append({
            "name": name, "stages": [s for s in parts[0].split(",") if s in STAGES],
            "attempts": parts[1], "gate": parts[2],
            "requires": "" if parts[3] == "-" else parts[3], "review": parts[4],
            "methods": named(asked_of_route.get(name)), "changes": took.get(name, 0)})
    for name, value in block(risk, "surfaces"):
        parts = value.split()
        if len(parts) != 4:
            continue
        out["surfaces"].append({
            "name": name, "tier": parts[0], "autonomy": parts[1], "screen": parts[2] == "yes",
            "paths": parts[3].split(","), "methods": named(asked_of_surface.get(name))})
    for name, value in block(methods, "methods"):
        kind, _, command = value.partition(" ")
        out["methods"].append({"name": name, "kind": kind, "command": command.strip()})
    if unread:
        out["error"] = "進め方の表を読めません: " + "、".join(unread)
    return out


SOURCE_FILES = 400
SOURCE_BYTES = 2 * 1024 * 1024
PALETTE_NAMES = {"DARK": "dark", "LIGHT": "light", "HIGH_CONTRAST": "high-contrast"}


def css_colour(call, arguments):
    numbers = [int(n) for n in re.findall(r"\d+", arguments)]
    if call == "from_rgb" and len(numbers) == 3:
        return "#%02x%02x%02x" % tuple(min(255, n) for n in numbers)
    if call == "from_black_alpha" and len(numbers) == 1:
        return "rgba(0,0,0,%s)" % ("%.3f" % (min(255, numbers[0]) / 255)).rstrip("0").rstrip(".")
    return ""


def read_catalog(root):
    """What the application defines: the colour roles of its three palettes,
    its marks and how often each is used, its keys."""
    out = {"palettes": [], "icons": [], "keys": [], "errors": []}
    root = Path(root)

    def source(name):
        text, cut = read_text(root / name, SOURCE_BYTES)
        if text is None or cut:
            out["errors"].append("読めませんでした: " + name)
            return ""
        return text

    theme = source("src/theme.rs")
    for constant, body in re.findall(
            r"const (\w+)_PALETTE: Palette = Palette \{(.*?)\n\};", theme, re.S):
        roles = []
        for role, value in re.findall(r"^    (\w+):\s*(.*?)(?=^    \w+:|\Z)", body, re.S | re.M):
            calls = re.findall(r"Color32::(\w+)\(([^)]*)\)", value)
            colours = [c for c in (css_colour(call, args) for call, args in calls) if c]
            if value.lstrip().startswith("["):
                roles.extend({"role": "%s.%d" % (role, at), "colour": colour}
                             for at, colour in enumerate(colours))
            elif colours:
                roles.append({"role": role, "colour": colours[0]})
        out["palettes"].append({"name": PALETTE_NAMES.get(constant, constant.lower()), "roles": roles})
    glyphs = source("src/glyphs.rs")
    drawn = dict(re.findall(r"const (ICON_\w+): &str = icons::(\w+);", glyphs))
    at = glyphs.find("ICON_VOCABULARY")
    listed = re.findall(r'\("(ICON_\w+)"', glyphs[at:]) if at >= 0 else []
    uses = {name: 0 for name in listed}
    files = []
    for folder, folders, names in os.walk(str(root / "src")):
        folders.sort()
        for name in sorted(names):
            path = Path(folder) / name
            relative = path.relative_to(root).as_posix()
            if name.endswith(".rs") and relative not in ("src/glyphs.rs", "src/tests.rs"):
                files.append(path)
    for path in files[:SOURCE_FILES]:
        text, _ = read_text(path, SOURCE_BYTES)
        for name in re.findall(r"\bICON_\w+\b", text or ""):
            if name in uses:
                uses[name] += 1
    out["icons"] = [{"name": name, "glyph": drawn.get(name, ""), "uses": uses[name]} for name in listed]
    keymap = source("src/app/keymap.rs")
    for action, label, default in re.findall(
            r'id:\s*"([^"]+)",\s*label:\s*"([^"]*)",\s*default:\s*"([^"]+)"', keymap):
        for word, mark in CHORD_WORDS:
            default = default.replace(word, mark)
        out["keys"].append({"id": action, "label": label, "chord": default})
    return out


BADGE = {"steady": "安定", "changing": "変更中", "waiting": "判断待ち"}
STAGE_WORDS = {"plan": "計画", "design": "設計", "build": "実装", "test": "テスト",
               "deploy": "出荷", "maintain": "保守"}


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
    if model["catalogue_error"]:
        lines.append(model["catalogue_error"])
    moving = [f for f in model["features"] if f["badge"] != "steady"]
    lines.append("動いている機能")
    if not moving:
        lines.append("  なし")
    for feature in moving:
        lines.append("  %s  %s  %s%s" % (
            feature["id"], feature["title"], BADGE[feature["badge"]],
            " · " + STAGE_WORDS.get(feature["stage"], feature["stage"]) if feature["stage"] else ""))
    screens = model["flow"]["screens"]
    missing = [s for s in screens if not s.get("shot")]
    lines.append("画面 %d · 写真なし %d" % (len(screens), len(missing)))
    for screen in missing:
        lines.append("  %s  %s" % (screen["id"], screen.get("shot_problem") or "写真がありません"))
    for said in (model["shots_error"], model["shots_notice"]):
        if said:
            lines.append(said)
    lines.append("地図: %s" % out)
    return "\n".join(lines) + "\n"


def find(model, word):
    word = word.lower()
    lines = []
    for change in model["changes"]:
        labels = [AREA_LABELS.get(area, area) for area in change["areas"]]
        if (word in change["name"].lower() or word in change["title"].lower()
                or word in (area.lower() for area in change["areas"])
                or word in labels
                or any(word in link["id"] for link in change["features"])):
            lines.append("%s  %s  %s  %s  — %s" % (
                change["name"], change["status"] or "?", change["stage"] or "?",
                ",".join(labels), change["title"]))
    for feature in model["features"]:
        if (word in feature["id"].lower() or word in feature["title"].lower()
                or word in feature["screen"].lower() or word in feature["summary"].lower()):
            lines.insert(0, "機能 %s  %s  %s  %s  — %s" % (
                feature["id"], feature["title"], feature["screen"],
                BADGE[feature["badge"]], feature["summary"]))
    found = []
    for domain in model["domains"]:
        if word in domain["id"].lower() or word in domain["title"].lower():
            found.append("ドメイン %s  %s  — %s" % (domain["id"], domain["title"], domain["summary"]))
    for term in model["terms"]:
        if (word in term["id"].lower() or word in term["term"].lower()
                or word in term["en"].lower()):
            found.append("用語 %s  %s  — %s" % (term["id"], term["term"], term["meaning"]))
    for screen in model["flow"]["screens"]:
        if word in screen["id"].lower() or word in screen["title"].lower():
            found.append("画面 %s  %s" % (screen["id"], screen["title"]))
    lines = found + lines
    return "\n".join(lines) + "\n" if lines else "該当する変更はありません\n"


def data_block(model):
    """JSON that no document's text can end: every character HTML or a script
    parser gives meaning to is written as an escape."""
    text = json.dumps(model, ensure_ascii=False, separators=(",", ":"))
    for raw, escaped in (("&", "\\u0026"), ("<", "\\u003c"), (">", "\\u003e"),
                         ("\u2028", "\\u2028"), ("\u2029", "\\u2029")):
        text = text.replace(raw, escaped)
    return text


def page_of(model):
    return page_template().replace("__HEADLINE__", headline(model)).replace("__DATA__", data_block(model))


def write_page(model, out):
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    page = page_of(model)
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


SERVE_PORT = 8765
REQUEST_BYTES = 8 * 1024
REQUEST_SECONDS = 5
LOOPBACK = "127.0.0.1"


def serve(root, port, requests, model_of):
    """The page on this machine only, built again for every load. Read-only:
    one path, GET alone, and nothing a request carries is kept."""
    class Handler(http.server.BaseHTTPRequestHandler):
        timeout = REQUEST_SECONDS
        server_version = "OperonMap"

        def log_message(self, *arguments):
            pass

        def answer(self, code, body, kind="text/plain; charset=utf-8"):
            self.send_response(code)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(body)

        def handle_one_request(self):
            # The request line and the headers together are cut at 8 KiB.
            self.rfile = Bounded(self.rfile, REQUEST_BYTES)
            self.close_connection = True
            # The timeout bounds each read, not the request: a client sending a
            # byte every few seconds would hold this one-at-a-time server, so
            # its reading side is shut when the whole request is late.
            late = []

            def cut():
                late.append(True)
                try:
                    self.connection.shutdown(socket.SHUT_RD)
                except OSError:
                    pass

            timer = threading.Timer(REQUEST_SECONDS, cut)
            timer.start()
            try:
                try:
                    self.raw_requestline = self.rfile.readline(REQUEST_BYTES + 1)
                    whole = self.raw_requestline and self.raw_requestline.endswith(b"\n")
                    parsed = whole and self.parse_request()
                finally:
                    timer.cancel()
                if late:
                    self.requestline, self.command = "", ""
                    self.request_version = "HTTP/1.0"
                    self.answer(408, "要求が時間内に届きません\n".encode("utf-8"))
                    return
                if not self.raw_requestline:
                    return
                if not whole:
                    # Cut at the ceiling: a line with no end is not a request.
                    self.requestline, self.command = "", ""
                    self.request_version = "HTTP/1.0"
                    self.answer(414, "要求が長すぎます\n".encode("utf-8"))
                    return
                if not parsed:
                    return
                if self.command != "GET":
                    self.answer(405, "GET だけに答えます\n".encode("utf-8"))
                elif self.path.split("?", 1)[0] != "/":
                    self.answer(404, "地図は / にあります\n".encode("utf-8"))
                else:
                    self.answer(200, page_of(model_of()).encode("utf-8"), "text/html; charset=utf-8")
                self.server.answered += 1
            except (OSError, ValueError):
                pass

    class Bounded:
        """A reader that stops giving bytes at the ceiling."""
        def __init__(self, stream, limit):
            self.stream, self.left = stream, limit

        def readline(self, size=-1):
            size = self.left if size < 0 else min(size, self.left)
            line = self.stream.readline(size) if size > 0 else b""
            self.left -= len(line)
            return line

        def __getattr__(self, name):
            return getattr(self.stream, name)

    try:
        server = http.server.HTTPServer((LOOPBACK, port), Handler)
    except OSError as error:
        if error.errno == errno.EADDRINUSE:
            sys.stderr.write("ポート %d は使用中です\n" % port)
        else:
            sys.stderr.write("地図を配信できません: %s\n" % error)
        return 2
    server.answered = 0
    server.timeout = 0.5

    def stop(*_):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, stop)
    sys.stdout.write("地図を配信しています: http://%s:%d/ （止めるには Ctrl-C）\n" % (
        LOOPBACK, server.server_address[1]))
    sys.stdout.flush()
    try:
        while not requests or server.answered < requests:
            server.handle_request()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


def main():
    parser = argparse.ArgumentParser(description="Every change at a glance.")
    parser.add_argument("--root", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--out")
    parser.add_argument("--find", metavar="WORD")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--backlog", action="store_true",
                        help="also read the backlog through scripts/backlog.py")
    parser.add_argument("--backlog-seconds", type=int, default=BACKLOG_SECONDS)
    parser.add_argument("--serve", action="store_true",
                        help="serve the page on 127.0.0.1, rebuilt on every load")
    parser.add_argument("--port", type=int, default=SERVE_PORT)
    parser.add_argument("--requests", type=int, default=0,
                        help="stop after this many answers (0: until stopped)")
    arguments = parser.parse_args()
    if not 0 <= arguments.port <= 65535 or arguments.requests < 0:
        parser.error("--port は 0 から 65535、--requests は 0 以上です")
    seconds = max(1, min(120, arguments.backlog_seconds))
    root = Path(arguments.root).resolve()
    if not (root / "docs/sdlc/changes").is_dir():
        sys.stderr.write("docs/sdlc/changes がありません: %s\n" % root)
        return 2
    def model_of():
        return build(root, read_backlog(root, seconds) if arguments.backlog else None)

    if arguments.serve:
        return serve(root, arguments.port, arguments.requests, model_of)
    model = model_of()
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


PAGE_DIRECTORY = Path(__file__).resolve().parent / "change-map"


def page_template():
    """The page, put together from its three files beside this script."""
    def part(name):
        return (PAGE_DIRECTORY / name).read_text(encoding="utf-8")
    return part("page.html").replace("__CSS__", part("page.css")).replace("__JS__", part("page.js"))


if __name__ == "__main__":
    sys.exit(main())
