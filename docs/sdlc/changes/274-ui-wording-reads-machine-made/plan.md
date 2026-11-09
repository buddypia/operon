# Plan: the app's wording is rewritten where it reads as machine-made

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | R1's and R5's tests move to the new words; R2's and R3's tests are new; assertions quoting a re-keyed id follow it |
| `src/i18n_tables.rs` | the 188 messages of `wording.md`; both tables re-sorted by message id |
| `src/ui/widgets.rs` | the idle group id, the idle chip hint, the idle tab hint |
| `src/app.rs`, `src/app/screens.rs`, `src/app/knowledge.rs`, `src/history.rs`, `src/cli.rs`, `src/files.rs`, `src/git.rs`, `src/agents.rs`, `src/agents/accounts.rs`, `src/store.rs` | the re-keyed Japanese literals at their call sites, nothing else |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | the reworded labels they quote; the idle state-table rows |
| `docs/sdlc/changes/274-ui-wording-reads-machine-made/` | this change's record, `wording.md`, `verification.yaml`, `mutations.py` |

## Order of work

1. Write R2's and R3's tests and move R1's and R5's to the new words; run them
   and watch all four fail on the unchanged tables.
2. Apply `wording.md` with one script: replace each re-keyed id as an exact
   string literal (`"old"` → `"new"`) across `src/`, set the English and Korean
   rows, re-sort each table by the unescaped message id, and refuse to run if a
   new id collides with an existing one or a placeholder set differs.
3. Change the documents; R5 passes.
4. Run the named tests, the table invariants, and the two layout tests; fix any
   test that quoted part of a reworded message.
5. Mutation sweep, `make q.fast`, the store model and the launch-gate proof
   that the touched surfaces ask for.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| a re-keyed id left behind at a call site | a raw Japanese id shown in English or Korean | `every_message_the_code_shows_has_a_row_in_every_table`, `every_message_row_is_one_the_code_shows` |
| a row out of order after re-keying | its translation is silently skipped | `every_translation_table_is_sorted_by_message_id` |
| a placeholder renamed or dropped | a literal `{name}` on screen | `every_translation_keeps_the_placeholder_names_of_its_message_id`, the script's own check |
| a test quoting part of an old message | a red test unrelated to behaviour | step 4's run of the named tests, then CI's full suite |
| a longer label clipped | a chip or tab cut off | the sidebar fit loop over the four group words; labels capped at +2 (ja/ko) / +4 (en) characters |
| a store or durability notice that promises more or less than before | a person told data is lost when it is on disk | the spec's durability row; rows 8, 9, 218, 467, 536, 942 keep "could not confirm", none says "failed" |
| a translation drifting from the Japanese meaning | two languages say different things | `wording.md` read by the evaluator against the Japanese |
| a store, launch-gate or git source edited beyond its literals | a behaviour change hidden in a wording diff | the diff of those files is string-literal lines only, checked before commit; the store model and launch-gate proof re-run |

## Proof of completion

- The acceptance commands in `spec.md` pass.
- `mutations.py` catches every mutation.
- `cargo test --locked model_check_store_survives_every_interruption_of_a_write`
  passes, and CI's verify job proves
  `proof_safe_agent_command_admits_no_line_break`.

## Departures from the plan

- **One message id split.** 「手順」 labelled two things: the knowledge kind and
  the tool-setup button in Settings. Its English row moved from "Setup" to
  "Procedure" for the knowledge kind, which would have mislabelled the button,
  so the button now reads a new id 「導入手順」 ("Setup", 「설치 방법」) and
  `docs/MANUAL.ja.md` names it. The tables gain one row each; `wording.md`
  lists the rewritten messages and not this new one.
- **One more rewording after the evaluator's pass.** The empty-state hint
  became 「要対応になったセッションは、ここの一番上に表示されます」 (the
  reviewer's 「ここの先頭に」 read stiffly). `wording.md` was regenerated.
- **A guard for the durability claim.** The plan's risk row for the
  `CommittedButNotSynced` notices named only the spec; the claim now has a test,
  `a_written_but_unconfirmed_notice_never_says_the_write_failed`, which reads
  every 「書き込まれましたが」 notice in three languages and refuses "failed" in
  any of them. Two mutations in `mutations.py` watch it go red.
- **Acceptance commands.** `cargo test` takes one filter before `--`, so the
  spec's multi-name commands run as `cargo test --locked -- a b c`;
  `verification.yaml` names one test per obligation.
- **Documents.** Beyond the labels the spec names, `docs/MANUAL.md`'s English
  labels moved to sentence case with the app (Insert template, Open in external
  editor, AI review, Create manual checkpoint, Roll back to checkpoint, Review
  run script, Open an empty terminal, the notifications setting), and the
  Korean manual follows 키 바인딩, 변경 내용, 외부 편집기, 빈 터미널, 스테이징
  and 실행 명령어. Prose outside quoted labels is unchanged.
- **Bands.** `largest_module_lines` reads 9976 in `src/app/screens.rs`, which
  was 9978 on `origin/main`; the diagnose-tier breach predates this change.
- **`src/agents.rs` left untouched.** The first CI run proved every obligation
  but I6, `cargo kani --tests --harness proof_safe_agent_command_admits_no_line_break`,
  which the launch-gate surface asks for once `src/agents.rs` changes. Kani
  did not build: 177 errors under `RUSTFLAGS=-D warnings` in test code this
  change does not touch (the first, `unused variable: why` at
  `src/tests.rs:73325`). The only edit there was one permission-mode
  description, so its Japanese id keeps 「確認なしにファイルとコマンドを実行します。」
  and the Kani build is left to a change of its own. `wording.md` and the
  spec's R4 count 187 messages / 228 cells; the plan's file table above still
  names `src/agents.rs`.
- **Four tests quoted parts of reworded messages.** CI's full suite failed
  `history_partial_results_are_disclosed_even_during_automatic_discovery`
  (上限付き検索 / 上限付きスキャン), `one_durable_cancellation_copy_is_enough_to_allow_stop_and_repair_the_other`
  (1 つ永続化されました), `prevents_removing_a_record_required_by_a_queued_session`
  (依存されています) and `the_git_sub_tabs_are_translated_message_ids` (차이점);
  each now quotes the new wording. The pre-push scan matched whole messages
  and missed these fragments.
- **The durability guard was widened on the independent verifier's evidence.**
  It survived two mutations: a notice outside 「書き込まれましたが」
  (「取り込んだメタデータをディスクに保存できたか、まだ確認できていません。」)
  rewritten as "…failed", and "could not be saved" beside "confirm". It now
  reads every message about a save or write it could not confirm (11 rows)
  and refuses a longer list of failure words in each language; both mutations
  are in `mutations.py` (20 of 20 caught).
