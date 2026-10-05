# Spec: the release gate checks its three conditions instead of asking about them

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `.claude/hooks/guard-bash.sh` returns `allow` for a bundle-swap command only
   when `scripts/check-release-preconditions.sh` exits `0` for that exact command
   string. On any other exit it returns `ask`, and the reason it prints is the
   script's own explanation of which condition failed.
2. `scripts/check-release-preconditions.sh` establishes four conditions, in this
   order, stopping at the first failure:
   1. **The command is the canonical swap.** The three whitespace-separated
      tokens following `replace-macos-bundle.sh` are exactly
      `<repo>/dist/Operon.app`, `/Applications/Operon.app`, and
      `<repo>/dist/Operon.previous.app`. Any other argument shape, or a token
      carrying a shell metacharacter, fails.
   2. **The three gates pass on the tree as it stands.** `cargo fmt --check`,
      `cargo test --locked`, and `cargo clippy --locked -- -D warnings`, run now,
      not recalled from earlier in the session.
   3. **`dist/Operon.app` is what the packaging script builds from this tree.**
      `scripts/package-macos.sh` is run into a scratch `OPERON_DIST_DIR`, and the
      result must be `diff -r`-identical to `dist/Operon.app`.
   4. **No Operon is running.** No process matches the installed or staged bundle
      executable, and none matches a `target/{debug,release}/operon` build.
3. The script fails closed. A missing tool, an unreadable path, a non-zero exit
   from any step, or a command it cannot parse is a failure with a stated reason,
   never a pass. There is no flag, environment variable, or argument that makes
   it return `0` without running all four checks.
4. The script writes nothing outside its own scratch directory. It does not touch
   `dist/`, `/Applications`, or `target/release/operon` beyond what
   `cargo build --release --locked` does, and it removes the scratch directory on
   exit including on signal.
5. `gated_commands` in `scripts/harness-metrics.sh` still reads `6`. The gate did
   not go away; it stopped asking a person to stand in for it.
6. `docs/sdlc/README.md` no longer says the production boundary is one automation
   cannot cross on its own, because after this change it can — by checking. Both
   places that state the policy (the stage table at line 27 and the environment
   tiers at line 212) say what is now true.
7. `src/tests.rs` gains a test that fails if `guard-bash.sh` can reach `allow` for
   a bundle swap without consulting the preconditions script, and one that fails
   if the script's four checks are not all reachable.

## Behaviour

There is no UI. The observable behaviour is what a session sees at the swap:

**All four hold.** The hook returns `allow`. The session installs the build with
no interruption. The reason text records what was established, so the transcript
still shows why it was permitted rather than merely that it was.

**Any one fails.** The hook returns `ask`, exactly as today, and the reason names
the failed condition and what to do:

| Condition | What the reason says |
|---|---|
| not the canonical swap | which arguments were seen, and the triple that was expected |
| a gate failed | which of the three, and its last lines of output |
| the bundle does not reproduce | the `diff -r` output, capped |
| something is running | the pid and the executable path |

**The check cannot run at all** — `cargo` missing, the repository root not
resolvable, `jq` absent. `ask`, with the error as the reason. This is the same
outcome as today, so a broken check is never worse than no check.

No user-facing Japanese string is added: nothing here is read inside the app.
The hook's reason text is read by an agent and by the operator in the terminal,
and `guard-bash.sh` already writes those in English.

## Design

**`scripts/check-release-preconditions.sh`** — new, and the whole of the logic.
It takes the Bash tool's command string on stdin or as `$1`, and exits `0` with a
one-line summary on success or non-zero with the reason on failure. It is a
script and not inline hook code for three reasons: `guard-bash.sh` is a decision
table and stays one; a person can run the script to see why the gate refused; and
`src/tests.rs` can drive it directly.

Condition 3 rests on a property this change measured rather than assumed:
`scripts/package-macos.sh` is byte-reproducible. Two independent runs from the
same `target/release/operon`, into two different directories, produced
`diff -r`-identical bundles, and both were identical to the installed
`/Applications/Operon.app`. Ad-hoc `codesign` embeds no timestamp and no path, so
re-packaging is a total check of both "came from the script" and "came from this
tree" — strictly stronger than verifying the signature, which only proves nothing
changed *after* signing.

**`.claude/hooks/guard-bash.sh`** — the `replace-macos-bundle.sh` branch becomes:

```sh
if has 'replace-macos-bundle.sh'; then
  if verdict=$(bash "$root/scripts/check-release-preconditions.sh" "$command" 2>&1); then
    decide allow "Release gate — $verdict"
  fi
  decide ask "Release gate — $verdict"
fi
```

The `decide ask` line keeps its two-space indent so
`grep -cE '^  decide (ask|deny) '` still counts this boundary. The `decide allow`
is deeper, and is not a boundary.

The other three `ask` decisions and both `deny` decisions are untouched.

**`src/tests.rs`** — two guards, in the style of the existing harness tests: one
reads `guard-bash.sh` and fails unless the bundle-swap branch names the
preconditions script, one reads the script and fails unless all four conditions
are present and it has no bypass variable.

Timings measured on this machine, warm: `fmt` 1.7 s, `test` 18.4 s, `clippy`
1.4 s, packaging 3.1 s — about 25 s in total, against a 600 s hook timeout. The
existing `gate-commit.sh` already carries a 600 s timeout for the same reason.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | nothing is drawn; the change is a hook, a script, a document, and two tests |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | no glyph is added; nothing in this change reaches the UI |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | the canonical swap triple would otherwise be written in `package-macos.sh` and again in the checker. It is not restated: the checker derives `dist/Operon.app` and `dist/Operon.previous.app` from `OPERON_DIST_DIR`'s default the same way the packaging script does, and proves the bundle by re-running that script rather than by describing its output |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | no persisted shape changes. The invariant this touches is the bundle swap's, and it is strengthened, not weakened: the swap is now entered only from a tree whose build reproduces the staged bundle. `replace-macos-bundle.sh` and `package-macos.sh` are not edited, so the atomic-swap and rollback guarantees are untouched |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | the new children are spawned by a bash script from a hook, not by the Rust app, so `src/exec.rs`'s budgets do not apply and `subprocess_sites` does not move. The script bounds its own children instead: every command is a fixed literal with no interpolation of the untrusted command string, the command string is only ever *compared* against a derived triple and never evaluated, and output is captured and capped before it reaches the reason text |
| Documentation — user-facing docs change in all three languages together | No | `README.md` is not touched. `docs/sdlc/README.md` is an internal English document with no translations |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | no new dependency, no network, no `unsafe`. `cargo test --locked` and `cargo build --release --locked` are already what the repository runs |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | the reason text is the only output path. Each captured command's output is truncated to its last 2 000 bytes and the `diff -r` listing to its first 20 lines, so a failing gate cannot produce an unbounded hook reason |

## Flagged concerns

- **The command string is checked, and the command is a whole shell line.**
  The Bash tool's command is not just the swap — the one that prompted this
  change also piped to `tail`, ran `ls`, and ran `codesign`. So an `allow` from
  this gate authorizes a line that contains the canonical swap, not a line that
  contains *only* it. **Answered:** this is the pre-existing property of every
  decision in `guard-bash.sh` — `has 'gh release'` has always matched a
  substring of a larger line — and narrowing it is a different change. What this
  change must not do is make it worse, so the checker requires the three tokens
  after the script name to be the exact canonical triple with no metacharacters;
  a line that reaches the swap with different arguments falls back to `ask`. The
  two `deny` rules still run first and still fire on the same line, so the
  destructive shapes remain unreachable regardless of this gate's verdict.
- **Re-packaging inside the gate writes to `target/release/`.**
  Condition 3 runs `package-macos.sh`, which runs `cargo build --release
  --locked`. A gate with a side effect is a gate that can change the thing it is
  judging. **Answered:** the side effect is the one the session already caused by
  packaging, and it is idempotent — an unchanged tree rebuilds nothing, which is
  exactly why the byte comparison is meaningful. The alternative, a receipt file
  written at packaging time and trusted at swap time, moves the trust to a file
  that can be stale or forged and cannot notice a tree that moved in between. A
  check that re-derives beats a receipt that asserts.
- **A failure now costs 25 seconds before the prompt appears.**
  Today the prompt is immediate. **Answered:** accepted, and it is the right
  trade. The prompt is reached only when a condition actually failed, in which
  case the session was about to install a build it should not have, and the 25
  seconds bought the finding. On the success path the 25 seconds replace a human
  round trip, which is longer.
- **`cargo test --locked` skips the six `#[ignore]`d tests.**
  The gate therefore establishes what CI establishes and no more. **Answered:**
  left as is, and stated rather than hidden. Running the ignored six needs a live
  authenticated `codex`, `claude`, or `agy`, so a gate that required them would
  fail on a machine where those are not logged in and would be switched off. The
  reason text on success names what was run, so a release that needed the
  end-to-end restore coverage is still the operator's call — the same call
  `.claude/skills/ship/SKILL.md` already asks for.

## Acceptance

- `cargo test --locked` passes, including
  `the_release_gate_consults_the_preconditions_script` and
  `the_release_preconditions_script_has_no_bypass`.
- `bash scripts/check-release-preconditions.sh "$(printf 'bash %s/scripts/replace-macos-bundle.sh %s/dist/Operon.app /Applications/Operon.app %s/dist/Operon.previous.app' "$PWD" "$PWD" "$PWD")"` exits `0` on a clean tree with a freshly packaged `dist/Operon.app` and nothing running, and prints one line naming the four conditions it established.
- The same call exits non-zero, naming the condition, in each of four watched
  mutations: a file in `dist/Operon.app` edited by hand; a source file changed so
  the rebuilt bundle differs; a deliberately failing test; and Operon running.
- `bash scripts/check-bands.sh` reports `gated_commands` unmoved at `6`.
- `bash scripts/harness-metrics.sh` shows `hooks`, `gated_commands`,
  `tests_in_ci`, and `tests_ignored` in their bands, with `tests_in_ci` up by two.

## Rejected alternatives

- **Delete the `ask` branch.** Removes the boundary rather than automating it, and
  `gated_commands` would fall to `5`. The operator asked for the gate to hold.
- **An environment variable that skips the gate.** A bypass that exists is a
  bypass that becomes the default; it would also be indistinguishable, in a
  transcript, from a gate that passed.
- **A receipt file written by `package-macos.sh` and trusted by the hook.** Moves
  the trust into a file that can be stale, and requires editing the bundle-swap
  surface. Re-deriving costs 3 seconds.
- **`codesign --verify --deep --strict` alone.** Proves nothing changed after
  signing — measured, and it does catch an edited `Info.plist`, an added
  resource, and a replaced executable — but says nothing about *which* tree the
  bundle was built from. Re-packaging proves both.
- **Adding the check to `gate-commit.sh` instead.** The commit boundary is not the
  release boundary; a tree can move between them, which is the gap this closes.
  There is a sharper version of the same gap one layer up, found while this change
  was being built and left for its own change: `gate-commit.sh` reads the **working
  tree** while a commit records the **index**, so its verdict is not a statement
  about what is being committed. It was not hypothetical here — the gate ran the
  cargo gates against another session's dirty files while this change's index held
  six documents, and the verdict described neither.
- **Doing the same for `gh release`, `git push`, and `git tag`.** Out of scope by
  the operator's answer in `intent.md`. Those three publish outside this machine,
  where "correct" is not a property of the working directory.
