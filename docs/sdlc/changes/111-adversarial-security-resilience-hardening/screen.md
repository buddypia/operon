# Screen: Adversarial Security, Resilience, and Performance Hardening UI Details

- **Status**: approved

## 1. Run Script TOCTOU Re-Verification & Notice

When the user clicks "承認して実行" in `ui_run_script_modal`, if `.operon/run.sh` on disk differs from the previewed digest:
- The execution is halted immediately.
- The modal preview refreshes to show the new lines.
- A warning notice is displayed:
  "スクリプトの内容が変更されました。更新された内容を再確認してください。"

```
┌─ 実行スクリプトの確認 ────────────────────────────── 520px ─┐
│                                                              │
│  .operon/run.sh を実行しようとしています。                   │
│  内容を確認して実行を承認してください:                       │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ #!/usr/bin/env bash                                    │  │
│  │ # (Updated lines shown here)                           │  │
│  │ npm run dev                                            │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ⚠ スクリプトの内容が変更されました。再確認してください。    │
│                                                              │
│              [ キャンセル ]    [ 承認して実行 ]              │
└──────────────────────────────────────────────────────────────┘
```

## 2. Mainline Checkout Validation Notice

If a worktree landing is attempted while the main checkout is on another branch:
- A clear notification appears:
  "メイン作業ツリーが {mainline} ではなく {current} にあります。メイン作業ツリーで {mainline} をチェックアウトしてからマージしてください。"

## 3. Prompt Template Menu in Launch Form

- In `ui_launch_form`, the "テンプレートを挿入" dropdown no longer performs disk I/O on every frame.
- Menu click lazily loads or references cached templates, ensuring constant 60fps rendering.
