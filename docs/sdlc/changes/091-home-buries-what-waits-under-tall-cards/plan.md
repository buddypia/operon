# Plan: Home shows what waits first, and one line per session

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | `ui_home`: 要対応 line always (zero card / warning card with waiting rows), tiles unchanged, recent list and プロジェクトから始める in two columns (stacked under 720 px); new `home_session_row` with action and ··· menu |
| `src/i18n_tables.rs` | EN and KO rows for the ids in spec "Behaviour" |
| `src/tests.rs` | the three tests named in the spec's Acceptance |
| `README.md`, `README.ja.md`, `README.ko.md` | only if they describe Home's session cards |

## Order of work

1. `home_session_row`, then `ui_home` restructure. Compiles.
2. i18n rows; tests.
3. Gates, mutation check of each new test, rust-reviewer, commit, package,
   install, look at the screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row wraps to two lines | Home as tall as before | the one-line test compares the drawn heights of title, project, and 「開く」 |
| A card action is lost | cannot delete or close from Home | every card control is listed in the ··· menu by the spec; the rust-reviewer diff read |
| Internal details still on the face | `tmux:` visible | the one-line test asserts no `tmux:`/`cwd:` text is drawn |
| The zero line is missing or shown with waiters | Home cannot say "nothing waits" | `home_always_says_whether_anything_waits` both ways |
| The project button opens the wrong project | a session starts elsewhere | the project test clicks the second of two |
| Row click and button click both fire | 開く plus a stray selection | `clickable_card` senses behind its children, as the card relied on |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The three new tests, each watched failing with its rule inverted.
- Installed app: Home as in `./screen.md`.

## Departures from the plan

Found while building:

- The `list` salt in `home_session_row` (spec req 3) cannot be watched
  failing: a waiting row is drawn inside the 要対応 card's child `Ui`, and
  egui derives a scope's id from its parent's, so the two rows of one session
  already have different ids with the salt removed (mutation: `"waiting"` →
  `"recent"`, test stayed green). The salt stays as the stated intent; the
  no-clash assertion guards a future move of the waiting rows out of the card.
- `drawn_text_rect` gained a sibling `drawn_text_rects` (one rect per match):
  the project name is drawn on the row and in プロジェクトから始める, and a union
  of the two is no rect at all.
- Home is drawn at `click_at`'s 900×600 in the new tests, so a rect read from
  the frame is where the click lands.
- README ×3: the Home line names one-line rows and per-project starts; the
  banner sentence says it is always shown.

From review round 1 (rust-reviewer, one Important):

- A truncated label takes all the width it is offered, so a long title pushed
  the project and branch over the time and the action. The row now offers
  title, project, and branch fixed shares (`capped_label`); test
  `a_long_title_is_cut_short_before_the_time_and_the_action`, watched red with
  the cap removed.
- Fixing the same pattern in プロジェクトから始める found a second defect: a
  `right_to_left` layout inside the card's vertical `Ui` took the whole
  remaining height, so the first project's card filled the column and pushed
  the rest off screen. The row is wrapped in `horizontal`;
  `starting_from_a_project_on_home_opens_that_projects_launch_form` watched red
  without it.
- The waiting and recent lists collect references, sort, and clone only the
  rows drawn (at most five and six), not every session per frame.
