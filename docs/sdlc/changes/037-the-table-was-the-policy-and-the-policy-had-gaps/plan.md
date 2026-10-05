# Plan: the gate stops depending on the table being complete

- **Spec**: `./spec.md`
- **Approved**: 2026-09-08
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/agents.rs` | dashed-token normalisation, the heuristic, `PRE_APPROVING_ARGUMENTS`, the mode arm |
| `src/app.rs` | innermost root, `document_may_have_a_diff`, `named()` through `tf!` |
| `src/app/screens.rs` | the two diff call sites ask the shared question |
| `src/i18n_tables.rs` | one id, `{p0} · {p1}` |
| `src/tests.rs` | seven guards |
| `README.md`, `README.ja.md`, `README.ko.md` | the sentence 036 narrowed can say what the gate now does |

## Order of work

1. `src/agents.rs`: `launch_argument_tokens` yields `(dashed, text)`; the flag,
   alias and heuristic arms require `dashed`, the mode arm does not. Guard
   `a_value_that_spells_a_flag_is_not_a_flag` watched failing first.
2. `src/agents.rs`: the heuristic, with the table override.
   `a_permission_escape_no_table_names_still_asks` and
   `a_table_that_calls_a_flag_safe_outranks_the_heuristic`.
3. `src/agents.rs`: `PRE_APPROVING_ARGUMENTS` and the value read.
4. `src/app.rs`: innermost root. The existing 034 guards must stay green — that
   is the check that this did not change any case they pinned.
5. `src/app.rs` + `src/app/screens.rs`: `document_may_have_a_diff`, called from
   all three places.
6. `src/app.rs` + `src/i18n_tables.rs`: `named()` through `tf!`.
7. The three READMEs.
8. Mutation-verify, run the gates and the bands, then both reviews.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The heuristic overrides a table that says a flag is safe | `--allow-dangerously-skip-permissions` demands a checkbox nobody should sign | `a_table_that_calls_a_flag_safe_outranks_the_heuristic`, driven from `CLAUDE_FLAGS` rather than from a literal |
| Requiring a dash loses a spelling 036 closed | `agy -dangerously-skip-permissions` stops being gated | The 036 guards stay, and they carry both spellings |
| The innermost root changes a case 034 pinned | A worktree outside the project stops being the worktree's | 034's guards are unchanged and must stay green |
| `document_may_have_a_diff` is stricter than the view | A document that could show a diff stops being offered one | `one_question_decides_whether_a_document_may_have_a_diff` drives the real function from all three call sites |
| `Bash` matched loosely | `--allowed-tools Bashful` asks | Matched on the value, and the guard names both the asking and the not-asking case |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked --all-targets -- -D warnings` — no output.
- `bash scripts/check-bands.sh` — no new breach.
- Seven guards, each watched failing.

## Departures from the plan

Filled in during implementation.
