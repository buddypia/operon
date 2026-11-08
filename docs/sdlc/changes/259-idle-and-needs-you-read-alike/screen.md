# Session filter chips and an idle tab, existing layout

Before:
[要対応 2] [実行中 1] [指示待ち 3] [終了 4]
Tab tooltip on an idle session: 指示待ち / 次の依頼を待っています
Chip tooltip on 指示待ち: 起動していて、次の依頼を待っているセッション

After:
[要対応 2] [実行中 1] [手すき 3] [終了 4]
Tab tooltip on an idle session: 手すき / ターンを終えて手が空いています
Chip tooltip on 手すき: ターンを終えて手が空いているセッション

Korean: [대응 필요] [실행 중] [유휴] [종료]. Positions, colours, counts and the
order of chips are unchanged; the new word is one character shorter than the
old one, so the sidebar fit test still bounds it.
