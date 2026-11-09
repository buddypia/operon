# Session filter chips, an idle tab, and reworded notices, existing layout

Before (ja):
[要対応 2] [実行中 1] [手すき 3] [終了 4]
Tab tooltip on an idle session: 手すき / ターンを終えて手が空いています
Chip tooltip on 手すき: ターンを終えて手が空いているセッション

After (ja):
[要対応 2] [実行中 1] [アイドル 3] [終了 4]
Tab tooltip on an idle session: アイドル / ターンを終え、次の依頼を出せます
Chip tooltip on アイドル: ターンを終え、次の依頼を出せるセッション

Korean before → after: [대응 필요] [실행 중] [유휴] [종료] → [확인 필요] [실행 중]
[유휴] [종료]; idle tooltip 「유휴 / 턴을 마쳤습니다. 다음 요청을 보낼 수 있습니다」.
English chips unchanged; idle tooltip "Idle / Turn finished; ready for your
next request".

Project tab strip, Korean: [개요] [변경] [worktree] [PR] [파일] … and inside
変更: [변경된 파일] [변경 내용] [커밋 기록] (before: [변경된 파일] [차이점] [커밋 기록]).

A notice, English, before → after:
  {chord} is claimed by both \\\"Open\\\" and \\\"Close\\\".
  {chord} is claimed by both "Open" and "Close".

Positions, colours, counts, icons and the order of every chip, tab and button
are unchanged. 「アイドル」 is one character wider than 「手すき」; the sidebar fit
test bounds the four chips. Every other reworded label is no more than two
characters (ja/ko) or four (en) longer than before, most are the same or
shorter; the full list is `wording.md`.
