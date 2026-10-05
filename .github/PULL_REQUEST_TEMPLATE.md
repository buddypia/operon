## Summary / 概要 / 요약

<!-- What does this PR change, and why? Any language (English / 한국어 / 日本語). 何が変わり、なぜ必要か。/ 무엇이 바뀌고 왜 필요한가요? -->

## Pipeline artifacts / パイプライン成果物 / 파이프라인 산출물

<!-- Link the committed artifacts, or say which stages were skipped and why.
     See ../docs/sdlc/README.md — "When to skip stages". -->

- Intent: `docs/sdlc/changes/…/intent.md`
- Spec: `docs/sdlc/changes/…/spec.md`
- Plan: `docs/sdlc/changes/…/plan.md`
- Skipped because: <!-- typo / translation / one-line fix / n/a -->

## Type of change / 変更の種類 / 변경 유형

- [ ] Bug fix / バグ修正 / 버그 수정
- [ ] New feature / 新機能 / 새 기능
- [ ] Refactor (no behavior change) / リファクタ（挙動変更なし） / 리팩터링(동작 변경 없음)
- [ ] Documentation / translation / ドキュメント・翻訳 / 문서·번역
- [ ] Build / packaging / CI / ビルド・パッケージ・CI / 빌드·패키징·CI

## Checklist / チェックリスト / 체크리스트

<!-- Items are in English; ticking them is what matters. 項目は英語です。/ 항목은 영어입니다. -->

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --locked -- -D warnings` passes
- [ ] `cargo test --locked` passes
- [ ] Documentation updated in all three languages where applicable
      ([README.md](../README.md), [README.ko.md](../README.ko.md), [README.ja.md](../README.ja.md),
      and the matching `docs/MANUAL*.md` / `CONTRIBUTING*.md`)
- [ ] Theme/colour changes update [`DESIGN.md`](../DESIGN.md) and pass the contrast tests
- [ ] No telemetry, accounts, cloud calls, secrets, or new unsafe code added
- [ ] I agree my contribution is licensed under the [MIT License](../LICENSE)
- [ ] Durability-relevant changes (store schema, cancellation sidecar, tmux stop/retry,
      scan/output limits) respect the documented invariants
- [ ] Cross-CLI restore or shared-session archive changes were also run against
      `cargo test --locked -- --ignored` (or the PR says why they could not be)
- [ ] Reviewed against the passes in [`REVIEW.md`](../REVIEW.md), and this PR says
      which `paused` surface in [`docs/sdlc/risk.yaml`](../docs/sdlc/risk.yaml) it
      touches (or that it touches none)
- [ ] No test was removed and no `#[ignore]` was added (the commit gate checks
      this; if either was deliberate, say which behaviour stopped existing)
- [ ] Steering changes (`CLAUDE.md`, `AGENTS.md`, `.claude/`, a policy document)
      come with the guard that catches the next occurrence, and an entry in
      [`docs/sdlc/lessons.md`](../docs/sdlc/lessons.md) if a mistake prompted them

## Screenshots / スクリーンショット / 스크린샷

<!-- For UI changes, attach before/after screenshots if possible. -->
