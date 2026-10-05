# Harden identifier check and harness traversal

## Problem

An adversarial review of `steering_and_source_do_not_name_external_projects`, `harness_documents`, and `source_files` in `src/tests.rs` revealed:
1. `entry.path().is_dir()` blindly traverses symlinks, allowing symlink loop DoS or repository traversal escapes.
2. `filter(|(path, _)| !path.ends_with("tests.rs"))` created an uninspected blind spot over ~40,000 lines of test code.
3. `text.to_lowercase()` allocated a new full-document heap String per file per check, generating significant memory churn.
4. `contains(name)` without word boundary validation risked false positive matches on unrelated English words.
5. `source_files()` used raw `unwrap()`, crashing without informative diagnostics on unreadable or invalid files.
6. Test failure diagnostics were opaque (`path: pattern matched` with no line numbers, column, matched pattern, or context).

## Proposed Change

- Update `harness_documents` and `source_files` to verify `file_type.is_symlink()` and skip symlinks, preventing cyclic loops and jailbreak traversal.
- Add descriptive error diagnostics on file read failures instead of bare `unwrap()`.
- Implement `find_word_boundary_match` with zero heap allocation (`eq_ignore_ascii_case`) and token boundary checking.
- Include `src/tests.rs` in the source files scan while explicitly exempting only self-referential pattern definitions.
- Strip invisible/zero-width evasion characters before inspection.
- Report detailed failure context: file, line, column, pattern name, and line snippet.

## Risk and Review

Touches only test harness utilities in `src/tests.rs`. Preserves 100% backward compatibility with existing function signatures and sorted ordering contracts.
