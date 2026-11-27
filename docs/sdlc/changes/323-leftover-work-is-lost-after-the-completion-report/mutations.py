#!/usr/bin/env python3
"""The mutation sweep for change 323: the backlog files an item with its kind
and scores, refuses what it cannot score, files nothing twice, lists best
first, hands each item to one session, frees and closes items, says why when
GitHub cannot be reached, runs on main alone among its neighbours, and is
reachable from the skill. Each mutation takes one of those away and a
`backlog_` test must refuse to pass.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = ["backlog_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

SCRIPT = ROOT / "scripts/backlog.py"
SKILL = ROOT / ".claude/skills/backlog/SKILL.md"
SDLC = ROOT / ".claude/skills/sdlc/SKILL.md"
AGENTS = ROOT / "AGENTS.md"
POLICY = ROOT / ".claude/config/worktree-policy.json"
RISK = ROOT / "docs/sdlc/risk.yaml"

MUTATIONS = {
    # Requirement 1.
    "an item is filed without the label for its kind": (
        SCRIPT,
        '"labels": [LABEL, "%s:%s" % (LABEL, arguments.kind)]})',
        '"labels": [LABEL]})',
    ),
    "the scores are left out of the issue": (
        SCRIPT,
        "    parts.append(owned_lines(scores))\n",
        "    parts.append(\"\")\n",
    ),
    "where the item was found is dropped": (
        SCRIPT,
        '        parts.append("発見元: " + arguments.source.strip())',
        "        pass",
    ),
    "the title rides on gh's command line": (
        SCRIPT,
        "    command.append(REPOSITORY + path)\n",
        "    command.append(REPOSITORY + path + ((\"#\" + str(payload.get(\"title\", \"\"))) if payload else \"\"))\n",
    ),
    # Requirement 2.
    "a score of 0 or 11 is accepted": (
        SCRIPT,
        "not 1 <= int(raw) <= 10:",
        "not 0 <= int(raw) <= 11:",
    ),
    "an empty title is filed": (
        SCRIPT,
        '    if not title:\n        raise Refused("タイトルが空です")',
        '    if False:\n        raise Refused("タイトルが空です")',
    ),
    "a title of any length is filed": (
        SCRIPT,
        "    if len(title) > TITLE_CHARACTERS:",
        "    if False:",
    ),
    "a title of 256 characters is refused": (
        SCRIPT,
        "TITLE_CHARACTERS = 256",
        "TITLE_CHARACTERS = 255",
    ),
    "a body of any length is filed": (
        SCRIPT,
        "    if len(body) > BODY_CHARACTERS:",
        "    if False:",
    ),
    "an unknown kind is filed": (
        SCRIPT,
        "    if arguments.kind not in KINDS:\n",
        "    if False:\n",
    ),
    "the refusal no longer says the title is too long": (
        SCRIPT,
        "タイトルが長すぎます",
        "入力を確認してください",
    ),
    # Requirement 3.
    "a title already open is filed again": (
        SCRIPT,
        "    existing = earliest_same()\n    if existing is not None:",
        "    existing = earliest_same()\n    if False:",
    ),
    "titles are compared letter for letter": (
        SCRIPT,
        '    return " ".join(text.split()).casefold()',
        "    return text",
    ),
    "of two filed at once both stay open": (
        SCRIPT,
        "    if earlier is not None and earlier < number:",
        "    if False:",
    ),
    # Requirement 4.
    "the lowest score is listed first": (
        SCRIPT,
        '-(item["score"] or 0), item["number"]))',
        '(item["score"] or 0), item["number"]))',
    ),
    "of equal scores the newest is first": (
        SCRIPT,
        '-(item["score"] or 0), item["number"]))',
        '-(item["score"] or 0), -item["number"]))',
    ),
    "an item whose scores cannot be read is dropped": (
        SCRIPT,
        'if "pull_request" not in issue]',
        'if "pull_request" not in issue and scores_of(issue.get("body"))]',
    ),
    "a pull request is listed as an item": (
        SCRIPT,
        'items = [item_of(issue) for issue in issues if "pull_request" not in issue]',
        "items = [item_of(issue) for issue in issues]",
    ),
    "an unscored item no longer says its scores cannot be read": (
        SCRIPT,
        'scored = "スコアを読めません"',
        'scored = "ICE -"',
    ),
    "an empty backlog prints nothing": (
        SCRIPT,
        '        print("backlog は空です")',
        "        pass",
    ),
    "a held item is not marked in the list": (
        SCRIPT,
        '        text += "  着手中: %s %s" % (item["held_by"], ago(item["held_since"]))',
        "        pass",
    ),
    # Requirement 5.
    "nothing left to claim is an error": (
        SCRIPT,
        '        print("着手できる項目はありません")\n    return 0',
        '        print("着手できる項目はありません")\n    return 1',
    ),
    "a named item that is held is reported as an ordinary failure": (
        SCRIPT,
        '% (number, holder), 3)',
        '% (number, holder), 2)',
    ),
    "claim says nothing about the item it took": (
        SCRIPT,
        "                show_claimed(item, arguments.json)\n",
        "                pass\n",
    ),
    # Requirement 6.
    "a claim that arrived second keeps the item": (
        SCRIPT,
        '            if claims and claims[0]["id"] == mine["id"]:',
        "            if claims:",
    ),
    "the claim that lost is left on the item": (
        SCRIPT,
        '            withdraw(mine["id"])\n            holder = claims[0]["by"] if claims else "不明"',
        '            holder = claims[0]["by"] if claims else "不明"',
    ),
    "the latest claim wins": (
        SCRIPT,
        '    claims.sort(key=lambda claim: claim["id"])',
        '    claims.sort(key=lambda claim: -claim["id"])',
    ),
    # Requirement 7.
    "release leaves the claim where it was": (
        SCRIPT,
        '        gh("DELETE", "issues/comments/%d" % held["id"])',
        "        pass",
    ),
    "done leaves the issue open": (
        SCRIPT,
        '{"state": "closed", "state_reason": "completed"}',
        '{"state_reason": "completed"}',
    ),
    "done drops the reference it was given": (
        SCRIPT,
        '{"body": "完了: " + (arguments.ref or "参照なし")}',
        '{"body": "完了"}',
    ),
    "done leaves the claim on the closed issue": (
        SCRIPT,
        '    drop_claims(arguments.number)\n    gh("POST", "issues/%d/comments" % arguments.number, {"body": "完了',
        '    gh("POST", "issues/%d/comments" % arguments.number, {"body": "完了',
    ),
    "the list no longer says who holds an item": (
        SCRIPT,
        '            item["held_by"], item["held_since"] = claims[0]["by"], claims[0]["since"]\n    if arguments.json:',
        '            item["held_by"], item["held_since"] = "着手中", claims[0]["since"]\n    if arguments.json:',
    ),
    # Requirement 8.
    "new scores are added beside the old ones": (
        SCRIPT,
        '    body = OWNED_LINES.sub("", issue.get("body") or "").rstrip()',
        '    body = (issue.get("body") or "").rstrip()',
    ),
    "score answers without changing the issue": (
        SCRIPT,
        '    issue = gh("PATCH", "issues/%d" % arguments.number, {"body": body})\n',
        '    issue["body"] = body\n',
    ),
    # Requirement 9.
    "a refusal by gh no longer says GitHub was not reached": (
        SCRIPT,
        'failure = Refused("GitHub に届きません: " + first)',
        'failure = Refused("エラー: " + first)',
    ),
    "gh's own first line is dropped from the refusal": (
        SCRIPT,
        'failure = Refused("GitHub に届きません: " + first)',
        'failure = Refused("GitHub に届きません: ")',
    ),
    "a missing gh is a traceback": (
        SCRIPT,
        '            raise Refused("gh が見つかりません") from None',
        "            raise",
    ),
    "the wait for gh says the wrong number of seconds": (
        SCRIPT,
        'raise Refused("GitHub の応答がありません (%d 秒)" % timeout_seconds)',
        'raise Refused("GitHub の応答がありません (%d 秒)" % TIMEOUT_SECONDS)',
    ),
    "a claim that could not be checked is left on the item": (
        SCRIPT,
        '            except Refused:\n                withdraw(mine["id"])\n                raise',
        "            except Refused:\n                raise",
    ),
    # Findings the verifier reproduced (F1 to F3).
    "text that quotes a claim line holds the item": (
        SCRIPT,
        'CLAIM_COMMENT = re.compile(r"\\A着手しました: [^\\n]*\\n<!-- backlog claim: by=([^\\n]*?) -->\\s*\\Z")',
        'CLAIM_COMMENT = re.compile(r"[\\s\\S]*<!-- backlog claim: by=([^\\n]*?) -->")',
    ),
    "a comment by someone who cannot push holds the item": (
        SCRIPT,
        'if found and comment.get("author_association") in CLAIMANTS:',
        "if found:",
    ),
    "a closed item can be claimed by its number": (
        SCRIPT,
        '    if issue.get("state") != "open":',
        "    if False:",
    ),
    "exactly a thousand items is reported as more than a thousand": (
        SCRIPT,
        "            return rows, bool(got)",
        "            return rows, True",
    ),
    # Requirement 10.
    "main admits any python3 command": (
        POLICY,
        '"python3\\\\s+scripts/backlog\\\\.py"',
        '"python3"',
    ),
    "main admits any file whose name starts like the script's": (
        POLICY,
        '"python3\\\\s+scripts/backlog\\\\.py"',
        '"python3\\\\s+scripts/backlog\\\\.py\\\\S*"',
    ),
    "main no longer admits the script": (
        POLICY,
        '"python3\\\\s+scripts/backlog\\\\.py"',
        '"python3\\\\s+scripts/nothing\\\\.py"',
    ),
    "the script is no longer gate configuration": (
        RISK,
        'approval.md,scripts/backlog.py"',
        'approval.md"',
    ),
    # Requirement 11.
    "the skill sends a session to a sub-command that does not exist": (
        SKILL,
        "python3 scripts/backlog.py claim 12",
        "python3 scripts/backlog.py take 12",
    ),
    "the skill never says how to give an item back": (
        SKILL,
        'python3 scripts/backlog.py release 12 --reason "再現できなかった。手順が必要"',
        "(give it back)",
    ),
    "the script loses a sub-command the skill names": (
        SCRIPT,
        'commands.add_parser("score"',
        'commands.add_parser("rescore"',
    ),
    "AGENTS.md no longer says when to read the skill": (
        AGENTS,
        "- When a change leaves work over, or the next task is asked for, read `.claude/skills/backlog/SKILL.md`.\n",
        "",
    ),
    "the closing step no longer points at the skill": (
        SDLC,
        "  `.claude/skills/backlog/SKILL.md` — a remark in the final message is lost",
        "  the backlog skill — a remark in the final message is lost",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
