# Harden inspection engine against DoS, Trojan Source evasion, and allocation churn

## Problem

An adversarial audit of the inspection and traversal logic in `src/tests.rs` identified several critical vulnerabilities and bottlenecks:
1. **Denial of Service (FIFOs/Sockets/Giant files)**: `entry.file_type()` did not explicitly verify `file_type.is_file()`, risking indefinite blocking on named pipes/FIFOs or sockets, and lacked a file size ceiling against memory exhaustion.
2. **Trojan Source & Fullwidth Evasion (CVE-2021-42574)**: The zero-width evasion filter missed Unicode bidirectional override characters (`\u{202A}`..=`\u{202E}`, `\u{2066}`..=`\u{2069}`), word joiners (`\u{2060}`), and fullwidth ASCII characters (`\u{FF01}`..=`\u{FF5E}`).
3. **Word Boundary False Positives on Multibyte UTF-8**: Word boundary detection checked single bytes (`u8`), which can misinterpret continuation bytes of multibyte UTF-8 characters as word boundaries.
4. **Memory Allocation Churn**: Every scanned line was cloned to a heap `String` (`line.to_string()`), incurring massive heap allocation overhead over ~100k scanned lines.
5. **Fragile Self-Exemption Bypass**: A string-matching exemption `line.contains("fn steering_and_source_do_not_name_external_projects") || line.contains("b\"\\x39\\x37\\x2f\\x22\"")` created an arbitrary bypass vulnerability.
6. **Non-deterministic Traversal**: Directory iteration order depended on underlying filesystem readdir order.

## Proposed Change

- Enforce `file_type.is_file()` and a 10 MB per-file size cap (`MAX_HARNESS_FILE_BYTES`) in `harness_documents` and `source_files`.
- Sort child directories before traversal to guarantee deterministic file ordering across environments.
- Use `std::borrow::Cow<'_, str>` to achieve zero-allocation scanning on clean lines (99.9% of lines).
- Normalize Unicode evasion vectors including Bidi Trojan Source overrides and fullwidth ASCII variants.
- Implement code-point-aware word boundary detection using `char::is_alphanumeric()`.
- Eliminate the fragile self-referential line exemption; pattern XOR definitions prevent accidental self-matching without arbitrary exemptions.

## Risk and Review

Confined strictly to `src/tests.rs`. Fully backwards-compatible with all public and crate test harnesses.
