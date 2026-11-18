# Intent: a person cannot see where every change stands, or reach its documents, without opening files one by one

- **Status**: approved
- **Opened**: 2026-11-18

## Problem

The request, as the person wrote it (2026-11-18):

> 기능지도가 제대로 가이드라인이 되고 있는지? 기존에 있는 프로세스와 제대로
> 융합되었는지? 인간이 리뷰하고 바로 원하는 장면을 선택해서 상세내용을 알 수
> 있고(특히 문서), 전체적인 흐름과 진행도도 알 수 있어햐. 전체적으로 한눈에
> 알아보고 인간이 원할때 상세페이지로 보는 방식이 좋을것 같아. 더 좋은 방식이
> 있으면 그걸 채용해도 좋아.

The first two questions have a plain answer: no feature map is committed in
this repository. Nothing by that name is on `main` or in the history of any
branch, and no skill, script, or test refers to one. One branch carries the
name — `feature/feature-map-review-surface` — and it holds no commit beyond
`main`; the managing session reports it as the empty working copy it opened
for this same request. Whether it holds uncommitted files was not read. So
nothing committed guides the work or is fused with the process. The rest of
the request describes what is missing.

Before this one the repository holds 307 changes. Each one keeps its position in its own
state file and its reasoning in up to five documents. Nothing shows them
together:

- There is no place to see which changes are waiting for the person, which are
  under way, and how far along each is. A change that stopped to ask a question
  looks, from the file listing, like every finished one.
- Work under way lives in other working copies until it lands, so from the main
  copy it is invisible: every change there reads as finished.
- To review one change a person opens its directory and reads the files in the
  right order; to compare two they do it twice. The documents are the review
  material, and reaching them is the slow part.
- A session deciding where new behaviour belongs has the list of modules, and
  not what earlier changes already did to each one.

## Who feels it, and when

- The person, when they come back to the project and want to know what needs
  them, what moved, and what a given change decided — without asking a session
  to summarise it.
- A session starting a request, when it checks whether the work is already
  under way somewhere and which earlier changes touched the same area.

## Desired outcome

- One page shows every change: what waits for the person first, then what is
  under way and at which stage, then everything by the area of the product it
  touched.
- Choosing a change shows its documents as written — intent, spec, plan, screen
  layout — and the findings left open for the person, on the same page.
- A change under way in another working copy appears with its real position.
- The page is produced from the records the pipeline already keeps, so it
  cannot disagree with them, and no change can be missing from it.
- A session opening a request is told by the pipeline to read the same view
  before it decides resume-or-new and placement.

## Constraints this change inherits

- Local-first: no network, no service; a file a browser opens.
- User-facing text is Japanese; code, comments, and docs are English.
- No second record of a change's position: `state.yaml` and the documents stay
  the only home of every fact (`.claude/skills/feature-pilot/SKILL.md`).
- `docs/sdlc/README.md` records a decision not to build a domain map, an
  ownership index, or HTML review decks. This change departs from part of it and
  must say which part and why.

## Systems likely affected

None of the application's modules. `scripts/` (a new generator),
`.claude/skills/feature-pilot/SKILL.md`, `docs/sdlc/README.md`, `src/tests.rs`.

## Open questions

- Does the person have a feature map somewhere else — another repository, an
  uncommitted file, the working copy of `feature/feature-map-review-surface`
  — that this should have been built on? Asked on 2026-11-18, unanswered. Adopted default: none exists, build from the records; the change
  stops before landing so the person can say otherwise.

## Not in scope

- Removing the scripts under `.claude/scripts/lib/` that still expect a domain
  map and a review deck. That directory is gate configuration, where removing
  code is a person's decision; it is a separate change.
- Showing the map inside the Operon application.
- Editing a change, approving it, or answering its question from the page.
