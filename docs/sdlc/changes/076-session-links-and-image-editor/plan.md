# Plan: open generated files from terminal via Cmd+click, make session tree icons interactive, and preview images in editor

- **Spec**: `./spec.md`
- **Approved**: 2026-09-24
- **Status**: complete

## Files that change

| File | Change |
|---|---|
| `Cargo.toml` | Enable `jpeg`, `gif`, `webp` features on `image` dependency. |
| `src/models.rs` | Add `Image` variant to `FileOpenOutcome` with bytes, width, and height. |
| `src/git.rs` | Update `read_file_for_editing` to recognize and decode image files into `FileOpenOutcome::Image`. |
| `src/app.rs` | Store image preview data in `OpenDocument`, handle `FileOpenOutcome::Image` in `apply_opened_file`, and add synchronous fallback resolution in `open_terminal_target`. |
| `src/ui/files.rs` | Add click sensing on folder disclosure chevrons, folder icons, and file icons in `file_tree_rows`. |
| `src/ui/terminal.rs` | Ensure image extensions are recognized and improve path cache probing. |
| `src/app/screens.rs` | Render image preview view in `ui_editor` with dimensions, file size, responsive image display, and external opener. |
| `src/i18n_tables.rs` | Add Japanese translations for image dimensions and size formatting if needed. |
| `src/tests.rs` | Unit and integration tests for dynamic terminal link opening, file tree icon clicks, image loading, and editor preview rendering. |

## Order of work

1. **Cargo.toml and dependencies**:
   Update `image` crate features to `["png", "jpeg", "gif", "webp"]`.
2. **Outcome Model and File Reading**:
   Update `FileOpenOutcome` in `src/models.rs` and `read_file_for_editing` in `src/git.rs` to decode and load images.
3. **App State and Image Document**:
   Update `OpenDocument` in `src/app.rs` to retain image data and dimensions. Update `open_terminal_target` to dynamically resolve missing paths on click.
4. **File Tree Icon Click Interactivity**:
   Update `file_tree_rows` in `src/ui/files.rs` so clicking chevrons/folders dispatches `Toggle` and clicking file icons dispatches `Open`.
5. **In-Editor Image Preview UI**:
   Update `ui_editor` in `src/app/screens.rs` to render responsive image preview when document has image data.
6. **Testing and Verification**:
   Write targeted unit and regression tests in `src/tests.rs`. Run `cargo fmt`, `cargo test --locked`, and `cargo clippy --locked`.
7. **Packaging & Atomic App Replace**:
   Package macOS app via `bash scripts/package-macos.sh` and atomically update `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Corrupt or malformed image panic | Panic on decode | `image::load_from_memory` is guarded with `match` and error handling. |
| Huge image OOM | UI stutter or high memory | Max byte ceiling check (`EDITOR_FILE_MAX_BYTES` / bounded size) before decoding. |
| Terminal click steals focus on invalid target | Terminal loses focus unexpectedly | Only valid open modifiers and resolvable targets navigate. |
| Tree row layout shift when making icons clickable | Tree item indentation drifts | Sense is added to existing icon labels without changing dimensions. |

## Proof of completion

- `cargo fmt --check` — clean exit.
- `cargo test --locked` — all tests pass, zero failed.
- `cargo clippy --locked -- -D warnings` — clean exit without warnings.
- New tests:
  - `test_session_file_tree_icon_clicks_toggle_and_open`
  - `test_open_terminal_target_dynamic_fallback_resolution`
  - `test_read_file_for_editing_image_outcome`
  - `test_in_editor_image_preview_rendering`
- Packaging script `bash scripts/package-macos.sh` succeeds.
- App replaced at `/Applications/Operon.app`.

## Departures from the plan

*(None yet)*
