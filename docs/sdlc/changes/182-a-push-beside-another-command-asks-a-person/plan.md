# Plan: a push the session can rewrite is not put to a person

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-10-20
- **Status**: done

## The rule

The publishing gate asks a person only when the push itself is a decision:
a force, a `+` or `:` refspec, `--mirror`, `--delete`, `--all`, another remote,
or anything the guard cannot read as one of the shapes below. A push that goes
where a plain push goes but is written so the guard cannot see that — joined to
other commands with `&&`, `||`, `;`, `|` or a newline, a redirect such as
`2>&1`, or with the remote and branch left implicit — is refused with `deny`
and a reason that names the form that passes: `git push origin <branch>`, alone
in its own call. The session reruns it; no person is asked. `-q`/`--quiet` is
admitted in the plain form, since it changes only what is printed.

`deny` rather than `allow` for the joined form: the plain form's guarantee is
that the whole command is that push, so nothing the guard has not read runs
with it — `git config remote.origin.mirror true && git push origin x` stays out.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/guard-bash.sh` | `plain_push` admits `-q`/`--quiet`; a second shape check per segment decides `deny` (rewrite it) versus `ask` (a decision) |
| `src/tests.rs` | `a_plain_push_to_origin_is_not_asked_about` gains allowed `-q` cases and a denied list, and checks the deny reason names `git push origin` |
| `docs/sdlc/lessons.md` | entry 080, with its Guard |

## Order of work

1. Commit this plan, `state.yaml`, and `approvals.log`.
2. Test cases. Allowed: `git push -q origin feature/x`,
   `git push --quiet -u origin fix/1`. Denied (was `ask`): `git push`,
   `git push -q`, `git push origin`, `git push origin feature/x && rm -rf target`,
   `git push origin feature/x\ngit push origin main`,
   `git commit -q -m x && git push -q 2>&1 | tail -2; sleep 10; gh run list`.
   Still asked: every case on the current ask list except the three moved, plus
   `git commit -m x && git push --force origin main`,
   `git push -q origin feature/x && git push -f origin main`,
   `bash -c "git push origin x"`, `git push origin x &`. Watch the new
   allowed and denied cases fail on the unedited hook.
3. The hook. Green.
4. Mutations: drop the `-q` alternative; make the per-segment check accept
   `--force`; drop the segment split so a joined plain push is asked; make the
   deny reason omit the form. Each must turn the test red.
5. Gates, lesson, review (rust-reviewer for craft, subprocess-safety-reviewer
   for gate-configuration).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The segment check admits a force or another remote to `deny` (which is fine) or the plain shape admits one to `allow` (which is not) | a destructive push runs unasked | the still-asked list, including force pushes joined after a plain one |
| Splitting on `;` or `|` inside quotes misreads a segment | a quoted push is denied instead of asked | a segment holding a quote or `$` fails the shape and is asked; nothing reaches `allow` through the split |
| `deny` loops: the session cannot find a form that passes | the same push refused twice | the reason names the exact form, and that form is the existing allowed one |
| A push the guard never sees (`git -C dir push`) | not asked, as before | unchanged by this change; out of scope |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked a_plain_push_to_origin_is_not_asked_about` — fails on the
  unedited hook at the first new case, passes after.
- The whole suite in CI on the push.

## Departures from the plan
