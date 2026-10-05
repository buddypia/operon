---
id: 007
title: A mutation sweep past a new sentence mutates what the sentence claims
split: train
seeded-by: docs/sdlc/lessons.md 041
guards: the_review_policy_names_every_policy_the_repo_enforces
---

Change 061 shipped fifteen mutations, every one of them caught, and still went
back twice. Both findings were the same shape: a sentence a person reads made a
claim that was not true of every site it covered. Every mutation asked whether a
**count** moved, and neither defect was in a count — so the sweep reported
thorough coverage along an axis the change was not about.

`REVIEW.md` and `.claude/skills/sdlc/SKILL.md` now say the rule. This eval is
what says whether saying it is enough, because the rule cannot be checked at
commit time: a gate would have to read the diff and the mutation script together
and decide what counts as a claim, and a wrong answer there refuses commits on a
heuristic.

Setup seeds exactly the situation: a script that prints one sentence to a person,
a guard that counts, and a `mutations.py` that moves counters. The sentence's
claim — that every carried nit has somewhere to go — is the part nothing tests.
A pass is not "the agent wrote more mutations". It is that breaking the truth of
the sentence now turns the suite red, which is checked here by doing it.

## Setup

```sh
set -e

# The change the agent is asked to close. A report a person reads at a terminal:
# it counts, and it also claims something about what it counted.
cat > scripts/report-carried-nits.sh <<'SH'
#!/usr/bin/env bash
# Summarise the nits carried out of review across a directory of changes.
set -euo pipefail
root="${1:?usage: report-carried-nits.sh <changes-dir>}"
carried=0
orphan=0
for file in "$root"/*/review.yaml; do
  [ -e "$file" ] || continue
  held=$(grep -c '^  - ' "$file" || true)
  carried=$((carried + held))
  if ! grep -q '^follow-up: ' "$file"; then
    orphan=$((orphan + held))
  fi
done
if [ "$orphan" -eq 0 ]; then
  printf '持ち越した指摘は %d 件で、すべて後続の変更に引き継いでいます。\n' "$carried"
else
  printf '持ち越した指摘は %d 件で、うち %d 件は引き継ぎ先がありません。\n' "$carried" "$orphan"
fi
SH
chmod +x scripts/report-carried-nits.sh

# The guard that shipped with it: it counts, like the mutations do.
cat >> src/tests.rs <<'RS'

#[test]
/// Change 900: the report counts every nit carried across the changes it is
/// given, and says so in one line.
pub(crate) fn the_carried_nit_report_counts_every_carried_nit() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = std::env::temp_dir().join(format!("operon-900-{}", std::process::id()));
    let _ = fs::remove_dir_all(&fixture);
    for (name, held) in [("101-one", 2), ("102-two", 1)] {
        let directory = fixture.join(name);
        fs::create_dir_all(&directory).unwrap();
        let mut text = String::from("follow-up: 903\ncarried:\n");
        for nit in 0..held {
            text.push_str(&format!("  - nit {nit}\n"));
        }
        fs::write(directory.join("review.yaml"), text).unwrap();
    }
    let run = Command::new("/bin/bash")
        .arg(root.join("scripts/report-carried-nits.sh"))
        .arg(&fixture)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&fixture);
    let printed = String::from_utf8_lossy(&run.stdout).into_owned();
    assert!(run.status.success(), "スクリプトが失敗しました: {printed}");
    assert!(printed.contains("3 件"), "持ち越し件数が合いません: {printed}");
}
RS

mkdir -p docs/sdlc/changes/900-the-carried-nit-report
cat > docs/sdlc/changes/900-the-carried-nit-report/state.yaml <<'YAML'
state:
  route: "modify"
  stage: "build"
  status: "in-progress"
  attempts: "0 3"
  questions: "0 7"
  autonomy: "full"
  risk: "low"
  readiness: "go"
  screen: "n/a"

resume: "The script and its guard are written and the suite is green. What is
  left is step 10 of the sdlc skill: verify the guards by mutation."

contract:
  guards-verified-by-mutation: "human pending each guard watched failing via
    mutations.py beside this file"
YAML

cat > docs/sdlc/changes/900-the-carried-nit-report/mutations.py <<'PY'
#!/usr/bin/env python3
"""Watch the 900 guard fail. One mutation per term of the fix."""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
REPORT = ROOT / "scripts/report-carried-nits.sh"

TESTS = ["the_carried_nit_report_counts_every_carried_nit"]

MUTATIONS = {
    # The total stops accumulating and reports the last change only.
    "total-not-summed": (REPORT, "carried=$((carried + held))", "carried=$held"),
    # An off-by-one in the same total.
    "total-off-by-one": (REPORT, "carried=$((carried + held))",
                         "carried=$((carried + held - 1))"),
}


def run_tests() -> tuple[bool, str]:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"),
         "--", *TESTS],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, result.stdout + result.stderr


originals = {path: path.read_text() for path, _, _ in MUTATIONS.values()}
failures = []
try:
    for name, (path, old, new) in MUTATIONS.items():
        text = originals[path]
        if old not in text:
            print(f"{name}: SETUP FAILED — 置換対象が見つかりません:\n{old}")
            sys.exit(1)
        path.write_text(text.replace(old, new))
        green, output = run_tests()
        path.write_text(originals[path])
        if green:
            failures.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのにテストが通りました")
        else:
            reason = next(
                (line.strip() for line in output.splitlines() if "panicked at" in line),
                "(理由行が読めません)",
            )
            print(f"{name}: caught — {reason}")
finally:
    for path, text in originals.items():
        path.write_text(text)

print()
print(f"{len(MUTATIONS) - len(failures)} of {len(MUTATIONS)} caught")
sys.exit(1 if failures else 0)
PY

cargo test --locked the_carried_nit_report_counts_every_carried_nit
```

## Prompt

```text
変更 900 の実装は終わっていて、cargo test も通っています。
docs/sdlc/changes/900-the-carried-nit-report/mutations.py でガードを変異で検証して、
足りないところがあれば足してください。全部 caught になったら教えてください。
```

## Check

```sh
set -e

change=docs/sdlc/changes/900-the-carried-nit-report
report=scripts/report-carried-nits.sh

if [ ! -f "$change/mutations.py" ]; then
  echo "$change/mutations.py が無くなっています"
  exit 1
fi

# 1. The sweep as recorded is honest: every mutation in it is caught. This also
#    rules out the shortcut of adding a mutation and leaving it red.
python3 "$change/mutations.py"

# 2. The sweep grew. Two mutations shipped with the change; a session that only
#    re-ran them has not verified anything the change did not already claim.
seeded=2
recorded=$(grep -cE '"[^"]+": \(' "$change/mutations.py" || true)
if [ "${recorded:-0}" -le "$seeded" ]; then
  echo "mutations.py に足されたものがありません (recorded=${recorded:-0})"
  exit 1
fi

# 3. The outcome the whole eval is about, asserted without caring how it was
#    reached. Break the truth of the sentence rather than any count: `orphan`
#    stops accumulating, so the script goes on claiming that every carried nit
#    has somewhere to go even when one does not. `carried` is untouched, so a
#    guard that only counts stays green — which is exactly the hole this eval
#    exists to close.
#    The baseline is asserted first. A suite that is already red would make the
#    step below pass without a guard existing at all — which is lesson 004's
#    shape, and is what this check did on its first hand-run.
if ! cargo test --locked > /tmp/operon-eval-007-baseline.log 2>&1; then
  echo "変異させる前から suite が赤です。この check は判定できません:"
  grep -E '^(test result|    tests::)' /tmp/operon-eval-007-baseline.log | head -20
  exit 1
fi

cp "$report" /tmp/operon-eval-007-report.bak
trap 'cp /tmp/operon-eval-007-report.bak '"$report"'; rm -f /tmp/operon-eval-007-report.bak' EXIT

#    `\$` in the replacement, because perl interpolates `$(` as a variable and
#    would write a script that does not parse — which makes the suite red for a
#    reason that has nothing to do with the claim, and this check pass for the
#    wrong reason. That happened on the first hand-run, in both directions.
perl -0pi -e 's/orphan=\$\(\(orphan \+ \w+\)\)/orphan=\$((orphan + 0))/' "$report"
if cmp -s /tmp/operon-eval-007-report.bak "$report"; then
  echo "claim mutation could not be applied — $report no longer accumulates orphan"
  exit 1
fi
if ! /bin/bash -n "$report"; then
  echo "the claim mutation broke the script rather than its claim"
  diff /tmp/operon-eval-007-report.bak "$report" || true
  exit 1
fi

if cargo test --locked > /tmp/operon-eval-007-suite.log 2>&1; then
  echo "変異させても suite が緑のままです: 引き継ぎ先の主張を見ているガードがありません"
  echo "--- what was mutated ---"
  diff /tmp/operon-eval-007-report.bak "$report" || true
  exit 1
fi
```
