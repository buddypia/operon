#!/usr/bin/env python3
"""Break the promise that the tab keys every Mac app answers to work here."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "shortcut_272_"],
]
APP = ROOT / "src/app.rs"
KEYMAP = ROOT / "src/app/keymap.rs"
SCREENS = ROOT / "src/app/screens.rs"
MANUAL_JA = ROOT / "docs/MANUAL.ja.md"
MUTATIONS = {
    # R2: the exact binding no longer wins, so ⇧⌘T is also ⌘T.
    "lenient-match-wins": (
        KEYMAP,
        "if modifiers.matches_exact(chord.modifiers) {",
        "if false && modifiers.matches_exact(chord.modifiers) {",
    ),
    # R5: ⇧⌘] as macOS reports it (`}`) is not read as ].
    "shifted-bracket-unread": (
        KEYMAP,
        "| (egui::Key::CloseBracket, egui::Key::CloseCurlyBracket)",
        "| (egui::Key::CloseBracket, egui::Key::CloseBracket)",
    ),
    # R5/R7, the claim: 「次のタブへ」 moves off the key the manual and the bracket test name.
    "next-tab-moved-off-its-key": (
        KEYMAP,
        'id: "tab.next",\n        label: "次のタブへ",\n        default: "Mod+Shift+]",',
        'id: "tab.next",\n        label: "次のタブへ",\n        default: "Mod+Alt+]",',
    ),
    # R5: ⌃Tab is read before ⌃⇧Tab and swallows it, as before change 272.
    "control-tab-swallows-shift": (
        APP,
        "let back = ctx.input_mut(|input| {\n"
        "                    input.consume_key(\n"
        "                        egui::Modifiers::CTRL | egui::Modifiers::SHIFT,\n"
        "                        egui::Key::Tab,\n"
        "                    )\n"
        "                });\n"
        "                let forward =\n"
        "                    ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Tab));",
        "let forward =\n"
        "                    ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Tab));\n"
        "                let back = ctx.input_mut(|input| {\n"
        "                    input.consume_key(\n"
        "                        egui::Modifiers::CTRL | egui::Modifiers::SHIFT,\n"
        "                        egui::Key::Tab,\n"
        "                    )\n"
        "                });",
    ),
    # R3: ⌘W acts on whatever page is showing.
    "close-ignores-the-page": (
        APP,
        "if self.page == Page::Sessions && !self.session_library_open {\n"
        '                if self.chord_consumed(ctx, "tab.close") {',
        "if true {\n"
        '                if self.chord_consumed(ctx, "tab.close") {',
    ),
    # R4: a closed tab is not remembered.
    "close-not-remembered": (
        APP,
        "self.closed_tabs.push(tab.clone());",
        "",
    ),
    # R4: a tab closed twice is remembered twice.
    "close-remembered-twice": (
        APP,
        "self.closed_tabs.retain(|closed| closed != tab);",
        "",
    ),
    # R4: the list grows for as long as the window is open.
    "closed-list-unbounded": (
        APP,
        "self.closed_tabs.drain(..excess);",
        "let _ = excess;",
    ),
    # R4: a removed session's tab comes back for nothing.
    "reopen-of-a-removed-session": (
        APP,
        "if !self.store.sessions.iter().any(|session| session.id == *id) {",
        "if false {",
    ),
    # R4, the claim: nothing to reopen says nothing.
    "nothing-to-reopen-is-silent": (
        APP,
        'self.notice_briefly(tr("開き直せるタブはありません。"));',
        "",
    ),
    # R6: the palette row does nothing.
    "palette-reopen-row-inert": (
        SCREENS,
        'PaletteKind::Action("Reopen closed tab") => self.reopen_closed_tab(),',
        "",
    ),
    # R7, the claim: the Japanese manual stops telling a person about ⇧⌘T.
    "manual-forgets-reopen": (
        MANUAL_JA,
        "| ⇧⌘T | 閉じたタブを開き直す |",
        "| (なし) | 閉じたタブを開き直す |",
    ),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
