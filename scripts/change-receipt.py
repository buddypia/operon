#!/usr/bin/env python3
"""Reconstruct provenance, not a verification verdict, from committed records.

Run from the repository: python3 scripts/change-receipt.py CHANGE [REVISION]
No report is persisted. Unknown run telemetry stays null. Reads at most thirteen
fixed records (64 KiB each); every git invocation has a ten-second deadline.
"""
import hashlib
import json
import os
import re
import selectors
import subprocess
import sys
import time

MAX_BYTES = 64 * 1024
GIT_SECONDS = 10
RECORDS = ("state.yaml", "intent.md", "spec.md", "plan.md", "approvals.log", "review.yaml")
POLICIES = (
    "AGENTS.md", "CLAUDE.md", "REVIEW.md", "docs/sdlc/routes.yaml",
    "docs/sdlc/risk.yaml", ".claude/skills/feature-pilot/SKILL.md",
    ".claude/skills/sdlc/SKILL.md",
)


class ReceiptError(Exception):
    pass


def git(*args, optional=False):
    # CLI session exports must not redirect this repository. Replacement
    # objects and grafts must not rewrite an explicitly named commit.
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_NO_REPLACE_OBJECTS="1", GIT_GRAFT_FILE=os.devnull,
                       GIT_NO_LAZY_FETCH="1", GIT_ADVICE="0")
    child = subprocess.Popen(["git", *args], stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, env=environment)
    deadline = time.monotonic() + GIT_SECONDS
    buffers = {child.stdout: bytearray(), child.stderr: bytearray()}
    total = 0
    try:
        with selectors.DefaultSelector() as selector:
            for stream in buffers:
                selector.register(stream, selectors.EVENT_READ)
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise ReceiptError("git exceeded its ten-second deadline")
                for key, _ in selector.select(remaining):
                    data = os.read(key.fd, min(8192, MAX_BYTES + 1 - total))
                    total += len(data)
                    if total > MAX_BYTES:
                        raise ReceiptError("git output exceeds 64 KiB")
                    if data:
                        buffers[key.fileobj].extend(data)
                    else:
                        selector.unregister(key.fileobj)
            code = child.wait(timeout=max(0.001, deadline - time.monotonic()))
        if code:
            if optional:
                return None
            raise ReceiptError("git could not read the requested commit or object")
        return bytes(buffers[child.stdout])
    finally:
        if child.poll() is None:
            child.kill()
        child.wait()
        for stream in buffers:
            stream.close()


def blob(commit, path):
    object_id = git("rev-parse", "--verify", "--end-of-options",
                    f"{commit}:{path}", optional=True)
    if object_id is None:
        return None
    object_id = object_id.decode("ascii").strip()
    size = int(git("cat-file", "-s", object_id))
    if size > MAX_BYTES:
        raise ReceiptError(f"{path}: record exceeds 64 KiB")
    # Immutable objects cannot grow; git() still counts actual output reads.
    # A tree/submodule instead of a record fails explicit blob conversion.
    return git("cat-file", "blob", object_id)


def validate_change(change):
    if not re.fullmatch(r"[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*", change):
        raise ReceiptError("expected a numbered change name, not a path")


def resolve_commit(revision):
    if not revision or revision.startswith("-") or len(revision) > 200:
        raise ReceiptError("invalid revision")
    return git("rev-parse", "--verify", "--end-of-options",
               revision + "^{commit}").decode("ascii").strip()


def receipt(change, revision):
    validate_change(change)
    commit = resolve_commit(revision)
    # Read the object itself: rev-list hides parents at a shallow boundary.
    header = git("cat-file", "commit", commit).split(b"\n\n", 1)[0]
    parents = [line[7:].decode("ascii") for line in header.splitlines()
               if line.startswith(b"parent ")]
    records = {}
    for name in RECORDS:
        data = blob(commit, f"docs/sdlc/changes/{change}/{name}")
        records[name] = None if data is None else {
            "sha256": hashlib.sha256(data).hexdigest(), "text": data.decode("utf-8")}
    if records["state.yaml"] is None or not records["state.yaml"]["text"].strip():
        raise ReceiptError("change has no nonempty committed state.yaml")
    policies = {}
    for path in POLICIES:
        data = blob(commit, path)
        policies[path] = None if data is None else hashlib.sha256(data).hexdigest()
    return {
        "schema_version": 1,
        "change": change,
        "snapshot": commit,
        "previous_snapshot": parents[0] if parents else None,
        "rollback_scope": "previous snapshot only; not a whole-change revert",
        "evidence_scope": "committed records; not fresh verification",
        "records": records,
        "policies": policies,
        "policy_version": hashlib.sha256(json.dumps(policies, sort_keys=True).encode()).hexdigest(),
        "telemetry": dict.fromkeys(("model_route", "tools_used", "tests", "cost_usd", "human_corrections")),
    }


def main():
    try:
        if len(sys.argv) not in (2, 3):
            raise ReceiptError("usage: change-receipt.py CHANGE [REVISION]")
        result = receipt(sys.argv[1], sys.argv[2] if len(sys.argv) == 3 else "HEAD")
        print(json.dumps(result, ensure_ascii=True, sort_keys=True, indent=2))
        return 0
    except (ReceiptError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(f"change-receipt: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
