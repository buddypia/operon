#!/usr/bin/env python3
"""The mutation sweep a change copies. Put it in the change directory as
`mutations.py` and fill in the four blocks below. The runner is
scripts/mutation_sweep.py, and its header holds the seven rules every sweep
keeps — read it before writing a mutation. Rule 4 is the one a copy forgets:
mutate the claim, not only the counter. `.claude/skills/sdlc/SKILL.md` step
10 asks for a sweep; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

# 1. WHERE. An absolute path: this runs from anywhere, and a worktree session's
#    `GIT_DIR`/`GIT_WORK_TREE` do not point here.
ROOT = Path('/Users/a13973/dev/buddypia/operon/.worktrees/fix/234-session-project-picker-lost-its-controls')

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 180

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = ['workspace_project_picker_', 'select_session_clears_terminal_focus_request_for_other_session', 'test_remove_project_cleans_session_selection_and_path_cache', 'a_long_project_name_keeps_its_count_and_add_inside_its_block', 'every_translation_table_is_sorted_by_message_id']
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
MUTATIONS = {
    'target-session-loses-its-focus-request': (ROOT / "src/app.rs", 'if focus_requested {', 'if focus_requested && false {'),
    'deleted-project-skips-terminal-cache-cleanup': (ROOT / "src/app.rs", 'if selected_removed {', 'if selected_removed && false {'),
    'deleted-project-retains-visible-workspace': (ROOT / "src/app.rs", 'if self.workspace_project == Some(project_id) {', 'if self.workspace_project == Some(project_id) && false {'),
}

# 5. Nothing below this line is filled in.

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
