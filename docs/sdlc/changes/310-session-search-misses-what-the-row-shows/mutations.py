#!/usr/bin/env python3
"""Mutation sweep for change 310: 「セッションを検索」 reads what the row shows
— the listed title, the agent as written and as stored, the project, the
branch — trimmed and without case, the same in the list and the grid; and
「ローカル履歴」 says "none" only for the text of a finished search, and
"searching" only while one runs under a field that still holds text."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = ["search_310_", "f310_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # R1
    "the-listed-title-is-not-searched": (
        SCREENS,
        "            || has(&session_list_title(\n",
        "            || false && has(&session_list_title(\n",
    ),
    "the-stored-agent-word-is-not-searched": (
        SCREENS,
        "            || has(&session.agent)\n",
        "",
    ),
    "the-agent-chip-is-not-searched": (
        SCREENS,
        "            || has(agent_short_label(&session.agent))\n",
        "",
    ),
    "the-product-name-is-not-searched": (
        SCREENS,
        "                && has(agent_choice_copy(&session.agent).0))\n",
        "                && false)\n",
    ),
    # F2: the picker's word for an agent it does not list is not on the row.
    "an-unlisted-agent-is-found-by-the-pickers-word": (
        SCREENS,
        '            || (matches!(session.agent.as_str(), "codex" | "claude" | "gemini")\n',
        "            || (true\n",
    ),
    "the-branch-is-not-searched": (
        SCREENS,
        "            || session.branch.as_deref().is_some_and(has)\n",
        "",
    ),
    "another-projects-name-keeps-the-session": (
        SCREENS,
        "                .any(|project| project.id == session.project_id && has(&project.name))\n",
        "                .any(|project| project.id != session.project_id && has(&project.name))\n",
    ),
    "the-search-reads-letter-case": (
        SCREENS,
        "        let has = |text: &str| text.to_lowercase().contains(needle);\n",
        "        let has = |text: &str| text.contains(needle);\n",
    ),
    "spaces-around-the-words-count": (
        SCREENS,
        "        self.search.trim().to_lowercase()\n",
        "        self.search.to_lowercase()\n",
    ),
    # R2
    "the-grid-keeps-every-session": (
        SCREENS,
        "                            && self.session_matches_search(session, &needle)\n",
        "                            && (true || self.session_matches_search(session, &needle))\n",
    ),
    "the-list-keeps-every-session": (
        SCREENS,
        "            .filter(|session| self.session_matches_search(session, &needle))\n",
        "            .filter(|session| true || self.session_matches_search(session, &needle))\n",
    ),
    # R3
    "none-is-said-of-text-nobody-searched": (
        SCREENS,
        "        } else if self.transcript_searched.as_deref() == Some(self.transcript_search_input.trim())\n"
        "            && self.knowledge.session_hits.is_empty()\n",
        "        } else if self.knowledge.session_hits.is_empty()\n",
    ),
    "a-finished-search-is-not-remembered": (
        APP,
        "                    self.transcript_searched = Some(query.clone());\n",
        "",
    ),
    # F1: matches an empty submit cleared are not a search that found nothing.
    "an-empty-submit-leaves-the-search-remembered": (
        APP,
        "            self.transcript_matches.clear();\n            self.transcript_searched = None;\n",
        "            self.transcript_matches.clear();\n",
    ),
    # R4
    "a-running-scan-reads-none": (
        SCREENS,
        "            .contains(&BackgroundKey::TranscriptSearch)\n            || self\n",
        "            .contains(&BackgroundKey::WorkspaceScan)\n            || self\n",
    ),
    "a-running-indexed-search-reads-none": (
        SCREENS,
        "                .contains(&BackgroundKey::KnowledgeSessionSearch)\n        {\n",
        "                .contains(&BackgroundKey::WorkspaceScan)\n        {\n",
    ),
    "searching-is-said-under-an-emptied-field": (
        SCREENS,
        "        } else if self.transcript_search_input.trim().is_empty() {\n",
        "        } else if false {\n",
    ),
    # The claim: the line a running search draws must not be the "none" answer.
    "a-running-search-is-told-there-was-nothing": (
        SCREENS,
        '            ui.label(RichText::new(tr("履歴を検索しています…")).weak());\n',
        '            ui.label(RichText::new(tr("一致する履歴はありませんでした。")).weak());\n',
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
