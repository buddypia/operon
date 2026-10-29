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
TESTS = ["workspace_project_picker_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), *TESTS, "--", "--nocapture"]]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
MUTATIONS = {
    'saved-project-order-lost': (ROOT / 'src/app/screens.rs', '        sort_column_projects(&mut projects, &self.session_arrangement);\n        egui::ScrollArea::vertical()', '        let _ = &mut projects;\n        egui::ScrollArea::vertical()'),
    'project-launch-control-does-nothing': (ROOT / 'src/app/screens.rs', 'self.open_project_session_setup(project.id);', 'self.selected_project = Some(project.id);'),
    'archived-history-floods-live-tabs': (ROOT / 'src/app.rs', 'SessionStatus::Starting | SessionStatus::Active | SessionStatus::Unknown', 'SessionStatus::Starting | SessionStatus::Active | SessionStatus::Unknown | SessionStatus::Exited'),
    'project-roundtrip-discards-file-and-layout': (ROOT / 'src/app.rs', '.insert(previous_project, (previous, self.selected_session));', '.insert(previous_project, ({ let _ = previous; WorkspaceTabs::default() }, self.selected_session));'),
    'project-highlight-does-not-follow-focus': (ROOT / 'src/app.rs', 'self.selected_project = Some(project);', 'let _ = project;'),
    'project-name-is-no-longer-a-drag-grip': (ROOT / 'src/app/screens.rs', 'self.workspace_project_drag = Some(project.id);', 'self.workspace_project_drag = None;'),
    'closed-live-tabs-are-resurrected': (ROOT / 'src/app.rs', 'self.workspace_seen_sessions\n                    .insert(session.id)', '{ self.workspace_seen_sessions.insert(session.id); true }'),
    'live-count-claims-to-be-history': (ROOT / 'src/app/screens.rs', 'tr("起動中のセッション")', 'tr("履歴")'),
    'launcher-keeps-the-old-project-workspace': (ROOT / 'src/app.rs', '            self.select_workspace_project(project_id);', '            let _ = project_id;'),
    'hidden-history-terminal-is-marked-read': (ROOT / 'src/app/screens.rs', '        if !self.session_library_open {\n            if let Some(WorkspaceTab::Session(id)) = self.workspace.active().cloned()', '        if true {\n            if let Some(WorkspaceTab::Session(id)) = self.workspace.active().cloned()'),
}

# 5. Nothing below this line is filled in.

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
