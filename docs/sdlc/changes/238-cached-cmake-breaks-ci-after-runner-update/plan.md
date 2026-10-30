# Plan: ランナーイメージ更新後も CI の CMake キャッシュが壊れないようにする

- **Spec**: n/a (`bugfix` route — see `docs/sdlc/routes.yaml`)
- **Approved**: 2026-10-30
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Root cause

GitHub Actions の `macos-latest` (`macos-26-arm64`) ランナーイメージが `20261021.0445.8` から `20261026.0455.2` に更新された際、プリインストールされた Rust ツールチェーン（`1.88.0` / `1.99.1`）は同一のまま、Homebrew の OpenSSL が `/opt/homebrew/Cellar/openssl@4/4.0.3` から `/opt/homebrew/Cellar/openssl@4/4.1.0` に更新され、`4.0.3` ディレクトリが削除された。

`Swatinem/rust-cache@v2` は OS・CPU アーキテクチャ・Rust コンパイラ版数・`Cargo.lock` からキャッシュキーを計算するため、旧イメージ `20261021.0445.8` で保存されたプライマリキャッシュ `v0-rust-test-Darwin-arm64-014c4b9f-a449688a` が新イメージ `20261026.0455.2` でも完全一致（`full match: true`）で復元される。

復元された `target/debug/build/llama-cpp-sys-2-*/out/build/CMakeCache.txt` には旧イメージの `/opt/homebrew/Cellar/openssl@4/4.0.3/lib/libssl.dylib` 等が記録されており、`llama-cpp-sys-2` の `build.rs` は `.always_configure(false)` で `cmake --build` (`make install`) を呼ぶ。このとき CMake バイナリ自体のタイムスタンプ更新により `make` が既存 `CMakeCache.txt` のまま CMake 再生成を走らせるが、`FindOpenSSL.cmake` はキャッシュ変数 `OPENSSL_SSL_LIBRARY` が非空のため再検索を行わず、`EXISTS "/opt/homebrew/Cellar/openssl@4/4.0.3/lib/libssl.dylib"` が偽となるため `OpenSSL::SSL` インポートターゲットを定義しない。結果として `llama.cpp` 内の `vendor/cpp-httplib/CMakeLists.txt` で `Target "cpp-httplib" links to: OpenSSL::SSL but the target was not found.` が発生し、プライマリキー完全一致のため壊れたキャッシュが更新されないまま新イメージに割り当てられたジョブが毎回失敗する。

Change 230（Lesson 102）はローカル worktree 作成時の `scripts/seed-target.sh` で `target/debug/build/*/out/build/CMakeCache.txt` を退避したが、CI のキャッシュ復元経路および不要な OpenSSL 検出自体には対処していなかった。

## Files that change

| File | Change |
|---|---|
| `scripts/seed-target.sh` | `reset_seeded_cmake_caches` を `target/*/build/*/out/build/CMakeCache.txt`（`debug` / `release` 等すべてのプロファイル）対象に拡張し、`--reset-only <root>` でキャッシュ復元後のツリー単体でも呼び出せるようにする。 |
| `.github/workflows/ci.yml` | ワークフロー `env` に `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON` を設定し、`checks`・`test`・`package` の各 `Swatinem/rust-cache@v2` の `key` に `${{ env.ImageOS }}-${{ env.ImageVersion }}-no-openssl` を渡してランナーイメージ更新時にプライマリキーを分離し、キャッシュ復元直後に `bash scripts/seed-target.sh --reset-only .` を実行する。 |
| `.github/workflows/harness.yml` | `evals` ジョブにも同様に `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON`、`Swatinem/rust-cache@v2` の `key: ${{ env.ImageOS }}-${{ env.ImageVersion }}-no-openssl`、およびキャッシュ復元直後の `bash scripts/seed-target.sh --reset-only .` を設定する。 |
| `src/tests.rs` | ガードテスト `ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake` を追加し、(1) `ci.yml` と `harness.yml` の全 Rust キャッシュ利用ジョブが `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON`・`ImageOS`/`ImageVersion` キー・`seed-target.sh --reset-only .` を備えること、(2) 実際の CMake + `find_package(OpenSSL)` プロジェクトで旧 OpenSSL パスが消えたキャッシュからそのまま再生成すると `OpenSSL::SSL` 未検出で失敗し、`seed-target.sh --reset-only` と `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=ON` で `debug`/`release` 両方が再構成に成功すること（シンボリックリンク先の外部ディレクトリは変更しないこと）を検証する。 |
| `docs/sdlc/lessons.md` | Lesson 102 に続く再発防止として、CI キャッシュとランナーイメージ更新・ネイティブ CMake 外部依存に関する教訓を追加する。 |

## Order of work

1. `src/tests.rs` にガードテスト `ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake` を追加し、修正前に実行して失敗（RED）を確認する。
2. `scripts/seed-target.sh` に `--reset-only <root>` モードを追加し、`target/*/build/*/out/build/CMakeCache.txt` を退避するように更新する。
3. `.github/workflows/ci.yml` と `.github/workflows/harness.yml` に `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON`、`Swatinem/rust-cache@v2` の `key: ${{ env.ImageOS }}-${{ env.ImageVersion }}-no-openssl`、および `bash scripts/seed-target.sh --reset-only .` を追加する。
4. 新規ガードテストと既存の関連テストを実行し、すべて PASS（GREEN）することを確認する。
5. ミューテーション検証を実施し、`docs/sdlc/lessons.md` に教訓を追加して `every_lesson_is_admitted_with_its_rule_and_a_guard_that_ran`・`cargo fmt --check`・`cargo clippy --locked -- -D warnings`・`bash scripts/check-bands.sh`・独立レビュアー検証を通す。

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| フォールバック（`restore-keys`）で旧ランナーイメージのキャッシュが復元された際に `CMakeCache.txt` が残る | `llama-cpp-sys-2` が旧イメージのパスや設定を引き継ぐ | キャッシュ復元直後の `bash scripts/seed-target.sh --reset-only .` と `ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake` |
| `package` ジョブの `target/release` キャッシュが `target/debug` 限定の退避から漏れる | `cargo build --release --locked` で旧 `CMakeCache.txt` が残る | `scripts/seed-target.sh` の `target/*/build/*/out/build/CMakeCache.txt` グロブ化とテストでの `debug`/`release` 両検証 |
| シンボリックリンクされた `target` や `build` ディレクトリに対して `--reset-only` がリンク先を書き換える | 共有キャッシュ元の `CMakeCache.txt` が消える | `reset_seeded_cmake_caches` の `[ -L ... ]` ガードとテストでの symlink 保護アサーション |

## Proof of completion

- `cargo fmt --check` — 出力なし。
- `cargo test --locked ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake` — 修正前に失敗（RED）し、修正後に `test result: ok. 1 passed; 0 failed` となること。
- `cargo test --locked cloned_native_cache_is_reconfigured_without_changing_its_source` — `test result: ok. 1 passed; 0 failed`。
- `cargo test --locked every_lesson_is_admitted_with_its_rule_and_a_guard_that_ran` — `test result: ok. 1 passed; 0 failed`。
- `cargo clippy --locked -- -D warnings` — 警告・エラーなし。
- `bash scripts/ci-verified.sh fix/238-cached-cmake-breaks-ci-after-runner-update` — GitHub Actions CI の `checks`・`test`・`package` 全ジョブが成功すること。

## Departures from the plan

None yet.
