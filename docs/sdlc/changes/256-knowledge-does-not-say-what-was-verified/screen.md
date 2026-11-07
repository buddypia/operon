# Screen: 検証済み / 未確認 on knowledge entries

- **Change**: `256-knowledge-does-not-say-what-was-verified`
- **Status**: approved

## Where it lives

One existing surface, プロジェクト › ナレッジ in `src/app/knowledge.rs`: the list
row (`ui_knowledge_entry_row`) and the open entry's heading. No new tab, modal,
control, icon, colour role, or layout change.

## ASCII wireframes

### List row

Before:

```text
▸ 原因  共有の一時ディレクトリがテスト間で残る        ▮▮▮▮▮▮▯▯▯▯ 参照 3  Claude Code
```

After:

```text
▸ 原因  検証済み  共有の一時ディレクトリがテスト間で残る  ▮▮▮▮▮▮▯▯▯▯ 参照 3  Claude Code
▸ 手順  未確認    リリース前に署名を確認する              ▮▮▮▯▯▯▯▯▯▯ 参照 0  Codex
```

### Open entry heading

```text
← グループ一覧
原因  検証済み  共有の一時ディレクトリがテスト間で残る
▮▮▮▮▮▮▯▯▯▯ 参照 3
```

## Tokens

- Label size 12 (an existing `typography` level, the same as the kind label).
- 検証済み in `text_muted`; 未確認 in `text_faint` — quieter, since it is the
  default for every entry kept before this change. Neither borrows a status role.
- Spacing: the row's existing `SPACE_SM` item spacing.
