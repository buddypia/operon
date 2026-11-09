# Intent: the app's wording reads as machine-made in all three languages

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person wrote: 「言葉の表現がいまいち。てすきとか微妙だな。韓国語、日本語、英語を
正しくAI Slopっぽいない文章にして。」 — the wording falls short, the idle status
word 「手すき」 (chosen in change 259) feels off, and the Korean, Japanese and
English text should be correct and stop reading like machine output.

Read row by row, the three languages fail in different ways. Japanese is mostly
natural, but carries an old-fashioned status word, error messages that end in a
bare noun (「…のエンコード: {error}」), and stiff technical nouns where a plain
verb reads better (「ディスク永続化」). English mixes "land" and "merge" for the
same action, strays into Title Case, and keeps calques ("Level with", "Send them
in two goes", "Attempting to run"). Korean has the most: literal calques of the
Japanese (「枠」 → 창, 「鮮度」 → 신선도, ハンク → 덩어리, 差分 → 차이점), the status
word 대응 필요, a stray Japanese character inside a Korean word (병合합니다),
Japanese brackets left in a sentence (「停止」), and escaped quotes that reach the
screen as backslashes.

## Who feels it, and when

Anyone reading the session list, tooltips, notices or error messages, in any of
the three languages: a word that reads as odd or translated makes the app feel
unfinished, and a term used two ways ("land"/"merge", 차이점/변경 사항) makes a
person wonder whether two different things happen.

## Desired outcome

Every row a person reads says what it means the way a native writer of a
developer tool would put it, with one term per concept in each language, and the
mistakes that only a reader of that language would notice — stray script from
another language, a broken escape, a doubled word — are gone.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English.
- The idle word must not contain 「待」 / 「대기」 / "waiting": only 要対応 reads
  as waiting on the person (change 259), and 「待機中」 names a queued session
  (change 221).
- Meaning does not change: no fact, condition, or promise is added or dropped.
- Short labels sit in fixed-width chips, tabs and buttons.

## Systems likely affected

The message tables, the status group word and hints, the tests that quote
messages, and the READMEs and manuals in three languages wherever they quote a
reworded label.

## Open questions

None: the person asked for the whole text; which rows change and to what is a
reversible wording judgement recorded in `wording.md`.

## Not in scope

- Rewriting the READMEs' and manuals' own prose beyond the labels they quote.
- Which states belong to which status group.
- Re-shooting screenshots under `docs/images/`.
