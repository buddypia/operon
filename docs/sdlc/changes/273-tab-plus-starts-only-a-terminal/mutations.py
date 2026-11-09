#!/usr/bin/env python3
"""Break the promises of change 273: the + asks what to start, an agent starts
in the pane with its remembered options, the danger acknowledgement still
holds, a missing CLI cannot be chosen, and the tabs are compact."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [
    ["cargo", "test", "--locked", "session_273_"],
    ["cargo", "test", "--locked", "session_254_"],
    ["cargo", "test", "--locked", "session_260_"],
]
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
GLYPHS = ROOT / "src/glyphs.rs"
MUTATIONS = {
    # R1: the + starts a terminal outright again instead of opening the menu.
    "plus-starts-terminal-directly": (
        SCREENS,
        "            ui.memory_mut(|memory| memory.toggle_popup(popup));\n",
        "            self.launch_workspace_terminal(group);\n",
    ),
    # R3/R4: an agent row starts a plain terminal.
    "agent-row-starts-a-terminal": (
        SCREENS,
        "Some(agent) => self.launch_workspace_agent(group, agent),",
        "Some(_agent) => self.launch_workspace_terminal(group),",
    ),
    # R3: a pin for another CLI is applied to this one.
    "pin-for-another-cli-applies": (
        APP,
        "            .filter(|pin| pin.agent == agent);\n",
        "            .filter(|_| true);\n",
    ),
    # R3: the options the sheet last used for this CLI are ignored.
    "remembered-options-ignored": (
        APP,
        ".or_else(|| self.recent_agent_settings.agents.get(agent).cloned())",
        ".or_else(|| None)",
    ),
    # R3: the new tab lands in whichever pane had the focus.
    "agent-tab-ignores-the-pane": (
        APP,
        "        if let Some(group) = group {\n            self.workspace.focused = Some(group);\n        }\n        self.launch_progress = Some(LaunchProgress {",
        "        self.launch_progress = Some(LaunchProgress {",
    ),
    # R3: the quick launch records the sheet's draft as remembered options.
    "quick-launch-records-the-draft": (
        APP,
        "        self.remember_session_account(id, start.account);\n",
        "        self.remember_session_account(id, start.account);\n        self.persist_recent_agent_settings();\n",
    ),
    # R4: the danger acknowledgement is skipped.
    "danger-check-skipped": (
        APP,
        "        if !acknowledged\n            && launch_needs_acknowledgement(",
        "        if false\n            && launch_needs_acknowledgement(",
    ),
    # R4 (the claim): the sheet opens, but on the CLI the form was already on.
    "sheet-opens-on-the-wrong-cli": (
        APP,
        "                    self.selected_agent = agent.to_owned();\n",
        "",
    ),
    # R4: the fallback switches CLI through select_agent, which rewrites the file.
    "fallback-records-the-form": (
        APP,
        "                    self.selected_agent = agent.to_owned();\n                    self.apply_agent_launch_settings(agent);\n",
        "                    self.select_agent(agent);\n",
    ),
    # R5: a CLI this Mac does not have can be chosen.
    "missing-cli-enabled": (
        SCREENS,
        "(agent_choice_copy(choice).0, tools.agent_available(choice))",
        "(agent_choice_copy(choice).0, true)",
    ),
    # R5: the terminal row is judged like an agent and comes out disabled.
    "terminal-row-disabled": (
        SCREENS,
        "(agent_short_label(choice), true)",
        "(agent_short_label(choice), tools.agent_available(choice))",
    ),
    # R7: the old tab margins.
    "old-tab-margins": (
        SCREENS,
        ".inner_margin(egui::Margin::symmetric(6, 2))",
        ".inner_margin(egui::Margin::symmetric(8, 4))",
    ),
    # R7: the tab row takes the shared 28px height inside its margin.
    "tab-row-shared-height": (
        SCREENS,
        "                                            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_SMALL;\n",
        "",
    ),
    # R7: the close button grows with the shared padding again.
    "close-button-padded": (
        SCREENS,
        "                                            ui.spacing_mut().button_padding = egui::Vec2::ZERO;\n",
        "",
    ),
    # R7: the state mark back at full size.
    "state-mark-full-size": (
        GLYPHS,
        "pub(crate) const TAB_STATUS_MARK_SIZE: f32 = 14.0;",
        "pub(crate) const TAB_STATUS_MARK_SIZE: f32 = 18.0;",
    ),
    # F1: the Escape that closes the menu reaches the terminal as well.
    "menu-escape-passed-on": (
        SCREENS,
        "        if launch_menu_open\n            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))\n",
        "        if launch_menu_open && false\n",
    ),
    # The + names itself as it used to: the claim of what it does.
    "plus-claims-a-plain-terminal": (
        SCREENS,
        'icon_button(ui, palette, ICON_ADD, tr("ターミナルや AI を起動"))\n                }).inner;',
        'icon_button(ui, palette, ICON_ADD, tr("空のターミナルを開く"))\n                }).inner;',
    ),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
