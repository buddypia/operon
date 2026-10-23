#!/usr/bin/env python3
"""Mutation sweep for change 217: parallel worktrees skip redundant hook git
queries, share warm targets, raise node hook timeouts, and leave packaging to
main.
"""
import sys
from pathlib import Path

ROOT = Path(
    "/Users/a13973/dev/buddypia/operon/.worktrees/fix/217-parallel-worktrees-repeat-hook-and-ci-work"
)

RUN_SECONDS = 1800

TESTS = [
    "tests::ci_runs_on_the_branches_the_merge_gate_asks_about",
    "tests::a_new_worktree_builds_from_mains_cache_and_formats_its_own_files",
    "tests::every_hook_gives_node_guards_time_to_finish_under_load",
    "tests::worktree_and_allowlisted_calls_skip_redundant_trunk_git_and_registry_clones",
]
CHECKS = [
    [
        "cargo",
        "test",
        "--locked",
        "--manifest-path",
        str(ROOT / "Cargo.toml"),
        "--",
        "--exact",
        *TESTS,
    ]
]

MUTATIONS = {
    "worktree-bash-queries-trunk-branch-before-segment-check": (
        ROOT / ".cli/hooks/worktree-policy-guard.mjs",
        "  if (!parseErr && segment === null) return HookOutput.passthrough();\n\n  const branch = queryBranch(projectDir);",
        "  const branch = queryBranch(projectDir);\n  if (!parseErr && segment === null) return HookOutput.passthrough();",
    ),
    "worktree-edit-queries-trunk-branch-before-tier-check": (
        ROOT / ".cli/hooks/worktree-policy-guard.mjs",
        "    if (deniedRelPath === null) return HookOutput.passthrough();\n\n    const branch = queryBranch(projectDir); // short budget — rationale at queryBranch",
        "    const branch = queryBranch(projectDir); // short budget — rationale at queryBranch\n    if (deniedRelPath === null) return HookOutput.passthrough();",
    ),
    "worktree-commit-queries-trunk-branch": (
        ROOT / ".cli/hooks/commit-guard.mjs",
        "  const branch =\n"
        "    isForeignRepo || isCreatePrActive || isWorktree\n"
        "      ? null\n"
        "      : safeGit('branch --show-current', projectDir);",
        "  const branch = isForeignRepo ? null : safeGit('branch --show-current', projectDir);",
    ),
    "overlay-loader-deep-clones-managed-registry-without-overlay": (
        ROOT / ".cli/lib/hook-registry-overlay.mjs",
        "  if (!read.overlay) {\n"
        "    return {\n"
        "      registry: managed,\n"
        "      added: [],\n"
        "      overridden: [],\n"
        "      present: read.present,\n"
        "      path: HOOK_REGISTRY_LOCAL_REL,\n"
        "      warnings: [...read.warnings],\n"
        "    };\n"
        "  }\n",
        "",
    ),
    "heredoc-strip-drops-memoization": (
        ROOT / ".cli/lib/heredoc-strip.mjs",
        "  if (command === _lastHeredocCmd && _lastHeredocEntry !== null) {\n    return _lastHeredocEntry;\n  }",
        "  if (false && command === _lastHeredocCmd && _lastHeredocEntry !== null) {\n    return _lastHeredocEntry;\n  }",
    ),
    "hook-timeout-reverts-to-five-seconds": (
        ROOT / ".claude/hooks.json",
        '            "command": "node \\"$(dirname \\"$(git rev-parse --path-format=absolute --git-common-dir || echo /nonexistent/nogit)\\")/.cli/_cli-dispatch.mjs\\" .cli/hooks/worktree-owner-tracker.mjs antigravity PostToolUse worktree-owner-tracker",\n'
        '            "timeout": 30',
        '            "command": "node \\"$(dirname \\"$(git rev-parse --path-format=absolute --git-common-dir || echo /nonexistent/nogit)\\")/.cli/_cli-dispatch.mjs\\" .cli/hooks/worktree-owner-tracker.mjs antigravity PostToolUse worktree-owner-tracker",\n'
        '            "timeout": 5',
    ),
    "seed-target-leaves-cargo-lock-in-cloned-target": (
        ROOT / "scripts/seed-target.sh",
        '  rm -f "$worktree/target/debug/.cargo-lock"\n',
        "",
    ),
    "ci-package-job-runs-on-every-branch": (
        ROOT / ".github/workflows/ci.yml",
        "  package:\n    name: macos app bundle packaging\n    if: github.ref == 'refs/heads/main'\n",
        "  package:\n    name: macos app bundle packaging\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
