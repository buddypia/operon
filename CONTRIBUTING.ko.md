[English](CONTRIBUTING.md) | [日本語](CONTRIBUTING.ja.md) | 한국어

# Operon에 기여하기

기여를 고려해 주셔서 감사합니다. 이슈, Pull Request, 번역 모두 환영합니다.
English / 한국어 / 日本語 중 어느 언어로 작성하셔도 됩니다.

## 개발 환경

- macOS (Apple Silicon 또는 Intel)
- [rustup](https://rustup.rs/)으로 설치한 Rust
- `tmux` (세션 실행에 필요하며, 일부 테스트도 tmux 관련 로직을 실행합니다)
- 실행 경로를 직접 확인하려면 agent CLI 하나 이상(`codex`, `claude`, `agy`)

```sh
git clone https://github.com/buddypia/operon.git
cd operon
cargo run --release
```

로컬 세션 인덱스는 한 번에 하나의 인스턴스만 잡을 수 있습니다. 소스에서
실행할 때는 설치된 빌드를 종료하고, 반대의 경우도 마찬가지입니다.

## PR을 열기 전에

CI와 동일한 검사를 실행하세요.

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

6개 테스트는 인증된 `codex`, `claude`, `agy`와 `tmux`가 머신에 있어야 하므로
`#[ignore]` 상태입니다. 이 테스트들은 CLI 간 복원에 대한 유일한 end-to-end
검증이며 CI에서는 실행할 수 없습니다. 변경이 복원이나 공유 세션 아카이브를
건드린다면 로컬에서 실행하고 PR에 그 사실을 밝혀 주세요.

```sh
cargo test --locked -- --ignored
```

`Cargo.lock` 변경은 의도한 것으로 한정하고, 가능하면 관련 없는 수정과 분리해
주세요.

## Pull request 절차

1. 변경용 branch를 만드세요. `main`에 직접 commit하지 않습니다.
2. 변경을 만들고 앞 절의 검사를 실행합니다.
3. PR을 열고 [PR 템플릿](.github/PULL_REQUEST_TEMPLATE.md)을 채웁니다.
   파이프라인 산출물 링크(생략했다면 그 이유), 체크리스트를 작성하고 UI 변경에는
   스크린샷을 첨부하세요.
4. [REVIEW.md](REVIEW.md)의 패스에 따른 리뷰에 응답합니다. merge 승인은 사람이
   합니다.
5. 사용자 문서를 건드리는 변경이라면 세 언어 버전을 함께 갱신합니다
   (규칙 절 참조).

## Commit 메시지

기존 히스토리처럼 짧은 Conventional Commits 스타일의 제목을 사용하세요.
`type(scope): 무엇을, 왜 바꿨는지` 형태이며, 예를 들어
`fix(tmux): the suite runs tmux on a server of its own` 또는
`docs: package-macos.sh builds dist/Operon.app` 와 같이 씁니다. 주요 type은
`feat`, `fix`, `docs`, `chore`, `test`입니다. 다음 절의 파이프라인을 생략했다면
commit 메시지에 그렇게 적어 주세요.

## 변경을 만드는 방식

규모가 있는 변경은 [docs/sdlc/README.md](docs/sdlc/README.md)의 파이프라인을
따릅니다. 먼저 `intent.md`(해결책이 아니라 문제)를 commit하고, 다음으로
`spec.md`(무엇을 만족해야 하는지, 그리고 이 프로젝트의 어떤 정책에 닿는지),
마지막으로 `plan.md`(어떤 파일을 어떤 순서로, 무엇으로 완료를 증명하는지)를
씁니다. 한 변경의 산출물은 `docs/sdlc/changes/` 아래에 함께 놓입니다.

사용자에게 보이는 동작을 추가하거나, 두 개 이상의 모듈을 건드리거나, 저장되는
형태를 바꾸거나, 의존성을 추가하거나, 정책을 바꿀 때 사용하세요. 오타, 번역
수정, 한 줄 수정에는 생략해도 되며, 그 경우 commit 메시지에 그렇게 적어 주세요.

[REVIEW.md](REVIEW.md)는 리뷰 정책입니다. 어떤 패스를 도는지, Important와 Nit의
경계, 제외 대상, 그리고 merge 전에 사람이 반드시 읽어야 하는 변경이 적혀
있습니다.

## 규칙

- **문서는 세 언어로 관리합니다.** 사용자 문서를 변경할 때는
  [README.md](README.md), [README.ko.md](README.ko.md),
  [README.ja.md](README.ja.md)를 함께 갱신해 주세요. 구조를 동일하게 유지하면
  리뷰가 쉬워집니다.
- **디자인 토큰의 기준은 한 곳입니다.** 색·테마 변경은 상수와 함께
  [`DESIGN.md`](DESIGN.md)를 갱신하고 WCAG 2.1 AA 명암비 테스트를 통과해야
  합니다.
- **내구성이 중요합니다.** 저장소 스키마, 마이그레이션/롤백, 취소 사이드카,
  tmux 정지/재시도, 스캔·출력 상한을 바꾸는 변경에는 추가 테스트와 그 변경이
  다루는 실패 양상에 대한 설명이 필요합니다.
- **로컬 퍼스트는 말 그대로 로컬 퍼스트입니다.** 텔레메트리, 계정, 클라우드
  호출은 없습니다. 새 의존성은 이 기준으로 신중하게 검토합니다.
- **시크릿은 금지입니다.** API 키, 토큰, 개인정보를 commit하지 마세요.

## 기여물의 라이선스

기여물을 제출하면 그것이 프로젝트의 나머지 부분과 동일한
[MIT License](LICENSE)로 라이선스된다는 데 동의하는 것으로 간주합니다.

## 버그와 취약점 신고

버그는 이 저장소의 **Issues** 탭에서 버그 리포트 템플릿으로 제보해 주세요.
보안 취약점은 [SECURITY.ko.md](SECURITY.ko.md)를 따르며, 공개 이슈로 올리지
마세요.
