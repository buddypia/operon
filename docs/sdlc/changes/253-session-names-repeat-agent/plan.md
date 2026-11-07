# Plan: Short session names and direct rename

- **Spec**: `./spec.md`
- **Status**: approved

## Files that change

- src/app.rs: numbered launch defaults, shared workspace title projection, editor location flag and displayed rename seed.
- src/app/screens.rs: shared titles; inline editor, double click and context menu in sidebar and tabs.
- src/i18n_tables.rs: numbered-name translation in every language.
- src/tests.rs: numbered names, legacy titles, real pointer rename/save/cancel and persistence checks.

## Order of work

1. Evaluate and commit intent, spec, screen and plan before source edits.
2. Write failing tests for numbering and gestures, then implement.
3. Run named tests, make q.fast, mutation sweep and independent empirical verification.
4. Push and wait for CI; land from main and package; ask before quitting the running app for installation.

## Risks

- Sidebar and tab draw the same session: use a memory-only editor location flag so focus belongs to one editor.
- Custom names resemble a generated fallback: require matching workspace basename, matching provider and empty initial request, retaining current topic precedence.
- Stored numbered names in another language collide: check all UI translations when allocating a number.
- UI events swallowed by a containing row: test real pointer events and keyboard input.

## Proof of completion

Named tests and mutations pass; q.fast passes; independent verifier reruns evidence; CI passes before merge; package builds; installed app checked after restart approval.

## Departures from the plan

None.
