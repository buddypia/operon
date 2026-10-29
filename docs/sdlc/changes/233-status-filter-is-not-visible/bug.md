# Follow-up: preserve history controls after central tab migration

CI 40898842279 on ebb7eea reports 1041 passed, 28 failed, 6 ignored. The failure list includes legacy column/pinned-row render tests, legacy global editor_jump_line assertions, dock session-list placement tests, and history-grid bounds. See the authoritative CI run for full details.

Named causes: normal workspace deliberately no longer draws session rows, so tests of those rows must target their history/list home rather than assert that the old global left sidebar survives. File jumps now use exact DocumentId keys, so assertions must preserve the targeted line semantics. show_status_group still opens the normal workspace without the list and its filter controls. History currently draws both ui_sessions and the advanced column sequentially, risking clipping and duplicate navigation.

Do not weaken tests into merely passing: retain rename, stop/remove, waiting message, pinned dedup, grid bounds, jump identity, dock inspector moves, and new project-only sidebar coverage. Distinguish stale expectations from actual unreachable controls. No persistence/CLI/restore contract changes are needed.
