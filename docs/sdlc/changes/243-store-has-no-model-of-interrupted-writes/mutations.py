#!/usr/bin/env python3
"""Change 243's mutation sweep: break the store, not the model, and require the
model to go red. Also the change's `empirical-mutation` obligation, so CI runs
it again — ROOT is found from this file.
"""
import sys
from pathlib import Path

# docs/sdlc/changes/<this>/mutations.py -> the checkout.
ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

CHECKS = [[
    "cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--",
    "--exact", "tests::model_check_store_survives_every_interruption_of_a_write",
]]

STORE = ROOT / "src/store.rs"
WRITE = """        file.write_all(contents)?;
        sync_file(&file)?;
        drop(file);
        fs::rename(&temporary, path)?;
"""

MUTATIONS = {
    # The rename publishes bytes that are not on disk yet.
    "rename-before-the-file-sync": (
        STORE,
        WRITE,
        """        file.write_all(contents)?;
        fs::rename(&temporary, path)?;
        sync_file(&file)?;
        drop(file);
""",
    ),
    # The rename publishes an empty file.
    "rename-before-the-write": (
        STORE,
        WRITE,
        """        fs::rename(&temporary, path)?;
        file.write_all(contents)?;
        sync_file(&file)?;
        drop(file);
""",
    ),
    # Durable is reported before the directory holds the new name.
    "directory-sync-before-the-rename": (
        STORE,
        """        fs::rename(&temporary, path)?;
        let directory_sync =
            fs::File::open(parent).and_then(|directory| sync_directory(&directory));
""",
        """        let directory_sync =
            fs::File::open(parent).and_then(|directory| sync_directory(&directory));
        fs::rename(&temporary, path)?;
""",
    ),
    # A newer build's index is read as this build's, then saved over.
    "newer-schema-read-as-current": (
        STORE,
        "    if store.schema_version > STORE_SCHEMA_VERSION {",
        "    if false {",
    ),
    # An unreadable index is deleted instead of moved aside.
    "unreadable-index-deleted": (
        STORE,
        "            match fs::rename(path, &corrupt_file) {",
        "            match fs::remove_file(path) {",
    ),
    # The legacy import writes nothing, so a Mac upgrading starts empty.
    "import-skips-the-store": (
        STORE,
        "    let store_outcome =\n        write(current, &store_bytes)",
        "    let store_outcome =\n        write(&current_sidecar, &store_bytes)",
    ),
    # The fallback for an unreadable legacy snapshot overwrites it.
    "import-fallback-overwrites-the-legacy-snapshot": (
        STORE,
        "            let store_outcome = write(current, fallback.as_bytes())",
        "            let _ = fs::write(legacy, fallback.as_bytes());\n            let store_outcome = write(current, fallback.as_bytes())",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
