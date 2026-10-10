[English](CONTRIBUTING.md) | 日本語 | [한국어](CONTRIBUTING.ko.md)

# Operon へのコントリビュート

コントリビュートをご検討いただきありがとうございます。Issue、Pull Request、
翻訳のいずれも歓迎します。English / 한국어 / 日本語のどれで書いていただいても
構いません。

## 開発環境

- macOS（Apple Silicon または Intel）
- [rustup](https://rustup.rs/) で導入した Rust
- `tmux`（セッションの起動に必要。一部のテストも tmux 関連のロジックを実行します）
- 起動経路を手動で確かめる場合は、agent CLI を 1 つ以上（`codex`、`claude`、`agy`）

```sh
git clone https://github.com/buddypia/operon.git
cd operon
cargo run --release
```

ローカルのセッションインデックスを保持できるインスタンスは同時に 1 つだけです。
ソースから実行するときはインストール済みのビルドを終了してください（その逆も同様です）。

## PR を出す前に

CI と同じ検査を実行してください。

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

6 件のテストは、認証済みの `codex`、`claude`、`agy` と `tmux` が手元に必要なため
`#[ignore]` されています。これらは CLI 間の復元を end-to-end で確かめる唯一の
テストで、CI では実行できません。変更が復元や共有セッションアーカイブに触れる
場合は、ローカルで実行し、その旨を PR に書いてください。

```sh
cargo test --locked -- --ignored
```

`Cargo.lock` の変更は意図したものに限り、可能な限り無関係な変更とは分けてください。

## Pull request の進め方

1. 変更用の branch を作成します。`main` に直接 commit しないでください。
2. 変更を加え、前節の検査を実行します。
3. PR を開き、[PR テンプレート](.github/PULL_REQUEST_TEMPLATE.md)を埋めます。
   パイプラインの成果物へのリンク（省いた場合はその理由）、チェックリスト、
   UI の変更ならスクリーンショットを添えてください。
4. [REVIEW.md](REVIEW.md) のパスに沿ったレビューに対応します。merge の承認は
   人が行います。
5. ユーザー向けドキュメントに触れる変更では、3 言語版を同時に更新します
   （「慣習」を参照）。

## Commit メッセージ

既存の履歴と同じく、短い Conventional Commits 形式の件名を使ってください。
`type(scope): 何を、なぜ変えたか` の形で、例えば
`fix(tmux): the suite runs tmux on a server of its own` や
`docs: package-macos.sh builds dist/Operon.app` のように書きます。主な type は
`feat`、`fix`、`docs`、`chore`、`test` です。次節のパイプラインを省いた場合は、
commit メッセージにそう書いてください。

## 変更の進め方

規模のある変更は [docs/sdlc/README.md](docs/sdlc/README.md) のパイプラインを
通します。まず `intent.md`（解決策ではなく問題）を commit し、次に `spec.md`
（何を満たすべきか、このプロジェクトのどのポリシーに触れるか）、最後に
`plan.md`（どのファイルをどの順で変更し、何をもって完了とするか）を書きます。
1 つの変更の成果物は `docs/sdlc/changes/` の下にまとめて置きます。

ユーザーに見える挙動を追加する、複数のモジュールに触れる、永続化される形を
変える、依存関係を追加する、ポリシーを変える — このいずれかに当たるときに
使ってください。タイポ、翻訳の修正、1 行の修正では省いて構いません。その場合は
commit メッセージにそう書いてください。

[REVIEW.md](REVIEW.md) はレビュー方針です。どのパスを回すか、Important と Nit の
境界、対象外とするもの、merge の前に人が必ず読むべき変更が書かれています。

## 慣習

- **ドキュメントは 3 言語で管理します。** ユーザー向けドキュメントを変更する
  ときは、[README.md](README.md)、[README.ko.md](README.ko.md)、
  [README.ja.md](README.ja.md) を同時に更新してください。構成を揃えておくと
  レビューしやすくなります。
- **Changelog を更新します。** ユーザーに影響する重要な変更やリリースノートは
  [CHANGELOG.md](CHANGELOG.md) に記録します。
- **デザイントークンの正は 1 か所です。** 色・テーマの変更では、定数と併せて
  [`DESIGN.md`](DESIGN.md) を更新し、WCAG 2.1 AA のコントラストテストを通して
  ください。
- **耐久性を重視します。** ストアスキーマ、マイグレーション/ロールバック、
  キャンセルサイドカー、tmux の停止/再試行、走査・出力の上限に触れる変更には、
  追加のテストと、扱う失敗モードの説明が必要です。
- **ローカルファーストを徹底します。** テレメトリ、アカウント、クラウド呼び出しは
  ありません。新しい依存関係はこの観点で精査します。
- **シークレットは含めません。** API キー、トークン、個人情報を commit しないで
  ください。

## コントリビューションのライセンス

コントリビューションを提出することで、それがプロジェクトの他の部分と同じ
[MIT License](LICENSE) でライセンスされることに同意したものとみなします。

## バグと脆弱性の報告

バグはこのリポジトリの **Issues** タブから、バグ報告テンプレートを使って報告して
ください。セキュリティ上の脆弱性は [SECURITY.ja.md](SECURITY.ja.md) に従い、
公開の Issue は立てないでください。
