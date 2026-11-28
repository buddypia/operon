#!/usr/bin/env python3
"""The backlog: work a session left undone, kept as GitHub issues (change 323).

    python3 scripts/backlog.py add --title T --kind remaining|improvement|problem \\
            --impact N --confidence N --ease N [--body TEXT] [--source TEXT]
    python3 scripts/backlog.py list [--json]
    python3 scripts/backlog.py claim [NUMBER] [--json]
    python3 scripts/backlog.py release NUMBER [--reason TEXT]
    python3 scripts/backlog.py score NUMBER --impact N --confidence N --ease N
    python3 scripts/backlog.py done NUMBER [--ref TEXT]

No file is kept here. Every fact is on the repository's issues, reached through
`gh api`, so the main checkout and every worktree read one list. An item's
score is impact x confidence x ease, each 1 to 10, held in a comment line in
the issue body. An item is held while it carries a claim comment; of several
claims the one GitHub numbered first wins and the others remove themselves.

A claim at least STALE_HOURS old whose holder left neither a linked working
copy of that name nor a commit naming the item as `(#N)` is given back by the
next `claim` that reaches it (change 324). A claim made on the main copy moves
into the working copy that names it with `claim NUMBER`, if that copy was
created after the claim. When git cannot answer, nothing is given back.

Exit 0 done (including "nothing to claim"), 2 refused or GitHub unreachable,
3 the named item is already held.

The main checkout's allowlist admits this file by name
(.claude/config/worktree-policy.json), which is why docs/sdlc/risk.yaml lists
it as gate configuration.
"""

import argparse
import datetime
import json
import os
import re
import subprocess
import sys
import tempfile
import threading

LABEL = "backlog"
KINDS = {"remaining": "残り作業", "improvement": "改善", "problem": "問題"}
LABEL_COLOURS = {LABEL: "5319e7", "remaining": "fbca04", "improvement": "a2eeef", "problem": "d73a4a"}
SCORES = ("impact", "confidence", "ease")

REPOSITORY = "repos/{owner}/{repo}/"  # gh fills both in from the directory it runs in
PAGE = 100
PAGES = 10
COMMENT_READS = 100
ANSWER_BYTES = 32 * 1024 * 1024
PRINTED_BODY_BYTES = 16 * 1024
TITLE_CHARACTERS = 256
BODY_CHARACTERS = 60000
TIMEOUT_SECONDS = 60
STALE_HOURS = 24
GIT_ANSWER_BYTES = 1024 * 1024
# A git hook exports these to its children; left in, git would read the hook's
# repository instead of the one this command runs in.
REPOSITORY_POINTERS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR")

ICE_LINE = re.compile(r"<!-- backlog ice: impact=(\d+) confidence=(\d+) ease=(\d+) -->")
OWNED_LINES = re.compile(r"\n*^ICE \d+ = [^\n]*\n<!-- backlog ice: [^\n]*-->[ \t]*$", re.M)
# The whole comment, so text that only quotes the line holds nothing.
CLAIM_COMMENT = re.compile(r"\A着手しました: [^\n]*\n<!-- backlog claim: by=([^\n]*?) -->\s*\Z")
# The repository is public and anyone may comment; only these may hold an item.
CLAIMANTS = ("OWNER", "MEMBER", "COLLABORATOR")


class Refused(Exception):
    """Something the person or the session reads, with the exit code."""

    def __init__(self, text, code=2):
        super().__init__(text)
        self.code = code


timeout_seconds = TIMEOUT_SECONDS


def gh(method, path, payload=None):
    """One `gh api` call: parsed JSON, or None for an empty answer."""
    command = ["gh", "api", "-X", method]
    if payload is not None:
        command += ["--input", "-"]
    command.append(REPOSITORY + path)
    with tempfile.TemporaryFile() as errors:
        try:
            child = subprocess.Popen(
                command,
                stdin=subprocess.PIPE if payload is not None else subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=errors,
            )
        except FileNotFoundError:
            raise Refused("gh が見つかりません") from None
        timed_out = threading.Event()

        def stop():
            timed_out.set()
            child.kill()

        timer = threading.Timer(timeout_seconds, stop)
        timer.start()
        try:
            if payload is not None:
                try:
                    child.stdin.write(json.dumps(payload).encode())
                    child.stdin.close()
                except BrokenPipeError:
                    pass
            answer = child.stdout.read(ANSWER_BYTES + 1)
            if len(answer) > ANSWER_BYTES:
                child.kill()
            code = child.wait()
        finally:
            timer.cancel()
        if timed_out.is_set():
            raise Refused("GitHub の応答がありません (%d 秒)" % timeout_seconds)
        if len(answer) > ANSWER_BYTES:
            raise Refused("GitHub の応答が大きすぎます")
        errors.seek(0)
        said = errors.read(64 * 1024).decode("utf-8", "replace").strip()
    text = answer.decode("utf-8", "replace")
    if code != 0:
        first = said.splitlines()[0] if said else "gh が %d で終わりました" % code
        failure = Refused("GitHub に届きません: " + first)
        failure.answer = text
        raise failure
    if not text.strip():
        return None
    try:
        return json.loads(text)
    except ValueError:
        raise Refused("GitHub の応答を読めません") from None


def git(*arguments):
    """One git child in this directory: its output, or None on any failure."""
    environment = {name: value for name, value in os.environ.items() if name not in REPOSITORY_POINTERS}
    try:
        child = subprocess.Popen(
            ["git", *arguments],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            env=environment,
        )
    except OSError:
        return None
    timer = threading.Timer(timeout_seconds, child.kill)
    timer.start()
    try:
        answer = child.stdout.read(GIT_ANSWER_BYTES + 1)
        if len(answer) > GIT_ANSWER_BYTES:
            child.kill()
        code = child.wait()
    finally:
        timer.cancel()
        child.stdout.close()
    if code != 0 or len(answer) > GIT_ANSWER_BYTES:
        return None
    return answer.decode("utf-8", "replace")


def working_copies():
    """(the main copy's name, the names of linked copies that exist), or None."""
    text = git("worktree", "list", "--porcelain")
    if text is None:
        return None
    entries = []
    for block in text.split("\n\n"):
        lines = block.splitlines()
        paths = [line[len("worktree "):] for line in lines if line.startswith("worktree ")]
        if paths:
            locked = any(line == "locked" or line.startswith("locked ") for line in lines)
            entries.append((paths[0], locked))
    if not entries:
        return None
    main = name_of(os.path.basename(entries[0][0]))
    linked = {name_of(os.path.basename(path)) for path, locked in entries[1:] if locked or os.path.isdir(path)}
    return main, linked


def left_behind(number, held, copies):
    """True only when the claim is old, its holder has no working copy, and
    no commit names the item. Anything git cannot answer keeps the item."""
    then = moment(held["since"])
    if then is None or datetime.datetime.now(datetime.timezone.utc).timestamp() - then < STALE_HOURS * 3600:
        return False
    if copies is None or held["by"] in copies[1]:
        return False
    named = git("log", "--branches", "--remotes", "-F", "--grep", "(#%d)" % number, "-1", "--format=%H")
    return named is not None and not named.strip()


def born_after(stamp):
    """Whether this working copy was created at or after the claim's time."""
    then = moment(stamp)
    directory = git("rev-parse", "--absolute-git-dir")
    if then is None or directory is None:
        return False
    try:
        born = os.stat(directory.strip()).st_birthtime
    except (OSError, AttributeError):
        return False
    return born >= then


def pages(path):
    """Every row of a listing, up to PAGES pages, and whether more were left."""
    rows = []
    for number in range(1, PAGES + 2):
        joiner = "&" if "?" in path else "?"
        got = gh("GET", "%s%sper_page=%d&page=%d" % (path, joiner, PAGE, number)) or []
        if number > PAGES:
            # One page past the ceiling, read only to learn whether it exists.
            return rows, bool(got)
        rows += got
        if len(got) < PAGE:
            break
    return rows, False


def scores_of(body):
    found = ICE_LINE.findall(body or "")
    if not found:
        return None
    values = tuple(int(value) for value in found[-1])
    return values if all(1 <= value <= 10 for value in values) else None


def item_of(issue):
    scores = scores_of(issue.get("body"))
    kinds = [
        label["name"].split(":", 1)[1]
        for label in issue.get("labels", [])
        if label.get("name", "").startswith(LABEL + ":")
    ]
    item = {
        "number": issue["number"],
        "title": issue.get("title", ""),
        "kind": kinds[0] if kinds else None,
        "impact": None,
        "confidence": None,
        "ease": None,
        "score": None,
        "url": issue.get("html_url", ""),
        "body": issue.get("body") or "",
        "commented": issue.get("comments", 0) > 0,
        "held_by": None,
        "held_since": None,
        "held_known": True,
    }
    if scores:
        item.update(zip(SCORES, scores))
        item["score"] = scores[0] * scores[1] * scores[2]
    return item


def open_items():
    """Open items, best first: by score, then oldest; unscored ones last."""
    issues, more = pages("issues?labels=%s&state=open" % LABEL)
    items = [item_of(issue) for issue in issues if "pull_request" not in issue]
    items.sort(key=lambda item: (item["score"] is None, -(item["score"] or 0), item["number"]))
    return items, more


def one_item(number):
    issue = gh("GET", "issues/%d" % number)
    names = [label.get("name") for label in (issue or {}).get("labels", [])]
    if not issue or "pull_request" in issue or LABEL not in names:
        raise Refused("#%d は backlog の項目ではありません" % number)
    if issue.get("state") != "open":
        raise Refused("#%d は閉じています" % number)
    return issue


def claims_on(number):
    """The claim comments on an issue, earliest first."""
    comments, _ = pages("issues/%d/comments" % number)
    claims = []
    for comment in comments:
        found = CLAIM_COMMENT.match(comment.get("body") or "")
        if found and comment.get("author_association") in CLAIMANTS:
            claims.append({"id": comment["id"], "by": found.group(1), "since": comment.get("created_at")})
    claims.sort(key=lambda claim: claim["id"])
    return claims


def name_of(text):
    """A directory name as a claim carries it; written and compared alike."""
    return re.sub(r"\s+|--+", "-", text)


def who():
    # The working copy's directory only: a host name is the person's, and an
    # issue is not the place for it. The copy's root, not the directory the
    # command runs in, so a session in a subdirectory is still known by the
    # working copy `left_behind` looks for.
    top = git("rev-parse", "--show-toplevel")
    here = top.strip() if top and top.strip() else os.getcwd()
    return name_of(os.path.basename(here)) or "session"


def moment(stamp):
    """GitHub's UTC time as seconds since the epoch, or None."""
    try:
        then = datetime.datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=datetime.timezone.utc)
    except (TypeError, ValueError):
        return None
    return then.timestamp()


def ago(stamp):
    then = moment(stamp)
    if then is None:
        return ""
    minutes = int((datetime.datetime.now(datetime.timezone.utc).timestamp() - then) // 60)
    if minutes < 60:
        return "%d分前" % max(minutes, 0)
    if minutes < 60 * 24:
        return "%d時間前" % (minutes // 60)
    return "%d日前" % (minutes // (60 * 24))


def line_of(item):
    if item["score"] is None:
        scored = "スコアを読めません"
    else:
        scored = "ICE %d (I%d C%d E%d)" % (item["score"], item["impact"], item["confidence"], item["ease"])
    text = "#%d  %s  %s  %s" % (item["number"], scored, KINDS.get(item["kind"], "種類なし"), item["title"])
    if item["held_by"]:
        text += "  着手中: %s %s" % (item["held_by"], ago(item["held_since"]))
    elif not item["held_known"]:
        text += "  着手状況は未確認"
    return text


def public(item):
    return {key: value for key, value in item.items() if key not in ("commented", "held_known")}


def owned_lines(scores):
    impact, confidence, ease = scores
    return "ICE %d = Impact %d × Confidence %d × Ease %d\n<!-- backlog ice: impact=%d confidence=%d ease=%d -->" % (
        impact * confidence * ease,
        impact,
        confidence,
        ease,
        impact,
        confidence,
        ease,
    )


def read_scores(arguments):
    values = []
    for name in SCORES:
        raw = getattr(arguments, name)
        if raw is None or not re.fullmatch(r"\d{1,2}", raw) or not 1 <= int(raw) <= 10:
            raise Refused("%s は 1 から 10 の整数です" % name)
        values.append(int(raw))
    return tuple(values)


def same_title(text):
    return " ".join(text.split()).casefold()


def ensure_labels(kind):
    existing, _ = pages("labels")
    names = {label.get("name") for label in existing}
    for name, colour in ((LABEL, LABEL_COLOURS[LABEL]), ("%s:%s" % (LABEL, kind), LABEL_COLOURS[kind])):
        if name in names:
            continue
        try:
            gh("POST", "labels", {"name": name, "color": colour})
        except Refused as failure:
            # Another session created it between the read and the write.
            if "already_exists" not in getattr(failure, "answer", ""):
                raise


def add(arguments):
    title = " ".join((arguments.title or "").split())
    if not title:
        raise Refused("タイトルが空です")
    if len(title) > TITLE_CHARACTERS:
        raise Refused("タイトルが長すぎます (%d 文字まで)" % TITLE_CHARACTERS)
    if arguments.kind not in KINDS:
        raise Refused("kind は %s のいずれかです" % ", ".join(KINDS))
    scores = read_scores(arguments)
    parts = [arguments.body.strip()] if arguments.body and arguments.body.strip() else []
    if arguments.source:
        parts.append("発見元: " + arguments.source.strip())
    parts.append(owned_lines(scores))
    body = "\n\n".join(parts)
    if len(body) > BODY_CHARACTERS:
        raise Refused("本文が長すぎます (%d 文字まで)" % BODY_CHARACTERS)

    def earliest_same():
        items, _ = open_items()
        same = [item["number"] for item in items if same_title(item["title"]) == same_title(title)]
        return min(same) if same else None

    existing = earliest_same()
    if existing is not None:
        print("既に登録されています: #%d" % existing)
        return 0
    ensure_labels(arguments.kind)
    issue = gh("POST", "issues", {"title": title, "body": body, "labels": [LABEL, "%s:%s" % (LABEL, arguments.kind)]})
    number = issue["number"]
    earlier = earliest_same()
    if earlier is not None and earlier < number:
        gh("POST", "issues/%d/comments" % number, {"body": "重複: #%d" % earlier})
        gh("PATCH", "issues/%d" % number, {"state": "closed", "state_reason": "not_planned"})
        print("既に登録されています: #%d" % earlier)
        return 0
    print("#%d ICE %d %s %s" % (number, scores[0] * scores[1] * scores[2], KINDS[arguments.kind], title))
    if issue.get("html_url"):
        print(issue["html_url"])
    return 0


def list_items(arguments):
    items, more = open_items()
    reads = 0
    for item in items:
        if not item["commented"]:
            continue
        if reads >= COMMENT_READS:
            item["held_known"] = False
            continue
        reads += 1
        claims = claims_on(item["number"])
        if claims:
            item["held_by"], item["held_since"] = claims[0]["by"], claims[0]["since"]
    if arguments.json:
        print(json.dumps([public(item) for item in items], ensure_ascii=False))
        return 0
    if not items:
        print("backlog は空です")
    for item in items:
        print(line_of(item))
    if more:
        print("%d 件を超える項目は読んでいません" % (PAGE * PAGES))
    return 0


def claim_body(by):
    return "着手しました: %s\n<!-- backlog claim: by=%s -->" % (by, by)


def claim(arguments):
    if arguments.number is not None:
        candidates, named = [item_of(one_item(arguments.number))], True
    else:
        items, _ = open_items()
        candidates, named = [item for item in items if item["score"] is not None], False
    tried = 0
    copies = False  # read once, when the first held item needs it
    for item in candidates:
        number = item["number"]
        if tried >= COMMENT_READS:
            break
        tried += 1
        holder = None
        if item["commented"]:
            claims = claims_on(number)
            holder = claims[0]["by"] if claims else None
        if holder is not None:
            here = who()
            if copies is False:
                copies = working_copies()
            # Every session on the main copy shares its name, so only a linked
            # copy can tell that a claim under its name is its own.
            linked_here = copies is not None and here in copies[1] and here != copies[0]
            if named and linked_here and holder == here:
                item["held_by"], item["held_since"] = holder, claims[0]["since"]
                show_claimed(item, arguments.json)
                return 0
            if named and linked_here and holder == copies[0] and born_after(claims[0]["since"]):
                move_here(item, claims, arguments.json)
                return 0
            if left_behind(number, claims[0], copies):
                give_back(number, holder)
                holder = None
        if holder is None:
            by = who()
            mine = gh("POST", "issues/%d/comments" % number, {"body": claim_body(by)})
            try:
                claims = claims_on(number)
            except Refused:
                withdraw(mine["id"])
                raise
            if claims and claims[0]["id"] == mine["id"]:
                item["held_by"], item["held_since"] = by, claims[0]["since"]
                show_claimed(item, arguments.json)
                return 0
            withdraw(mine["id"])
            holder = claims[0]["by"] if claims else "不明"
        if named:
            raise Refused("#%d は既に着手されています: %s" % (number, holder), 3)
    if arguments.json:
        print("null")
    else:
        print("着手できる項目はありません")
    return 0


def give_back(number, holder):
    """Delete every claim on an item whose holder is gone, and say why."""
    for held in claims_on(number):
        try:
            gh("DELETE", "issues/comments/%d" % held["id"])
        except Refused as failure:
            # Another session giving it back at the same moment deleted it.
            if "404" not in str(failure):
                raise
    gh(
        "POST",
        "issues/%d/comments" % number,
        {"body": "戻しました: 着手から 1 日以上たち、作業ツリーもこの項目のコミットもありません (着手: %s)" % holder},
    )


def move_here(item, claims, as_json):
    """Hold the item under this working copy's name instead of the main copy's."""
    by = who()
    mine = gh("POST", "issues/%d/comments" % item["number"], {"body": claim_body(by)})
    for held in claims:
        withdraw(held["id"])
    item["held_by"], item["held_since"] = by, mine.get("created_at")
    print("#%d の着手をこの作業ツリーに移しました" % item["number"], file=sys.stderr if as_json else sys.stdout)
    show_claimed(item, as_json)


def withdraw(comment_id):
    """Remove our own claim; a failure here must not hide the one that led to it."""
    try:
        gh("DELETE", "issues/comments/%d" % comment_id)
    except Refused:
        pass


def show_claimed(item, as_json):
    if as_json:
        print(json.dumps(public(item), ensure_ascii=False))
        return
    print(line_of(item))
    if item["url"]:
        print(item["url"])
    body = OWNED_LINES.sub("", item["body"]).strip()
    encoded = body.encode()
    if len(encoded) > PRINTED_BODY_BYTES:
        body = encoded[:PRINTED_BODY_BYTES].decode("utf-8", "ignore") + "\n… (以下省略)"
    if body:
        print()
        print(body)


def drop_claims(number):
    claims = claims_on(number)
    for held in claims:
        gh("DELETE", "issues/comments/%d" % held["id"])
    return len(claims)


def release(arguments):
    one_item(arguments.number)
    if drop_claims(arguments.number) == 0:
        print("#%d は着手されていません" % arguments.number)
        return 0
    if arguments.reason:
        gh("POST", "issues/%d/comments" % arguments.number, {"body": "戻しました: " + arguments.reason})
    print("#%d を戻しました" % arguments.number)
    return 0


def score(arguments):
    scores = read_scores(arguments)
    issue = one_item(arguments.number)
    body = OWNED_LINES.sub("", issue.get("body") or "").rstrip()
    body = (body + "\n\n" if body else "") + owned_lines(scores)
    issue = gh("PATCH", "issues/%d" % arguments.number, {"body": body})
    print(line_of(item_of(issue)))
    return 0


def done(arguments):
    one_item(arguments.number)
    drop_claims(arguments.number)
    gh("POST", "issues/%d/comments" % arguments.number, {"body": "完了: " + (arguments.ref or "参照なし")})
    gh("PATCH", "issues/%d" % arguments.number, {"state": "closed", "state_reason": "completed"})
    print("#%d を閉じました" % arguments.number)
    return 0


def main():
    global timeout_seconds
    parser = argparse.ArgumentParser(description="The backlog, kept as GitHub issues.")
    parser.add_argument("--timeout", type=int, default=TIMEOUT_SECONDS, help="seconds to wait for each gh call")
    commands = parser.add_subparsers(dest="command", required=True)

    def with_scores(sub):
        for name in SCORES:
            sub.add_argument("--" + name, help="1 to 10")

    sub = commands.add_parser("add", help="file one item")
    sub.add_argument("--title")
    sub.add_argument("--kind", help=" | ".join(KINDS))
    sub.add_argument("--body")
    sub.add_argument("--source", help="where it was found, for example the change's name")
    with_scores(sub)
    sub.set_defaults(run=add)

    sub = commands.add_parser("list", help="open items, best first")
    sub.add_argument("--json", action="store_true")
    sub.set_defaults(run=list_items)

    sub = commands.add_parser("claim", help="take the best item nobody holds, or the one named")
    sub.add_argument("number", nargs="?", type=int)
    sub.add_argument("--json", action="store_true")
    sub.set_defaults(run=claim)

    sub = commands.add_parser("release", help="give a held item back")
    sub.add_argument("number", type=int)
    sub.add_argument("--reason")
    sub.set_defaults(run=release)

    sub = commands.add_parser("score", help="replace an item's scores")
    sub.add_argument("number", type=int)
    with_scores(sub)
    sub.set_defaults(run=score)

    sub = commands.add_parser("done", help="close a finished item")
    sub.add_argument("number", type=int)
    sub.add_argument("--ref", help="the commit or change that finished it")
    sub.set_defaults(run=done)

    arguments = parser.parse_args()
    timeout_seconds = arguments.timeout
    try:
        return arguments.run(arguments)
    except Refused as refusal:
        print(str(refusal), file=sys.stderr)
        return refusal.code


if __name__ == "__main__":
    sys.exit(main())
