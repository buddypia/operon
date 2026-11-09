#!/usr/bin/env python3
"""The mutation sweep for change 265. Runner and its rules:
scripts/mutation_sweep.py. Each mutation breaks one thing the change claims;
the last group mutates the sentences a person reads.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "protected_folders_265",
    "discovers_native_codex_sessions_only_for_the_project",
    "a_dangerous_switch_cannot_be_launched_without_the_acknowledgement",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

TERMINAL = ROOT / "src" / "ui" / "terminal.rs"
CONFIG = ROOT / "src" / "config.rs"
CLI = ROOT / "src" / "cli.rs"
SCRIPT = ROOT / "scripts" / "package-macos.sh"
SCREENS = ROOT / "src" / "app" / "screens.rs"
TABLES = ROOT / "src" / "i18n_tables.rs"

MUTATIONS = {
    # Fix 2: the terminal link probe.
    "probe-refusal-removed": (
        TERMINAL,
        "    if is_in_protected_folder(&candidate, home) && !is_inside_folder(&candidate, base_dir) {",
        "    if false && is_in_protected_folder(&candidate, home) && !is_inside_folder(&candidate, base_dir) {",
    ),
    "probe-refusal-reads-the-resolved-path": (
        TERMINAL,
        "    let candidate = lexically_normal(&candidate);\n    if is_in_protected_folder(&candidate, home)",
        "    let candidate = lexically_normal(&candidate);\n    let candidate = candidate.canonicalize().unwrap_or(candidate);\n    if is_in_protected_folder(&candidate, home)",
    ),
    "probe-dotdot-not-normalized": (
        TERMINAL,
        "    let candidate = lexically_normal(&candidate);\n    if is_in_protected_folder",
        "    let candidate = candidate;\n    if is_in_protected_folder",
    ),
    "probe-compares-with-case": (
        TERMINAL,
        "PathBuf::from(path.to_string_lossy().to_lowercase())",
        "PathBuf::from(path.to_string_lossy().to_string())",
    ),
    "probe-volumes-forgotten": (
        TERMINAL,
        "    under(Path::new(PROTECTED_VOLUMES_ROOT))\n        || home",
        "    home",
    ),
    "probe-data-volume-spelling-forgotten": (
        TERMINAL,
        "    let path = match path.strip_prefix(&data_volume) {",
        "    let path = match path.strip_prefix(Path::new(\"/nowhere-265\")) {",
    ),
    "probe-desktop-forgotten": (
        CONFIG,
        '&["Desktop", "Documents", "Downloads"]',
        '&["Documents", "Downloads"]',
    ),
    "probe-project-exemption-removed": (
        TERMINAL,
        "&& !is_inside_folder(&candidate, base_dir) {",
        "&& true {",
    ),
    "probe-exemption-ignores-canonical-folder": (
        TERMINAL,
        "    path.starts_with(lexically_normal(folder))\n        || folder",
        "    path.starts_with(lexically_normal(folder))\n        || false && folder",
    ),
    # Fix 3: records compared as spelled.
    "codex-cwd-resolved-again": (
        CLI,
        "        allowed_paths.contains(&cwd).then(|| CliSession {",
        "        let cwd = cwd.canonicalize().unwrap_or(cwd);\n        allowed_paths.contains(&cwd).then(|| CliSession {",
    ),
    "antigravity-history-workspace-resolved-again": (
        CLI,
        "        if !allowed_paths.contains(Path::new(workspace)) {",
        "        if !allowed_paths.contains(&Path::new(workspace).canonicalize().unwrap_or_default()) && !allowed_paths.contains(Path::new(workspace)) {",
    ),
    "antigravity-log-workspace-resolved-again": (
        CLI,
        "        .any(|candidate| allowed_paths.contains(Path::new(candidate)))",
        "        .any(|candidate| allowed_paths.contains(&Path::new(candidate).canonicalize().unwrap_or_default()) || allowed_paths.contains(Path::new(candidate)))",
    ),
    "registered-folders-canonical-only": (
        CLI,
        "            [Some(folder), canonical]",
        "            [None, canonical]",
    ),
    # Fix 1: the bundle explains the prompt.
    "bundle-downloads-key-dropped": (
        SCRIPT,
        'plutil -insert NSDownloadsFolderUsageDescription -string "$folder_access_reason" "$plist"\n',
        "",
    ),
    "bundle-network-volumes-key-dropped": (
        SCRIPT,
        'plutil -insert NSNetworkVolumesUsageDescription -string "$folder_access_reason" "$plist"\n',
        "",
    ),
    # The claims a person reads.
    "claim-reason-stops-hedging": (
        SCRIPT,
        'folder_access_reason="多くの場合、Operon のターミナルで',
        'folder_access_reason="Operon のターミナルで',
    ),
    "claim-reason-drops-the-way-out": (
        SCRIPT,
        "心当たりがなければ「許可しない」を選んでください。",
        "",
    ),
    "claim-warning-drops-personal-folders": (
        SCREENS,
        "プロジェクトの外（書類・ダウンロード・デスクトップなど）のファイルも読み書きできます。",
        "ファイルを変更できます。",
    ),
    "claim-warning-loses-its-english-row": (
        TABLES,
        '("警告: 確認もサンドボックスもなしに、プロジェクトの外（書類・ダウンロード・デスクトップなど）のファイルも読み書きできます。", "Warning:',
        '("警告: 確認もサンドボックスもなしにファイルを変更できます。", "Warning:',
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
