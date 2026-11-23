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
import json
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
                "note": s.get("note", ""),
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


def read_requirements(features):
    rows = []
    for feature in features:
        spec = feature.get("spec")
        if not spec:
            continue
        for number, text in behaviour_items(spec["text"]):
            at = text.find("Pinned by")
            tests = re.findall(r"`([^`]+)`", text[at:]) if at >= 0 else []
            said = (text[:at] if at >= 0 else text).strip()
            rows.append({"feature": feature["id"], "domain": feature.get("domain", ""),
                         "number": number, "text": said, "tests": tests})
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
    for feature in layer["features"]:
        feature["terms"] = [t["id"] for t in glossary["terms"] if t["feature"] == feature["id"]]
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
        "requirements": read_requirements(layer["features"]),
    }


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
<title>Operon 製品地図</title>
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

header.top .brand { display: flex; align-items: baseline; gap: 14px; flex-wrap: wrap; }
nav.views { display: inline-flex; gap: 4px; margin-left: 18px; }
nav.views button { border: 1px solid var(--line); background: var(--panel); border-radius: 999px;
  padding: 3px 16px; font-weight: 600; }
nav.views button[aria-pressed="true"] { background: var(--ink); color: var(--bg); border-color: var(--ink); }
.lead { color: var(--soft); margin: 2px 0 14px; max-width: 900px; }
.tiles { display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); margin-top: 18px; }
.tile { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px 16px; }
.tile b { display: block; font-size: 1.9rem; line-height: 1.1; font-variant-numeric: tabular-nums; }
.tile span { color: var(--soft); font-size: .88rem; }
.tile.waiting b { color: var(--needs); } .tile.changing b { color: var(--flight); } .tile.steady b { color: var(--done); }
.badge { display: inline-block; padding: 1px 10px; border-radius: 999px; font-size: .8rem; font-weight: 600;
  white-space: nowrap; background: var(--done-bg); color: var(--done); }
.badge.changing { background: var(--flight-bg); color: var(--flight); }
.badge.waiting { background: var(--needs-bg); color: var(--needs); }
.screen { display: grid; grid-template-columns: minmax(260px, 420px) 1fr; gap: 20px; align-items: start;
  padding: 18px 0; border-top: 1px solid var(--line); }
.screen:first-of-type { border-top: 0; }
.screen.noshot { grid-template-columns: 1fr; }
.screen h3 { margin: 0 0 2px; font-size: 1.12rem; }
.screen .about { color: var(--soft); font-size: .88rem; margin-bottom: 10px; }
.shot { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; overflow: hidden; }
.shot img { display: block; width: 100%; height: auto; }
.shot .none { padding: 38px 16px; color: var(--soft); text-align: center; font-size: .9rem; }
.fcards { display: grid; gap: 10px; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); }
.fcard { text-align: left; background: var(--panel); border: 1px solid var(--line);
  border-top: 4px solid var(--done); border-radius: 10px; padding: 10px 14px 12px; width: 100%; }
.fcard.changing { border-top-color: var(--flight); } .fcard.waiting { border-top-color: var(--needs); }
.fcard:hover { outline: 2px solid var(--accent); outline-offset: 1px; }
.fcard .t { font-weight: 700; display: block; }
.fcard .s { color: var(--soft); font-size: .86rem; display: block; margin: 3px 0 8px; line-height: 1.45; }
.fcard .b { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; font-size: .82rem; color: var(--soft); }
.fview .fhead { margin: 18px 0 6px; }
.fview .back { border: 1px solid var(--line); background: var(--panel); border-radius: 8px; padding: 3px 12px; }
.fview h2 { font-size: 1.5rem; margin: 12px 0 4px; display: flex; gap: 12px; align-items: center; flex-wrap: wrap; }
.fview .meta2 { color: var(--soft); display: flex; gap: 10px; flex-wrap: wrap; align-items: center; }
.fview .tabs { border-bottom: 1px solid var(--line); margin-top: 14px; }
.fview .tabs button { font-size: 1rem; padding: 9px 18px; }
#f-body { padding: 18px 2px 60px; }
.note { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 8px 14px;
  color: var(--soft); font-size: .88rem; margin: 0 0 14px; max-width: 820px; }
ol.timeline { list-style: none; margin: 14px 0; padding: 0 0 0 20px; border-left: 3px solid var(--line); max-width: 860px; }
ol.timeline li { position: relative; padding: 0 0 14px 18px; }
ol.timeline li::before { content: ""; position: absolute; left: -29px; top: 6px; width: 13px; height: 13px;
  border-radius: 50%; background: var(--rest); border: 2px solid var(--bg); }
ol.timeline li.needs::before { background: var(--needs); } ol.timeline li.flight::before { background: var(--flight); }
ol.timeline li.done::before { background: var(--done); }
ol.timeline button { text-align: left; background: var(--panel); border: 1px solid var(--line); border-radius: 10px;
  padding: 8px 12px; width: 100%; display: block; }
ol.timeline button:hover { outline: 2px solid var(--accent); outline-offset: 1px; }
ol.timeline .head { display: flex; gap: 8px; flex-wrap: wrap; align-items: baseline; }
ol.timeline .num { font-weight: 700; font-variant-numeric: tabular-nums; }
ol.timeline .ttl { font-weight: 600; flex: 1 1 260px; }
ol.timeline .how { color: var(--soft); font-size: .8rem; margin-top: 3px; }
.shotbig { max-width: 980px; border: 1px solid var(--line); border-radius: 10px; overflow: hidden; background: var(--panel); }
.shotbig img { display: block; width: 100%; height: auto; }
.paths { color: var(--soft); font-size: .86rem; max-width: 820px; }
.fchips { display: flex; gap: 6px; flex-wrap: wrap; }
.fchip { border: 1px solid var(--line); background: var(--bg); border-radius: 999px; padding: 0 10px; font-size: .84rem; }
.fchip:hover { outline: 2px solid var(--accent); }
.progress { display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 8px; margin-top: 8px; }
.progress .col { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 8px 10px; min-height: 70px; }
.progress .col h4 { margin: 0 0 6px; font-size: .82rem; color: var(--soft); font-weight: 600; }
.progress .col.has { border-color: var(--flight); }
header.top button.basket { border: 1px solid var(--line); background: var(--panel); border-radius: 999px; padding: 3px 14px; }
header.top button.basket b { margin-left: 6px; color: var(--flight); }
.dcards { display: grid; gap: 12px; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); }
.dcard { text-align: left; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 14px 16px; }
.dcard:hover { outline: 2px solid var(--accent); outline-offset: 1px; }
.dcard .t { display: block; font-weight: 700; font-size: 1.08rem; }
.dcard .s { display: block; color: var(--soft); margin: 4px 0 10px; font-size: .9rem; line-height: 1.5; }
.dcard .b { display: flex; gap: 10px; flex-wrap: wrap; align-items: center; font-size: .84rem; color: var(--soft); }
.split2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; max-width: 1100px; }
.split2 .box { max-width: none; margin: 0; }
#v-domain h3, #fl-panel h4 { margin: 22px 0 8px; font-size: 1rem; }
#v-domain .box h3 { margin: 0 0 6px; }
#v-domain h3 small, #fl-panel h4 small { color: var(--soft); font-weight: 400; }
.fwrap { position: relative; }
.fwrap .bk { position: absolute; right: 8px; top: 8px; }
.bk { display: inline-flex; gap: 4px; align-items: center; font-size: .78rem; color: var(--soft); background: var(--panel);
  border: 1px solid var(--line); border-radius: 999px; padding: 0 8px; cursor: pointer; white-space: nowrap; }
.bk input { margin: 0; }
.flow { display: grid; grid-template-columns: minmax(0, 1fr) 360px; gap: 16px; align-items: start; }
.flow .canvas { overflow: auto; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; max-height: 82vh; }
.flow .side { position: sticky; top: 110px; background: var(--panel); border: 1px solid var(--line); border-radius: 12px;
  padding: 4px 16px 16px; max-height: 82vh; overflow: auto; }
.flow .side .shot { margin: 8px 0; }
.flow svg .band { fill: var(--bg); stroke: var(--line); }
.flow svg .bandlabel { fill: var(--soft); font-size: 12px; font-weight: 600; }
.flow svg .w { fill: none; stroke: var(--soft); stroke-opacity: .35; stroke-width: 1.3; }
.flow svg .w.dim { stroke-opacity: .1; }
.flow svg .w.in { stroke: var(--done); stroke-opacity: 1; stroke-width: 2.4; }
.flow svg .w.out { stroke: var(--flight); stroke-opacity: 1; stroke-width: 2.4; }
.flow svg .w.head { fill: var(--soft); fill-opacity: .5; stroke: none; }
.flow svg .w.dim.head { fill-opacity: .12; }
.flow svg .w.in.head { fill: var(--done); fill-opacity: 1; } .flow svg .w.out.head { fill: var(--flight); fill-opacity: 1; }
.flow svg .node { cursor: pointer; }
.flow svg .node rect { fill: var(--panel); stroke: var(--line); stroke-width: 1.4; }
.flow svg .node rect.k-page { stroke: var(--ink); stroke-width: 2; }
.flow svg .node rect.k-sheet, .flow svg .node rect.k-overlay { stroke-dasharray: 5 3; stroke: var(--flight); }
.flow svg .node rect.k-cli { fill: var(--code); }
.flow svg .node:hover rect, .flow svg .node:focus rect { stroke: var(--accent); stroke-width: 2.4; }
.flow svg .node.sel rect { fill: var(--flight-bg); stroke: var(--flight); stroke-width: 3; }
.flow svg .nt { text-anchor: middle; font-size: 12.5px; font-weight: 700; fill: var(--ink); }
.flow svg .nk { text-anchor: middle; font-size: 11px; fill: var(--soft); }
.flow svg .node rect.tag { fill: var(--flight-bg); stroke: var(--flight); stroke-width: 1; stroke-dasharray: none; }
.flow svg .tagt { text-anchor: middle; font-size: 10.5px; fill: var(--flight); font-weight: 600; }
ul.ways { list-style: none; padding: 0; margin: 0; }
ul.ways li { padding: 5px 0; border-top: 1px solid var(--line); }
ul.ways li:first-child { border-top: 0; }
ul.ways .how { color: var(--soft); font-size: .86rem; }
.what { color: var(--soft); font-size: .86rem; }
.terms { display: grid; gap: 10px; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); margin-bottom: 8px; }
.term { background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 10px 14px; }
.term.sel { outline: 3px solid var(--flight); }
.term .th { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; }
.term .tw { font-weight: 700; font-size: 1.1rem; }
.term .en { color: var(--soft); font-size: .86rem; flex: 1; }
.term p { margin: 4px 0 8px; }
.term .kv { grid-template-columns: 120px 1fr; font-size: .88rem; }
.avoid { text-decoration: line-through; color: var(--needs); margin-right: 6px; }
#rq-filters select { font: inherit; padding: 3px 8px; border-radius: 8px; border: 1px solid var(--line); background: var(--panel); color: var(--ink); }
#rq-list td.md { max-width: 720px; }
#basket { position: fixed; top: 0; right: 0; bottom: 0; width: min(620px, 100%); z-index: 11; background: var(--panel);
  box-shadow: var(--shadow); display: flex; flex-direction: column; }
#basket[hidden] { display: none; }
#basket .dhead { padding: 16px 22px 8px; border-bottom: 1px solid var(--line); }
#basket h2 { margin: 0; font-size: 1.12rem; flex: 1; }
#basket .nav button { border: 1px solid var(--line); background: var(--bg); border-radius: 7px; padding: 3px 11px; }
#bk-body { padding: 14px 22px 40px; overflow: auto; }
#bk-body textarea { width: 100%; font: .86rem/1.5 ui-monospace, "SF Mono", Menlo, monospace; background: var(--code); color: var(--ink);
  border: 1px solid var(--line); border-radius: 8px; padding: 10px; margin-top: 8px; }
#bk-body .tools { margin-top: 8px; }
#bk-body button.primary { background: var(--ink); color: var(--bg); border-color: var(--ink); padding: 3px 16px; }
@media (max-width: 1100px) { .flow { grid-template-columns: 1fr; } .flow .side { position: static; } .split2 { grid-template-columns: 1fr; } }
@media (max-width: 900px) { .screen { grid-template-columns: 1fr; } .progress { grid-template-columns: repeat(2, 1fr); } }
@media (max-width: 900px) {
  .area { grid-template-columns: 1fr; gap: 4px; } .track { grid-template-columns: repeat(2, 1fr); }
  main, header.top { padding-left: 14px; padding-right: 14px; }
}
</style>
</head>
<body>
<header class="top">
  <div class="brand"><h1>Operon 製品地図</h1><span class="counts">__HEADLINE__</span>
    <nav class="views" aria-label="表示"><button data-view="product" aria-pressed="true">製品</button><button data-view="domains" aria-pressed="false">ドメイン</button><button data-view="flow" aria-pressed="false">画面の流れ</button><button data-view="glossary" aria-pressed="false">用語集</button><button data-view="requirements" aria-pressed="false">要件</button><button data-view="changes" aria-pressed="false">変更一覧</button></nav></div>
  <div class="row">
    <input id="q" type="search" placeholder="検索: 機能・ドメイン・用語・振る舞い・変更の番号や題" aria-label="検索">
    <button id="bk-open" class="basket">作業かご<b id="bk-n"></b></button>
    <span class="meta" id="meta"></span>
  </div>
</header>
<main>
  <div id="v-product">
    <p class="lead">Operon を画面ごとに見渡します。機能を選ぶと、何をするものか・今できること・これまでの変更を読めます。</p>
    <div id="p-catalogue-error"></div>
    <section id="p-needs"><h2>あなたの判断を待っています <small id="p-needs-n"></small></h2><div id="p-needs-list"></div></section>
    <div class="tiles" id="p-tiles"></div>
    <section id="p-flight"><h2>いま進めている変更 <small>どの段階にあるか</small></h2><div id="p-flight-list"></div></section>
    <section id="p-screens"><h2>画面と機能 <small id="p-screens-n"></small></h2><div id="p-screen-list"></div></section>
    <section id="p-unmapped"></section>
  </div>
  <div id="v-feature" class="fview" hidden>
    <div class="fhead"><button class="back" data-view="product">← 製品に戻る</button>
      <h2><span id="f-title"></span><span id="f-badge"></span></h2>
      <div class="meta2" id="f-meta"></div></div>
    <div class="tabs" id="f-tabs" role="tablist"></div>
    <div id="f-body"></div>
  </div>
  <div id="v-domains" hidden>
    <p class="lead">Operon を責任の範囲（ドメイン）で分けて見渡します。どのコードと機能がどこに属し、どこで隣と受け渡すかが分かります。</p>
    <div id="dm-error"></div>
    <div class="dcards" id="dm-list"></div>
  </div>
  <div id="v-domain" class="fview" hidden>
    <div class="fhead"><button class="back" data-view="domains">← ドメインに戻る</button>
      <h2 id="dd-title"></h2><div class="meta2" id="dd-summary"></div></div>
    <div id="dd-body"></div>
  </div>
  <div id="v-flow" hidden>
    <p class="lead">画面・タブ・シート・重ねて開くものと、そのあいだの道（キーとボタン）です。画面を選ぶと、入る道（緑）と出る道（青）が色付きになります。どのページからも開けるもの（⌘K など）は「どこからでも」と示し、矢印はその画面を選んだときだけ描きます。</p>
    <div id="fl-error"></div>
    <div class="flow"><div class="canvas" id="fl-canvas"></div><div class="side" id="fl-panel"></div></div>
  </div>
  <div id="v-glossary" hidden>
    <p class="lead">Operon が使う言葉と、その意味・画面での表記・コードでの名前・使わない言葉です。新しい文言はここの言葉で書きます。</p>
    <div id="gl-error"></div>
    <section><h2>用語 <small id="gl-n"></small></h2><div id="gl-list"></div></section>
  </div>
  <div id="v-requirements" hidden>
    <p class="lead">すべての機能の「今の振る舞い」を 1 行ずつ並べ、どのテストがそれを確かめているかを示します。テストの記録がない行が、確かめられていない振る舞いです。</p>
    <section><h2>振る舞い <small id="rq-n"></small></h2><div class="filters" id="rq-filters"></div><div id="rq-list"></div></section>
  </div>
  <div id="v-changes" hidden>
    <section id="needs"><h2>あなたの判断を待っています <small id="needs-n"></small></h2><div id="needs-list"></div></section>
    <section id="flight"><h2>進行中 <small id="flight-n"></small></h2><div id="flight-list"></div></section>
    <section id="areas"><h2>領域ごとの変更 <small>どの部分に、これまで何をしてきたか</small></h2><div id="area-list"></div></section>
    <section id="all"><h2>すべての変更 <small id="all-n"></small></h2>
      <div class="filters" id="filters"></div><div id="all-list"></div></section>
  </div>
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
<aside id="basket" hidden aria-label="作業かご">
  <div class="dhead"><div class="dtitle"><h2>作業かご</h2><div class="nav"><button id="bk-close">閉じる</button></div></div>
    <p class="what">集めたものを 1 つの指示にして、CLI のセッションに貼り付けます。</p></div>
  <div id="bk-body"></div>
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
const BADGE = { steady: "安定", changing: "変更中", waiting: "判断待ち" };
const ROUTE = { feature: "新機能", modify: "変更", bugfix: "不具合の修正", refactor: "整理",
  security: "安全性", docs: "文書", trivial: "軽微な修正" };
const LINK = { recorded: "記録", inferred: "計画のファイルから推定", estimated: "題から推定" };
const FTABS = [["brief", "BRIEF", "人向けの説明"], ["spec", "SPEC", "AI 向けの仕様"], ["progress", "進捗", "変更の履歴"], ["screen", "画面", "画面とマニュアル"], ["terms", "用語", "この機能の言葉"]];
const FEATURES = M.features;
const featureById = new Map(FEATURES.map(f => [f.id, f]));
const screenLabel = id => { const s = M.screens.find(x => x.id === id); return s ? s.label : id; };
const routeWord = c => ROUTE[c.route] || c.route;
const stageWord = s => STAGE[s] || s;
const byName = new Map(M.changes.map(c => [c.name, c]));
const $ = id => document.getElementById(id);

function esc(text) {
  return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}
function statusWord(c) { return c.problem ? "読めない記録" : (STATUS[c.status] || c.status); }
function pill(c) { return '<span class="pill ' + c.group + '">' + esc(statusWord(c)) + "</span>"; }
function place(c) { return c.where.branch || (c.where.here ? "" : c.where.path); }
function featuresOf(c) {
  const titles = (c.features || []).map(l => (featureById.get(l.id) || { title: l.id }).title);
  if (titles.length) return titles.slice(0, 3).join("、") + (titles.length > 3 ? " ほか" : "");
  return c.features_named.trim() === "none" ? "（製品の機能ではありません）" : "（機能未設定）";
}
function areaLabel(name) { const a = M.areas.find(x => x.name === name); return a ? a.label : name; }
function haystack(c) {
  return [c.name, c.title, statusWord(c), c.status, c.route, STAGE[c.stage] || "", c.stage,
    c.areas.map(areaLabel).join(" "), c.areas.join(" "), place(c),
    (c.features || []).map(l => (featureById.get(l.id) || {}).title + " " + l.id).join(" ")].join(" ").toLowerCase();
}

function card(c, small) {
  let h = '<button class="card ' + c.group + '" data-open="' + esc(c.name) + '"><div class="head">' +
    '<span class="num">' + esc(c.number) + '</span><span class="title">' + esc(c.title) + "</span>" + pill(c) + "</div>";
  const sub = [STAGE[c.stage] || c.stage, routeWord(c), place(c)].filter(Boolean).join(" · ");
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
    "<th>段階</th><th>種類</th><th>機能</th></tr></thead><tbody>" + listed.map(c =>
      '<tr class="pick" data-open="' + esc(c.name) + '"><td class="num">' + esc(c.number) + "</td><td>" +
      esc(c.title) + "</td><td>" + pill(c) + "</td><td>" + esc(STAGE[c.stage] || c.stage) + "</td><td>" +
      esc(routeWord(c)) + "</td><td>" + esc(featuresOf(c)) + "</td></tr>").join("") +
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
  // No `[` inside the label: the search then restarts at the next bracket
  // instead of rescanning to the end from every one (quadratic on a wall of them).
  s = s.replace(/\[([^\]\[]+)\]\((https?:\/\/[^\s)]+)\)/g,
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

function featureChips(c) {
  const links = c.features || [];
  if (!links.length) return esc(featuresOf(c));
  return '<span class="fchips">' + links.map(l => '<button class="fchip" data-feature="' + esc(l.id) + '" title="' +
    esc(LINK[l.link]) + '">' + esc((featureById.get(l.id) || { title: l.id }).title) + (l.link === "recorded" ? "" : "（推定）") +
    "</button>").join("") + "</span>";
}
function overview(c) {
  let h = "";
  if (c.problem) h += '<div class="box needs"><h3>記録を読めません</h3>' + esc(c.problem) + "</div>";
  else if (c.group === "needs") h += '<div class="box needs"><h3>あなたの判断を待っています</h3>' + esc(c.resume) + "</div>";
  else if (c.resume) h += '<div class="box"><h3>' + (c.group === "flight" ? "次にすること" : "結果") + "</h3>" + esc(c.resume) + "</div>";
  const used = v => { const p = String(v).split(" "); return p.length === 2 ? p[0] + " / " + p[1] : v; };
  h += '<dl class="kv"><dt>状態</dt><dd>' + pill(c) + " <code>" + esc(c.status) + "</code></dd>" +
    "<dt>段階</dt><dd>" + esc(STAGE[c.stage] || c.stage) + " <code>" + esc(c.stage) + "</code></dd>" +
    "<dt>種類</dt><dd>" + esc(routeWord(c)) + " <code>" + esc(c.route) + "</code></dd>" +
    "<dt>機能</dt><dd>" + featureChips(c) + "</dd>" +
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
  $("d-facts").innerHTML = pill(c) + "<span>" + esc([routeWord(c), place(c)].filter(Boolean).join(" · ")) + "</span>";
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
    let rendered = null;
    // A document the renderer cannot take (nesting past the stack) is still
    // shown, as written.
    if (isMd && !rawView) { try { rendered = markdown(d.text); } catch (error) { rendered = null; } }
    h += rendered !== null ? '<div class="md">' + rendered + "</div>" : '<pre class="raw">' + esc(d.text) + "</pre>";
    body.innerHTML = h;
  }
  body.scrollTop = 0;
  const at2 = listed.indexOf(c);
  $("d-prev").disabled = at2 <= 0;
  $("d-next").disabled = at2 < 0 || at2 >= listed.length - 1;
}

let view = "product";
let feature = null;
let featureTab = "brief";
let baseHash = "";

function setHash(hash, replace) {
  if (location.hash === hash) { route(); return; }
  if (replace) { history.replaceState(null, "", hash || location.pathname + location.search); route(); }
  else location.hash = hash;
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
  history.replaceState(null, "", baseHash || location.pathname + location.search);
}

function renderFeatureBadge(f) {
  const stage = f.stage ? " · " + stageWord(f.stage) : "";
  return '<span class="badge ' + f.badge + '">' + BADGE[f.badge] + esc(f.badge === "steady" ? "" : stage) + "</span>";
}
function featureCard(f) {
  const open = f.changes.filter(l => l.link !== "estimated" && ["needs", "flight"].includes(byName.get(l.name).group));
  return '<button class="fcard ' + f.badge + '" data-feature="' + esc(f.id) + '"><span class="t">' + esc(f.title) +
    '</span><span class="s">' + esc(f.summary) + '</span><span class="b">' + renderFeatureBadge(f) +
    (open.length ? "<span>変更 " + esc(open.map(l => byName.get(l.name).number).join("・")) + "</span>" : "") + "</span></button>";
}
function featureMatches(f) {
  if (!query) return true;
  return [f.title, f.id, f.summary, screenLabel(f.screen), BADGE[f.badge]].join(" ").toLowerCase().includes(query);
}

function renderProduct() {
  $("p-catalogue-error").innerHTML = M.catalogue_error ? '<div class="box needs"><h3>' + esc(M.catalogue_error) +
    "</h3>製品の見方は出せません。変更一覧は読めます。</div>" : "";
  const needs = M.changes.filter(c => c.group === "needs");
  $("p-needs-n").textContent = "(" + needs.length + ")";
  $("p-needs-list").innerHTML = needs.length ? '<div class="cards">' + needs.map(c => card(c)).join("") + "</div>"
    : '<div class="empty">判断を待っている変更はありません</div>';
  const count = b => FEATURES.filter(f => f.badge === b).length;
  $("p-tiles").innerHTML = FEATURES.length ? ["waiting", "changing", "steady"].map(b => '<div class="tile ' + b + '"><b>' + count(b) +
    "</b><span>" + BADGE[b] + "の機能" + (b === "waiting" ? "（あなたの判断を待つ変更がある）" : b === "changing" ? "（変更が進んでいる）" : "（いま動いている変更はない）") +
    "</span></div>").join("") : "";
  const flight = M.changes.filter(c => c.group === "flight");
  $("p-flight-list").innerHTML = !flight.length ? '<div class="empty">進行中の変更はありません</div>' :
    '<div class="progress">' + M.stages.map(s => {
      const here = flight.filter(c => c.stage === s);
      return '<div class="col' + (here.length ? " has" : "") + '"><h4>' + stageWord(s) + "</h4>" +
        here.map(c => card(c, true)).join("") + "</div>";
    }).join("") + "</div>";
  const shown = M.screens.map(sc => ({ sc, fs: FEATURES.filter(f => f.screen === sc.id && featureMatches(f)) })).filter(x => x.fs.length);
  $("p-screens-n").textContent = FEATURES.length ? "(" + FEATURES.length + " 機能)" : "";
  $("p-screen-list").innerHTML = shown.length ? shown.map(({ sc, fs }) =>
    '<div class="screen' + (sc.image && M.images[sc.image] ? "" : " noshot") + '">' + (sc.image && M.images[sc.image] ? '<div class="shot" data-shot="' + esc(sc.id) + '"></div>' : "") + '<div><h3>' + esc(sc.label) +
    ' <small style="font-weight:400;color:var(--soft)">' + fs.length + ' 機能</small></h3><div class="fcards">' +
    fs.map(wrapCard).join("") + "</div></div></div>").join("") :
    (FEATURES.length ? '<div class="empty">該当する機能はありません</div>' : '<div class="empty">機能の一覧がありません</div>');
  for (const holder of document.querySelectorAll("[data-shot]")) {
    const sc = M.screens.find(x => x.id === holder.dataset.shot);
    if (sc && sc.image && M.images[sc.image]) {
      const picture = document.createElement("img");
      picture.src = M.images[sc.image]; picture.alt = sc.label + "の画面"; picture.loading = "lazy";
      holder.appendChild(picture);
    } else holder.innerHTML = '<div class="none">この画面の写真はありません</div>';
  }
  const loose = M.unmapped.length;
  $("p-unmapped").innerHTML = loose ? '<p class="lead">' + (M.changes.length - loose) + " 件の変更は機能に結び付いています。残りの " + loose +
    " 件は、開発の進め方や道具の変更など、機能に紐づかないもの（または記録に機能名がない古い変更）です。" +
    ' <button class="fchip" data-view="changes">変更一覧で見る</button></p>' : "";
  if (M.features_omitted.length) $("p-unmapped").innerHTML += '<p class="warn">機能が多すぎるため、載せていません: ' + esc(M.features_omitted.join(", ")) + "</p>";
}

function featureDoc(f, which) {
  const d = f[which];
  if (!d) return '<div class="box needs"><h3>文書がありません</h3>' + esc(which === "brief" ? "brief.md" : "spec.md") + " が見つかりません。</div>";
  let out = d.truncated ? '<p class="warn">長いため途中まで表示しています</p>' : "";
  let rendered = null;
  try { rendered = markdown(d.text); } catch (error) { rendered = null; }
  return out + (rendered !== null ? '<div class="md">' + rendered + "</div>" : '<pre class="raw">' + esc(d.text) + "</pre>");
}
function featureProgress(f) {
  const link = k => f.changes.filter(l => l.link === k).length;
  let h = '<div class="box ' + (f.badge === "waiting" ? "needs" : "") + '"><h3>いまの状態: ' + renderFeatureBadge(f) + "</h3>" +
    (f.badge === "steady" ? "この機能に手を入れている変更は、いまありません。" :
     f.badge === "changing" ? "この機能に手を入れている変更が進んでいます。" : "この機能に触れる変更が、あなたの判断を待っています。") + "</div>";
  if (!f.changes.length) return h + '<p class="empty">この機能に触れた変更の記録はありません</p>';
  h += "<p>これまでにこの機能へ触れた変更 " + f.changes.length + " 件（記録 " + link("recorded") + " · 計画のファイルから推定 " +
    link("inferred") + " · 題から推定 " + link("estimated") + "）。新しい順です。</p>";
  h += '<div class="note">「推定」は、変更の記録に機能名がないため、計画に書かれたファイルや題の言葉から割り出したものです。外れていることがあります。' +
    "機能名の記録は、変更 313 以降の変更に必ず付きます。</div>";
  h += '<ol class="timeline">' + f.changes.map(l => {
    const c = byName.get(l.name);
    return '<li class="' + c.group + '"><button data-open="' + esc(c.name) + '"><div class="head"><span class="num">' + esc(c.number) +
      '</span><span class="ttl">' + esc(c.title) + "</span>" + pill(c) + "<span>" + esc(stageWord(c.stage)) + "</span></div>" +
      '<div class="how">' + esc(LINK[l.link]) + " · " + esc(routeWord(c)) + (place(c) ? " · " + esc(place(c)) : "") + "</div></button></li>";
  }).join("") + "</ol>";
  return h;
}
function featureScreen(f) {
  let h = "";
  if (f.image && M.images[f.image]) h += '<div class="shotbig" id="f-shot"></div>';
  else h += '<p class="empty">この機能を写した画面写真はありません（画面は「' + esc(screenLabel(f.screen)) + "」です）</p>";
  h += "<h3>マニュアルの説明: " + esc(f.manual) + "</h3>";
  if (f.manual_text) { let m = null; try { m = markdown(f.manual_text); } catch (error) { m = null; } h += m !== null ? '<div class="md">' + m + "</div>" : '<pre class="raw">' + esc(f.manual_text) + "</pre>"; }
  else h += '<p class="empty">マニュアルに該当する節がありません</p>';
  h += '<p class="paths">この機能を実現しているコード: ' + f.paths.map(p => "<code>" + esc(p) + "</code>").join(" ") + "</p>";
  return h;
}
function featureTerms(f) {
  const own = TERMS.filter(t => t.feature === f.id);
  const used = TERMS.filter(t => t.feature !== f.id && t.used_by.includes(f.id));
  const where = FLOW.screens.filter(s => s.features.includes(f.id));
  return "<h3>この機能の用語 <small>" + own.length + '</small></h3><div class="terms">' + (own.map(termCard).join("") || '<p class="empty">なし</p>') + "</div>" +
    "<h3>BRIEF が使っているほかの用語</h3>" + termChips(used.map(t => t.id)) +
    "<h3>この機能が載っている画面</h3>" + (where.length ? '<span class="fchips">' + where.map(s => chip("data-screen", s.id, s.title)).join("") + "</span>" : '<p class="empty">なし</p>') +
    '<h3>振る舞い</h3><p><button class="fchip" data-reqs="' + esc(f.id) + '">要件の表でこの機能の ' + REQS.filter(r => r.feature === f.id).length + " 行を見る</button></p>";
}
function renderFeature() {
  const f = feature;
  $("f-title").textContent = f.title;
  $("f-badge").innerHTML = renderFeatureBadge(f);
  $("f-meta").innerHTML = "<span>画面: <b>" + esc(screenLabel(f.screen)) + "</b></span><span>ドメイン: " + domainChip(f.domain) +
    "</span>" + basketBox("feature", f.id) + "<span>" + esc(f.summary) + "</span>";
  $("f-tabs").innerHTML = FTABS.map(t => '<button role="tab" data-ftab="' + t[0] + '" aria-selected="' + (t[0] === featureTab) +
    '" title="' + esc(t[2]) + '">' + t[1] + (t[0] === "progress" && f.changes.length ? " " + f.changes.length : "") + "</button>").join("");
  const body = $("f-body");
  if (featureTab === "brief") body.innerHTML = featureDoc(f, "brief");
  else if (featureTab === "spec") body.innerHTML = '<div class="note">AI が従い、最新に保つための仕様です（英語）。人が読む説明は BRIEF にあります。</div>' + featureDoc(f, "spec");
  else if (featureTab === "progress") body.innerHTML = featureProgress(f);
  else if (featureTab === "terms") body.innerHTML = featureTerms(f);
  else {
    body.innerHTML = featureScreen(f);
    const holder = $("f-shot");
    if (holder) { const picture = document.createElement("img"); picture.src = M.images[f.image]; picture.alt = f.title + "の画面"; holder.appendChild(picture); }
  }
}

// Change 315: domains, the screen flow, the glossary, the requirement board
// and the work basket. Every value from the model passes through esc().
const DOMAINS = M.domains || [];
const TERMS = M.terms || [];
const FLOW = M.flow || { screens: [], ways: [], bands: [], box: [168, 58], width: 0, height: 0 };
const REQS = M.requirements || [];
const domainById = new Map(DOMAINS.map(d => [d.id, d]));
const termById = new Map(TERMS.map(t => [t.id, t]));
const flowById = new Map(FLOW.screens.map(s => [s.id, s]));
const KIND = { page: "ページ", tab: "タブ", panel: "パネル", sheet: "シート", overlay: "重ねて開く", cli: "コマンドライン" };
const WORK = [["modify", "変更", "既存の振る舞いを変える（route modify）"],
  ["bugfix", "不具合", "不具合を直す（route bugfix。root-cause から）"],
  ["docs", "文書", "文書を直す（route docs）"],
  ["test", "テスト", "テストのない振る舞いにテストを足す"]];
const domainTitle = id => (domainById.get(id) || { title: id || "（ドメイン未設定）" }).title;
const featureTitle = id => (featureById.get(id) || { title: id }).title;
const termWord = id => (termById.get(id) || { term: id }).term;

let domainSel = null;
let screenSel = null;
let termSel = null;
let reqFeature = "";
let reqDomain = "";
let reqTests = "";
const basket = new Map();
let workType = "modify";
let basketOpen = false;

function errorBox(error, notice) {
  return (error ? '<div class="box needs"><h3>' + esc(error) + "</h3>ほかの見方はそのまま使えます。</div>" : "") +
    (notice ? '<p class="warn">' + esc(notice) + "</p>" : "");
}
function basketBox(kind, id) {
  const key = kind + "|" + id;
  return '<label class="bk" title="作業かごに入れる"><input type="checkbox" data-basket="' + esc(key) + '"' +
    (basket.has(key) ? " checked" : "") + "> かご</label>";
}
function chip(attr, id, label) {
  return '<button class="fchip" ' + attr + '="' + esc(id) + '">' + esc(label) + "</button>";
}
function domainChip(id) { return id ? chip("data-domain", id, domainTitle(id)) : ""; }
function termChips(ids) {
  return ids.length ? '<span class="fchips">' + ids.map(id => chip("data-term", id, termWord(id))).join("") + "</span>"
    : '<span class="empty">なし</span>';
}
function featureChipsOf(ids) {
  return ids.length ? '<span class="fchips">' + ids.map(id => chip("data-feature", id, featureTitle(id))).join("") + "</span>"
    : '<span class="empty">なし</span>';
}
function wrapCard(f) { return '<div class="fwrap">' + featureCard(f) + basketBox("feature", f.id) + "</div>"; }

function renderDomains() {
  $("dm-error").innerHTML = errorBox(M.domains_error, M.domains_notice);
  const shown = DOMAINS.filter(d => !query || [d.title, d.id, d.summary, d.owns].join(" ").toLowerCase().includes(query));
  $("dm-list").innerHTML = shown.length ? shown.map(d => {
    const fs = d.features.map(id => featureById.get(id)).filter(Boolean);
    const moving = fs.filter(f => f.badge !== "steady").length;
    const terms = TERMS.filter(t => t.domain === d.id).length;
    return '<button class="dcard" data-domain="' + esc(d.id) + '"><span class="t">' + esc(d.title) + '</span><span class="s">' +
      esc(d.summary) + '</span><span class="b"><span>機能 ' + fs.length + "</span><span>コード " + d.modules.length +
      "</span><span>用語 " + terms + "</span><span>変更 " + d.changes.length + "</span>" +
      (moving ? '<span class="badge changing">動いている機能 ' + moving + "</span>" : '<span class="badge">安定</span>') +
      "</span></button>";
  }).join("") : (DOMAINS.length ? '<div class="empty">該当するドメインはありません</div>' : "");
}

function renderDomain() {
  const d = domainSel;
  $("dd-title").textContent = d.title;
  $("dd-summary").innerHTML = "<span>" + esc(d.summary) + "</span>" + basketBox("domain", d.id);
  let h = '<div class="split2"><div class="box"><h3>担当すること</h3>' + esc(d.owns) + '</div><div class="box"><h3>担当しないこと</h3>' +
    esc(d.excludes) + "</div></div>";
  h += "<h3>隣のドメイン</h3>" + (d.adjacent.length ? '<span class="fchips">' + d.adjacent.map(domainChip).join("") + "</span>" : '<p class="empty">なし</p>');
  h += "<h3>機能 <small>" + d.features.length + "</small></h3>" +
    '<div class="fcards">' + d.features.map(id => featureById.get(id)).filter(Boolean).map(wrapCard).join("") + "</div>";
  h += "<h3>コード <small>" + d.modules.length + ' ファイル</small></h3><table class="all"><tbody>' + d.modules.map(m =>
    "<tr><td><code>" + esc(m.path) + "</code></td><td>" + esc(m.about) + "</td></tr>").join("") + "</tbody></table>";
  h += "<h3>用語</h3>" + termChips(TERMS.filter(t => t.domain === d.id).map(t => t.id));
  h += "<h3>変更 <small>" + d.changes.length + "</small></h3>";
  h += d.changes.length ? '<ol class="timeline">' + d.changes.slice(0, 60).map(name => {
    const c = byName.get(name);
    return '<li class="' + c.group + '"><button data-open="' + esc(c.name) + '"><div class="head"><span class="num">' + esc(c.number) +
      '</span><span class="ttl">' + esc(c.title) + "</span>" + pill(c) + "</div></button></li>";
  }).join("") + "</ol>" + (d.changes.length > 60 ? '<p class="empty">ほか ' + (d.changes.length - 60) + " 件</p>" : "")
    : '<p class="empty">このドメインの機能に触れた変更の記録はありません</p>';
  $("dd-body").innerHTML = h;
}

const SVG = "http://www.w3.org/2000/svg";
function svgNode(name, attrs, text) {
  const node = document.createElementNS(SVG, name);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, String(v));
  if (text !== undefined) node.textContent = text;
  return node;
}
function wayHow(w) {
  const parts = [];
  if (w.chord) parts.push(w.chord);
  if (w.label) parts.push("「" + w.label + "」");
  if (w.click) parts.push(w.click);
  return parts.join(" / ");
}
// A way that every page offers (a page key, ⌘K, ⌘N) is one fact repeated
// from each page; drawn once each it buries the rest, so it is marked on its
// target as どこからでも and drawn only for the chosen box.
const PAGES = FLOW.screens.filter(s => s.kind === "page").map(s => s.id);
const GLOBAL = new Set();
const EVERYWHERE = new Map();
(function findEverywhere() {
  const groups = new Map();
  FLOW.ways.forEach((w, i) => {
    if (!w.key) return;
    const sig = w.to + "|" + w.key;
    if (!groups.has(sig)) groups.set(sig, []);
    groups.get(sig).push(i);
  });
  for (const [sig, list] of groups) {
    const from = new Set(list.map(i => FLOW.ways[i].from));
    const to = sig.split("|")[0];
    if (PAGES.length > 1 && PAGES.every(p => p === to || from.has(p))) {
      list.forEach(i => GLOBAL.add(i));
      if (!EVERYWHERE.has(to)) EVERYWHERE.set(to, []);
      EVERYWHERE.get(to).push(FLOW.ways[list[0]].chord || FLOW.ways[list[0]].key);
    }
  }
})();
function drawFlow() {
  const holder = $("fl-canvas");
  holder.textContent = "";
  if (!FLOW.screens.length) return;
  const [bw, bh] = FLOW.box;
  const svg = svgNode("svg", { width: FLOW.width, height: FLOW.height, viewBox: "0 0 " + FLOW.width + " " + FLOW.height,
    role: "img", "aria-label": "画面の流れ" });
  FLOW.bands.forEach(band => {
    svg.appendChild(svgNode("rect", { x: 4, y: band.y - 12, width: FLOW.width - 8, height: band.height + 24, rx: 12, class: "band" }));
    svg.appendChild(svgNode("text", { x: 16, y: band.y + 22, class: "bandlabel" }, screenLabel(band.id)));
  });
  const sel = screenSel ? screenSel.id : "";
  const plain = svgNode("g", {});
  const lit = svgNode("g", {});
  FLOW.ways.forEach((w, index) => {
    const a = flowById.get(w.from), b = flowById.get(w.to);
    if (!a || !b) return;
    const role = w.from === sel ? "out" : w.to === sel ? "in" : "";
    if (GLOBAL.has(index) && !role) return;
    // Down and up between the same two boxes leave and arrive at different
    // points, and a way that skips a band bulges to the side of the boxes it passes.
    const down = b.y > a.y, same = a.y === b.y;
    const x1 = a.x + bw * (same ? 0.5 : down ? 0.38 : 0.62), x2 = b.x + bw * (same ? 0.5 : down ? 0.38 : 0.62);
    const sy = same ? a.y : a.y + (down ? bh : 0), ey = same ? b.y : b.y + (down ? 0 : bh);
    const skip = !same && Math.abs(b.y - a.y) > bh + 80 && Math.abs(x2 - x1) < bw;
    const side = skip ? (down ? 1 : -1) * (bw * 0.62 + 6 * (FLOW.ways.indexOf(w) % 4)) : 0;
    const bend = same ? -34 - Math.abs(x2 - x1) / 10 : (ey - sy) / 2;
    const c1x = x1 + side, c1y = sy + bend, c2x = x2 + side, c2y = same ? ey + bend : ey - bend;
    const sx = x1, ex = x2;
    const d = "M" + sx + "," + sy + " C" + c1x + "," + c1y + " " + c2x + "," + c2y + " " + ex + "," + ey;
    const cls = "w " + role + (sel && !role ? " dim" : "");
    const group = svgNode("g", {});
    group.appendChild(svgNode("path", { d, class: cls }));
    // The arrowhead, drawn along the curve's last tangent.
    const len = Math.hypot(ex - c2x, ey - c2y) || 1, ux = (ex - c2x) / len, uy = (ey - c2y) / len;
    const size = role ? 9 : 7;
    const p1 = (ex - ux * size - uy * size * 0.55) + "," + (ey - uy * size + ux * size * 0.55);
    const p2 = (ex - ux * size + uy * size * 0.55) + "," + (ey - uy * size - ux * size * 0.55);
    group.appendChild(svgNode("path", { d: "M" + ex + "," + ey + " L" + p1 + " L" + p2 + " z", class: cls + " head" }));
    group.appendChild(svgNode("title", {}, a.title + " → " + b.title + "  " + wayHow(w)));
    (role ? lit : plain).appendChild(group);
  });
  svg.appendChild(plain);
  svg.appendChild(lit);
  FLOW.screens.forEach(s => {
    const g = svgNode("g", { class: "node" + (s.id === sel ? " sel" : ""), "data-screen": s.id, tabindex: 0 });
    g.appendChild(svgNode("rect", { x: s.x, y: s.y, width: bw, height: bh, rx: 10, class: "k-" + s.kind }));
    g.appendChild(svgNode("text", { x: s.x + bw / 2, y: s.y + 23, class: "nt" }, s.title.length > 11 ? s.title.slice(0, 10) + "…" : s.title));
    g.appendChild(svgNode("text", { x: s.x + bw / 2, y: s.y + 41, class: "nk" }, KIND[s.kind] || s.kind));
    g.appendChild(svgNode("title", {}, s.title));
    if (EVERYWHERE.has(s.id)) {
      const tag = "どこからでも " + EVERYWHERE.get(s.id).join(" ");
      const width = 16 + tag.length * 9;
      g.appendChild(svgNode("rect", { x: s.x + bw - width + 6, y: s.y - 10, width, height: 18, rx: 9, class: "tag" }));
      g.appendChild(svgNode("text", { x: s.x + bw - width / 2 + 6, y: s.y + 3, class: "tagt" }, tag));
    }
    svg.appendChild(g);
  });
  holder.appendChild(svg);
}
function screenPanel() {
  const s = screenSel;
  if (!s) return '<p class="empty">画面を選ぶと、そこに載っている機能と、そこへ入る道・そこから出る道が出ます。</p>';
  const line = i => {
    const w = FLOW.ways[i];
    const other = flowById.get(w.from === s.id ? w.to : w.from);
    return "<li>" + chip("data-screen", other.id, other.title) + ' <span class="how">' + esc(wayHow(w)) + "</span>" +
      (w.note ? '<div class="what">' + esc(w.note) + "</div>" : "") + "</li>";
  };
  let h = "<h3>" + esc(s.title) + ' <small>' + esc(KIND[s.kind] || s.kind) + " · " + esc(screenLabel(s.screen)) + "</small></h3>" +
    basketBox("screen", s.id);
  if (s.note) h += "<p>" + esc(s.note) + "</p>";
  if (s.parent) h += "<p>中にある場所: " + chip("data-screen", s.parent, (flowById.get(s.parent) || { title: s.parent }).title) + "</p>";
  if (s.image && M.images[s.image]) h += '<div class="shot" id="fl-shot"></div>';
  h += "<h4>機能</h4>" + featureChipsOf(s.features);
  h += "<h4>入ってくる道 <small>" + s.in.length + "</small></h4>" + (s.in.length ? '<ul class="ways">' + s.in.map(line).join("") + "</ul>" : '<p class="empty">なし</p>');
  h += "<h4>出ていく道 <small>" + s.out.length + "</small></h4>" + (s.out.length ? '<ul class="ways">' + s.out.map(line).join("") + "</ul>" : '<p class="empty">なし</p>');
  h += "<h4>用語</h4>" + termChips(s.terms);
  return h;
}
function renderFlow() {
  $("fl-error").innerHTML = errorBox(M.flow_error, M.flow_notice);
  drawFlow();
  $("fl-panel").innerHTML = screenPanel();
  const holder = $("fl-shot");
  if (holder && screenSel) {
    const picture = document.createElement("img");
    picture.src = M.images[screenSel.image]; picture.alt = screenSel.title + "の画面";
    holder.appendChild(picture);
  }
}

function termCard(t) {
  const row = (label, value) => value ? "<dt>" + label + "</dt><dd>" + value + "</dd>" : "";
  return '<div class="term' + (termSel && termSel.id === t.id ? " sel" : "") + '" id="term-' + esc(t.id) + '"><div class="th"><span class="tw">' + esc(t.term) + "</span>" +
    (t.en ? '<span class="en">' + esc(t.en) + "</span>" : "") + basketBox("term", t.id) + "</div><p>" + esc(t.meaning) + '</p><dl class="kv">' +
    row("画面での表記", t.ui.map(u => "「" + esc(u) + "」").join(" ")) +
    row("コード", t.code.map(c => "<code>" + esc(c) + "</code>").join(" ")) +
    row("使わない言葉", t.avoid.map(a => '<span class="avoid">' + esc(a) + "</span>").join(" ")) +
    row("関連語", t.see.length ? termChips(t.see) : "") +
    row("機能", chip("data-feature", t.feature, featureTitle(t.feature)) + " " + domainChip(t.domain)) +
    row("使っている BRIEF", t.used_by.length ? featureChipsOf(t.used_by) : "") + "</dl></div>";
}
function termMatches(t) {
  return !query || [t.term, t.id, t.en, t.meaning, t.ui.join(" "), t.code.join(" "), t.avoid.join(" ")].join(" ").toLowerCase().includes(query);
}
function renderGlossary() {
  $("gl-error").innerHTML = errorBox(M.terms_error, M.terms_notice);
  const shown = TERMS.filter(termMatches);
  $("gl-n").textContent = "(" + shown.length + " / " + TERMS.length + ")";
  if (!shown.length) { $("gl-list").innerHTML = TERMS.length ? '<div class="empty">該当する用語はありません</div>' : ""; return; }
  const order = DOMAINS.map(d => d.id).concat([""]);
  const groups = order.map(id => ({ id, terms: shown.filter(t => (domainById.has(t.domain) ? t.domain : "") === id) })).filter(g => g.terms.length);
  $("gl-list").innerHTML = groups.map(g => "<h3>" + esc(g.id ? domainTitle(g.id) : "ドメイン未設定") + " <small>" + g.terms.length + '</small></h3><div class="terms">' +
    g.terms.map(termCard).join("") + "</div>").join("");
  if (termSel) { const at = $("term-" + termSel.id); if (at) at.scrollIntoView({ block: "center" }); }
}

function renderRequirements() {
  const opt = (value, label, current) => '<option value="' + esc(value) + '"' + (value === current ? " selected" : "") + ">" + esc(label) + "</option>";
  $("rq-filters").innerHTML = '<select id="rq-domain" aria-label="ドメイン">' + opt("", "すべてのドメイン", reqDomain) + DOMAINS.map(d => opt(d.id, d.title, reqDomain)).join("") + "</select>" +
    '<select id="rq-feature" aria-label="機能">' + opt("", "すべての機能", reqFeature) + FEATURES.filter(f => !reqDomain || f.domain === reqDomain).map(f => opt(f.id, f.title, reqFeature)).join("") + "</select>" +
    '<select id="rq-tests" aria-label="テスト">' + opt("", "テストの有無を問わない", reqTests) + opt("yes", "テストの記録あり", reqTests) + opt("no", "テストの記録なし", reqTests) + "</select>";
  const rows = REQS.filter(r => (!reqDomain || r.domain === reqDomain) && (!reqFeature || r.feature === reqFeature) &&
    (!reqTests || (reqTests === "yes") === (r.tests.length > 0)) &&
    (!query || (r.text + " " + featureTitle(r.feature) + " " + r.tests.join(" ")).toLowerCase().includes(query)));
  const bare = REQS.filter(r => !r.tests.length).length;
  $("rq-n").textContent = "(" + rows.length + " / " + REQS.length + " · テストの記録なし " + bare + " 件)";
  $("rq-list").innerHTML = rows.length ? '<table class="all"><thead><tr><th></th><th>機能</th><th>番号</th><th>内容</th><th>テスト</th></tr></thead><tbody>' +
    rows.map(r => "<tr><td>" + basketBox("behaviour", r.feature + "#" + r.number) + "</td><td>" + chip("data-feature", r.feature, featureTitle(r.feature)) +
      '</td><td class="num">' + r.number + '</td><td class="md">' + inline(r.text) + "</td><td>" +
      (r.tests.length ? r.tests.map(t => "<code>" + esc(t) + "</code>").join("<br>") : '<span class="pill needs">テストの記録なし</span>') + "</td></tr>").join("") +
    "</tbody></table>" : '<div class="empty">該当する振る舞いはありません</div>';
}

function basketLine(key) {
  const [kind, id] = key.split("|");
  if (kind === "feature") return "- 機能「" + featureTitle(id) + "」(" + id + "): docs/features/" + id + "/brief.md と docs/features/" + id + "/spec.md";
  if (kind === "domain") return "- ドメイン「" + domainTitle(id) + "」(" + id + "): docs/features/domains.yaml の " + id;
  if (kind === "screen") return "- 画面「" + (flowById.get(id) || { title: id }).title + "」(" + id + "): docs/features/flow.yaml の " + id;
  if (kind === "term") return "- 用語「" + termWord(id) + "」(" + id + "): docs/features/glossary.yaml の " + id;
  const [feature, number] = id.split("#");
  const row = REQS.find(r => r.feature === feature && String(r.number) === number);
  return "- 振る舞い: 機能「" + featureTitle(feature) + "」の " + number + " 番 — docs/features/" + feature + "/spec.md の Behaviour " + number +
    (row ? "「" + row.text + "」" : "");
}
function instruction() {
  const work = WORK.find(w => w[0] === workType);
  return "Operon の次の作業を feature-pilot で進めてください（.claude/skills/feature-pilot/SKILL.md から始める）。\n" +
    "種類: " + work[1] + " — " + work[2] + "\n対象:\n" + Array.from(basket.keys()).map(basketLine).join("\n") +
    "\n読むもの: docs/features/index.yaml、docs/features/domains.yaml、docs/features/glossary.yaml、docs/features/flow.yaml\n" +
    "してほしいこと:\n（ここに書いてください）\n";
}
function renderBasket() {
  $("bk-n").textContent = basket.size ? " " + basket.size : "";
  if (!basketOpen) return;
  $("bk-body").innerHTML = basket.size ? '<ul class="ways">' + Array.from(basket.keys()).map(key =>
    "<li>" + esc(basketLine(key).slice(2)) + ' <button class="fchip" data-unbasket="' + esc(key) + '">外す</button></li>').join("") + "</ul>" +
    '<div class="filters">' + WORK.map(w => '<button data-work="' + w[0] + '" aria-pressed="' + (w[0] === workType) + '" title="' + esc(w[2]) + '">' + w[1] + "</button>").join("") + "</div>" +
    '<textarea id="bk-text" rows="14" readonly>' + esc(instruction()) + "</textarea>" +
    '<div class="tools"><span id="bk-said"></span><button data-empty-basket="1">空にする</button><button id="bk-copy" class="primary">指示をコピー</button></div>'
    : '<p class="empty">まだ何も入っていません</p><p class="what">機能・画面・振る舞い・用語・ドメインの「かご」に印を付けると、ここに集まります。</p>';
}
function copyInstruction() {
  const area = $("bk-text");
  const failed = () => { area.focus(); area.select(); $("bk-said").textContent = "コピーできませんでした。選択された文を手でコピーしてください"; };
  if (!navigator.clipboard) { failed(); return; }
  navigator.clipboard.writeText(area.value).then(() => { $("bk-said").textContent = "コピーしました"; }, failed);
}

function route() {
  const raw = decodeURIComponent(location.hash.slice(1));
  const featureRoute = raw.match(/^feature\/([^/]+)(?:\/(.+))?$/);
  const named = raw.match(/^(domain|screen|term|requirements)(?:\/(.+))?$/);
  const lists = { domains: "domains", flow: "flow", glossary: "glossary", requirements: "requirements" };
  if (current && (named || lists[raw])) { current = null; $("detail").hidden = true; $("shade").hidden = true; }
  if (named && named[1] === "domain" && domainById.has(named[2])) {
    baseHash = "#" + raw; view = "domain"; domainSel = domainById.get(named[2]); renderViews(); return;
  }
  if (named && named[1] === "screen") {
    baseHash = "#" + raw; view = "flow"; screenSel = flowById.get(named[2]) || null; renderViews(); return;
  }
  if (named && named[1] === "term") {
    baseHash = "#" + raw; view = "glossary"; termSel = termById.get(named[2]) || null; renderViews(); return;
  }
  if (named && named[1] === "requirements") {
    baseHash = "#" + raw; view = "requirements";
    if (named[2] && featureById.has(named[2])) { reqFeature = named[2]; reqDomain = ""; }
    renderViews(); return;
  }
  if (lists[raw]) {
    baseHash = "#" + raw; view = lists[raw];
    if (raw === "glossary") termSel = null;
    if (raw === "flow") screenSel = null;
    renderViews(); return;
  }
  const changeRoute = raw.match(/^(\d+|[\w-]+)(?:\/(.+))?$/);
  if (featureRoute && featureById.has(featureRoute[1])) {
    baseHash = "#" + raw; view = "feature"; feature = featureById.get(featureRoute[1]);
    featureTab = FTABS.some(t => t[0] === featureRoute[2]) ? featureRoute[2] : "brief";
    if (current) { current = null; $("detail").hidden = true; $("shade").hidden = true; }
  } else if (raw === "changes") {
    baseHash = "#changes"; view = "changes";
    if (current) { current = null; $("detail").hidden = true; $("shade").hidden = true; }
  } else if (raw === "" || raw === "product" || featureRoute) {
    baseHash = ""; view = "product";
    if (current) { current = null; $("detail").hidden = true; $("shade").hidden = true; }
  } else if (changeRoute) {
    const c = M.changes.find(x => x.number === changeRoute[1] || x.name === changeRoute[1]);
    if (c) { show(c.name, changeRoute[2]); }
  }
  renderViews();
}
const VIEW_OF = { product: "product", feature: "product", domains: "domains", domain: "domains", flow: "flow",
  glossary: "glossary", requirements: "requirements", changes: "changes" };
function renderViews() {
  for (const name of ["product", "feature", "domains", "domain", "flow", "glossary", "requirements", "changes"])
    $("v-" + name).hidden = view !== name;
  for (const b of document.querySelectorAll("nav.views button")) b.setAttribute("aria-pressed", String(b.dataset.view === VIEW_OF[view]));
  if (view === "product") renderProduct();
  else if (view === "feature") renderFeature();
  else if (view === "domains") renderDomains();
  else if (view === "domain") renderDomain();
  else if (view === "flow") renderFlow();
  else if (view === "glossary") renderGlossary();
  else if (view === "requirements") renderRequirements();
  else renderOverview();
  renderBasket();
  if (!["feature", "glossary", "flow"].includes(view)) window.scrollTo(0, 0);
}
function fromHash() { route(); }

document.addEventListener("click", event => {
  if (event.target.closest("label.bk")) return;
  const node = event.target.closest("[data-screen]");
  if (node) { setHash("#screen/" + node.dataset.screen, false); return; }
  const extra = event.target.closest("[data-domain],[data-term],[data-reqs],[data-unbasket],[data-work],[data-empty-basket]");
  if (extra) {
    if (extra.dataset.domain) setHash("#domain/" + extra.dataset.domain, false);
    else if (extra.dataset.term) setHash("#term/" + extra.dataset.term, false);
    else if (extra.dataset.reqs) setHash("#requirements/" + extra.dataset.reqs, false);
    else if (extra.dataset.unbasket) { basket.delete(extra.dataset.unbasket); renderViews(); }
    else if (extra.dataset.work) { workType = extra.dataset.work; renderBasket(); }
    else { basket.clear(); renderViews(); }
    return;
  }
  const target = event.target.closest("[data-open],[data-more],[data-group],[data-tab],[data-feature],[data-view],[data-ftab]");
  if (!target) return;
  if (target.dataset.feature) { setHash("#feature/" + target.dataset.feature + (current ? "" : ""), false); }
  else if (target.dataset.ftab) { setHash("#feature/" + feature.id + "/" + target.dataset.ftab, true); }
  else if (target.dataset.view) { setHash(target.dataset.view === "product" ? "" : "#" + target.dataset.view, false); }
  else if (target.dataset.open) show(target.dataset.open);
  else if (target.dataset.more) { expanded.add(target.dataset.more); renderOverview(); }
  else if (target.dataset.group !== undefined) { groupFilter = target.dataset.group; renderOverview(); }
  else if (target.dataset.tab) { show(current.name, target.dataset.tab); }
});
$("body").addEventListener("click", event => {
  if (event.target.id === "raw-toggle") { rawView = !rawView; renderDetail(); }
});
$("q").addEventListener("input", event => {
  query = event.target.value.trim().toLowerCase();
  if (view === "feature" || view === "domain") {
    view = view === "feature" ? "product" : "domains";
    baseHash = view === "product" ? "" : "#domains";
    history.replaceState(null, "", baseHash || location.pathname + location.search);
  }
  renderViews();
});
document.addEventListener("change", event => {
  const box = event.target.closest("input[data-basket]");
  if (box) { if (box.checked) basket.set(box.dataset.basket, true); else basket.delete(box.dataset.basket); renderBasket(); return; }
  const id = event.target.id;
  if (id === "rq-domain") { reqDomain = event.target.value; reqFeature = ""; renderRequirements(); }
  else if (id === "rq-feature") { reqFeature = event.target.value; renderRequirements(); }
  else if (id === "rq-tests") { reqTests = event.target.value; renderRequirements(); }
});
$("bk-open").addEventListener("click", () => { basketOpen = true; $("basket").hidden = false; renderBasket(); });
$("bk-close").addEventListener("click", () => { basketOpen = false; $("basket").hidden = true; });
$("basket").addEventListener("click", event => { if (event.target.id === "bk-copy") copyInstruction(); });
document.addEventListener("keydown", event => {
  const node = event.target.closest && event.target.closest("[data-screen]");
  if (node && event.key === "Enter") setHash("#screen/" + node.dataset.screen, false);
});
$("shade").addEventListener("click", hide);
$("d-close").addEventListener("click", hide);
$("d-prev").addEventListener("click", () => { const n = listed.indexOf(current); if (n > 0) show(listed[n - 1].name, currentTab); });
$("d-next").addEventListener("click", () => { const n = listed.indexOf(current); if (n >= 0 && n < listed.length - 1) show(listed[n + 1].name, currentTab); });
document.addEventListener("keydown", event => {
  if (event.key !== "Escape") return;
  if (current) hide();
  else if (basketOpen) { basketOpen = false; $("basket").hidden = true; }
});
window.addEventListener("hashchange", fromHash);

$("meta").innerHTML = "生成 " + esc(M.generated) + (M.worktrees_read ? "" : ' · <span class="warn">他の作業コピーを読めませんでした</span>');
route();
</script>
</body>
</html>
"""


if __name__ == "__main__":
    sys.exit(main())
