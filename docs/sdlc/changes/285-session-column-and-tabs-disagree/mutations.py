#!/usr/bin/env python3
"""Break the one list behind the column and the tabs, and the pin (change 285)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "session_285_"],
    ["cargo", "test", "--locked", "workspace_project_"],
    ["cargo", "test", "--locked", "sidebar_25"],
]
APP = ROOT / "src/app.rs"
ORDER = ROOT / "src/app/session_order.rs"
SCREENS = ROOT / "src/app/screens.rs"
TREE = ROOT / "src/ui/session_tree.rs"
MUTATIONS = {
    "column-draws-every-session-again": (ORDER, "        let mut rows = tabs\n            .session_ids()\n", "        let _ = tabs;\n        let mut rows = self.store.sessions.iter().map(|s| s.id)\n"),
    "column-drops-finished-tabs": (SCREENS, "                    .flat_map(|project| self.project_tab_sessions(project.id))\n                    .filter(|session| self.session_matches_filters(session))", "                    .flat_map(|project| self.project_tab_sessions(project.id))\n                    .filter(|session| self.session_matches_filters(session) && self.session_status_kind_for(session).group() != SessionStatusGroup::Finished)"),
    "other-project-session-opens-in-the-shown-one": (ORDER, "            if owner == project {", "            if owner == project || true {"),
    "queued-session-gets-no-tab": (ORDER, "                    SessionStatus::Queued\n                        | SessionStatus::Starting", "                    SessionStatus::Starting"),
    "deleted-tab-kept-in-a-project-not-shown": (ORDER, "                WorkspaceTab::Session(id) => sessions.iter().any(|s| s.id == *id),\n                WorkspaceTab::Document(_) => true,", "                WorkspaceTab::Session(_) => true,\n                WorkspaceTab::Document(_) => true,"),
    "tabs-keep-their-own-order": (ORDER, "        self.workspace.arrange(rank);", "        let _ = &rank;"),
    "sort-moves-past-a-document-only-when-adjacent": (TREE, ".find(|i| session(&group.tabs[*i]).is_some())", ".find(|_| true)"),
    "sort-moves-the-front-tab": (TREE, "                    group.active = at;", "                    let _ = at;"),
    "row-drag-has-no-targets": (SCREENS, "                                    session_targets.push(id, row.rect);", "                                    let _ = row.rect;"),
    "tab-drop-not-followed": (SCREENS, "                                                            self.follow_tab_drop(*group);\n", ""),
    "pin-not-first": (TREE, "        !arrangement.pinned.contains(&session.id),", "        false,"),
    "pin-not-saved": (TREE, "    #[serde(default)]\n    pub(crate) pinned: Vec<Uuid>,", "    #[serde(skip)]\n    pub(crate) pinned: Vec<Uuid>,"),
    "row-menu-does-not-pin": (SCREENS, "\n                                        toggle_pin = true;", "\n                                        let _ = toggle_pin;"),
    "tab-menu-does-not-pin": (SCREENS, "                                                            toggle_pin = true;", "                                                            let _ = toggle_pin;"),
    "pinned-tab-closes": (APP, "            if self.is_session_pinned(*id) {", "            if false && self.is_session_pinned(*id) {"),
    "pinned-tab-shows-its-close-button": (SCREENS, "if matches!(tab, WorkspaceTab::Session(id) if self.is_session_pinned(*id)) {", "if false && matches!(tab, WorkspaceTab::Session(id) if self.is_session_pinned(*id)) {"),
    "tiles-count-sessions-without-a-row": (APP, " && self.session_has_a_tab(session.id)", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
