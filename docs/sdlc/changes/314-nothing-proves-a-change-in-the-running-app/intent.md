# Intent: nothing proves a change in the running application

- **Status**: draft
- **Opened**: 2026-11-23

## Problem

The person's request (2026-11-23), naming the pstack plugin and the notes of
Lauren Tan's talk "All Code Is Slop (Except Mine)" as what to analyse:

> 아래의 분석대상을 제대로 분석 검토후에 이 매커니즘을 제대로 내 프로젝트에 적용시켜줘. 요즘 핫한 AI개발이라서 최대한 내 프로젝트에 적용해서 융합시키고 싶어.

Both sources describe one ordering of where trust in an agent's work comes
from: first what the code base itself refuses (design, types, lints, tests),
then checks of the running program and of every case of a model, and last a
thin layer of written rules. Read against that ordering, this repository
already has the first layer's tests, the every-case checks, lessons that each
carry a guard, and a document per feature. What it does not have is the middle
of the second layer:

- Every piece of evidence a change records is a test run. Nothing starts the
  application that was built, uses it the way a person does, and keeps what the
  window showed. A change that paints is agreed as an ASCII drawing before the
  code and is never looked at afterwards except by the person, by hand.
- The feature documents say what each feature does and which tests pin it. They
  do not say how to reach the feature in the window or what on the screen shows
  that it works, so two sessions asked to "check it in the app" would do two
  different things, and neither could be repeated.
- Nothing starts the application apart from the person's own sessions and
  terminals. A session that simply started a second copy to look at its change
  would be refused, or would be working inside the person's real data.

## Who feels it, and when

- The person, each time a change that alters a screen lands: the first time the
  real window is looked at is after it is installed, by them.
- A session finishing a change to a screen, which can say only that the tests
  pass, and a later session asked whether a feature still works, which has no
  repeatable way to find out.

## Desired outcome

- One command starts the application just built, apart from the person's data
  and terminals, uses one feature as written down for that feature, saves
  pictures of the window and what the window said at each step, stops what it
  started, and says pass or fail.
- The steps for a feature are written beside that feature's document, in the
  words the screen shows, and a step that names something the application no
  longer shows is caught without starting the application.
- Running it never changes the person's sessions, settings, or terminals, and
  never stops a program it did not start.
- A session building a change that paints is told to run it and to keep the
  pictures, and the person can open them.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The text every turn loads is at its ceiling; nothing may be added to it.

## Systems likely affected

No module of the application. The development process: a script, the feature
documents, the feature-pilot skill's references, and the suite.

## Open questions

None for the person. Whether the window can be operated without taking the
keyboard from the person was settled by trying it (it can, through the
accessibility tree the application already publishes).

## Not in scope

- The other two gaps the analysis found: a lint layer that names the fix in its
  error, and comments that state rules nothing enforces. Each is its own change.
- Running these checks in CI: a hosted runner has no permission to operate a
  window.
- Steps for every feature. This change writes them for the features it can
  prove today; the rest are added as each feature is next changed.
- Any automation that runs agents unattended on top of this.
