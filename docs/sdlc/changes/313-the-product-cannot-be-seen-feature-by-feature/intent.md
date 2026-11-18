# Intent: the product cannot be seen feature by feature, and no document says what a feature does now

- **Status**: approved
- **Opened**: 2026-11-19

## Problem

The person's request, after seeing the page change 312 produced (2026-11-19):

> 画面構成のUI/UXもわからないし、人間がレビューし、AIがSPECとかBRIEFと進捗が追える文章とかもない。これで人間が全体像を掴めないし、レビューもできないし、開発の進捗も追えない

and the request it answers, as the person first wrote it (2026-11-18):

> 기능지도가 제대로 가이드라인이 되고 있는지? 기존에 있는 프로세스와 제대로 융합되었는지? 인간이 리뷰하고 바로 원하는 장면을 선택해서 상세내용을 알 수 있고(특히 문서), 전체적인 흐름과 진행도도 알 수 있어햐. 전체적으로 한눈에 알아보고 인간이 원할때 상세페이지로 보는 방식이 좋을것 같아. 더 좋은 방식이 있으면 그걸 채용해도 좋아.

Change 312's page lists 308 change tickets by the part of the code they touched,
and each opens its intent, spec and plan, mostly in English. Reading it:

- Nothing shows the product. The screens a person uses (home, projects, the
  session column and terminal, the diff, files, knowledge, the command palette,
  the new-session sheet, settings) appear nowhere, so there is no whole picture
  and no way to say "the part of the product I mean".
- No document says what a feature does now. Each change's spec describes a
  difference from what was there before, so the answer to "what does the diff
  view do today" is the sum of dozens of old deltas. A person has nothing short
  to review; an AI starting a change has nothing current to follow and keep true.
- Nothing shows progress per feature: which parts of the product are steady,
  which are being changed right now and at what stage, which wait for the
  person, and what has been done to each before.

## Who feels it, and when

- The person, coming back to the project and wanting to know what the product
  is made of, what is moving, and what they need to decide — and, before a
  change lands, wanting a short document in their own language to read and judge.
- A session starting a request, which today infers what a feature does from old
  change specs and the code before it can place new behaviour.

## Desired outcome

- A person opens one page and sees the product as its screens, each with a real
  screenshot and the features on it, each feature marked steady, being changed
  (with the stage), or waiting for their decision; and what waits for them is
  first.
- Choosing a feature shows, on the same page, a short Japanese document of what
  it is for, who uses it when, where it is, what it can and cannot do now, and
  which changes touched it; an English document an AI follows; the feature's
  progress history; and its screenshot with the manual's section.
- The change list from change 312 is still there, as a second view.
- A session opening a request reads the feature's two documents before it
  places the work, and a change that alters a feature's behaviour carries the
  update of those documents in the same branch.
- A feature document that has fallen out of date in a way a machine can see is
  caught, and progress shows without anyone typing it into the documents.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English. The brief
  is the one document written for a person, so it is Japanese; the spec is
  English.
- Change 312's rule: the page is generated, never committed, and no second
  record of a change's position is kept.

## Systems likely affected

The feature-pilot and sdlc skills, `docs/sdlc/README.md` and its state template,
the change map script, a new `docs/features/`, and `src/tests.rs`.

## Open questions

Three direction decisions were put to the person and not answered within ten
minutes; the managing session adopted the recommended defaults provisionally,
recorded in `state.yaml`, and holds them for the person's review.

## Not in scope

- Editing the application or its manuals.
- The leftover scripts that expect a domain-map.json under docs/features.
- Translating the briefs into English or Korean.
