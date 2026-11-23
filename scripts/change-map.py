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
import base64
import datetime
import importlib.util
import json
import math
import os
import re
import subprocess
import sys
import tempfile
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
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


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


def write_page(model, out):
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    page = page_template().replace("__HEADLINE__", headline(model)).replace("__DATA__", data_block(model))
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


PAGE_DIRECTORY = Path(__file__).resolve().parent / "change-map"


def page_template():
    """The page, put together from its three files beside this script."""
    def part(name):
        return (PAGE_DIRECTORY / name).read_text(encoding="utf-8")
    return part("page.html").replace("__CSS__", part("page.css")).replace("__JS__", part("page.js"))


if __name__ == "__main__":
    sys.exit(main())
