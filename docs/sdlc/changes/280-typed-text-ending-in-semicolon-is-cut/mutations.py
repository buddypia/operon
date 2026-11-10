#!/usr/bin/env python3
"""Break how typed text is handed to tmux (change 280)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "typed_280_"]]
TMUX = ROOT / "src/tmux.rs"
FLAGS = 'arguments.extend(["-l", "--"].map(str::to_owned));'
ESCAPED = 'Some(head) => std::borrow::Cow::Owned(format!("{head}\\\\;")),'
MUTATIONS = {
    "text-may-be-read-as-a-flag": (TMUX, FLAGS, 'arguments.extend(["-l"].map(str::to_owned));'),
    "text-is-not-literal": (TMUX, FLAGS, 'arguments.extend(["--"].map(str::to_owned));'),
    "final-semicolon-is-not-escaped": (TMUX, "arguments.push(tmux_literal_argument(text).into_owned());", "arguments.push(text.clone());"),
    "final-semicolon-is-dropped": (TMUX, ESCAPED, "Some(head) => std::borrow::Cow::Owned(head.to_owned()),"),
    "every-semicolon-is-escaped": (TMUX, ESCAPED, 'Some(_) => std::borrow::Cow::Owned(text.replace(\';\', "\\\\;")),'),
    "commands-are-not-separated": (TMUX, '            arguments.push(";".to_owned());\n', ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
