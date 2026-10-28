# Dedicated memory review of selected completed changes

Independent read-only evaluator: artifact_evaluator. Result: **approve; NO_CHANGE**.
Input snapshot: `815e327a3ebfc52a17edcffca3ce4d74eb8e302a`.
Selected population: changes 227, 228, 229 and 230 only. The reviewer independently
verified all 14 present records, seven policy fingerprints and 87,794 input bytes
against committed objects. JSON was 31,905 bytes. Change 229's optional plan/review
records are absent; this is a visibility limit, not an additional failure.

Prevalence counts affected selected changes out of four. Repeated mentions and
repair records do not create independent failure events. No project-wide rate,
fresh test result, accuracy gain or cost saving is inferred.

| Pattern | Evidence in selected batch | Existing closure | Decision and reopening evidence |
|---|---|---|---|
| Relocated native cache retained old paths | 2/4: actual failures in 227 and 229; 230 is a repair, not a third event | Lesson 102 and seeder guard: two variants, preserved objects/source and parent containment | NO_CHANGE; reopen if a newly seeded tree fails after 230, including another configuration layout |
| Optional GNU utility removed eval deadline | 1/4: 228's observed unbounded regression; 227 refers to the same discovery | Lesson 101, mandatory portable process-group deadline, validated limits, timeout remains infra after early success | NO_CHANGE; reopen if a supported invocation escapes the configured deadline or accepts an invalid limit |
| Local Git reader could lazily fetch | 1/4: independent review of 227 | No-lazy-fetch setting, actual promisor-transport and streaming/deadline guards | NO_CHANGE; reopen if a supported partial clone initiates transport |
| Lesson lacked watched-failing evidence | 1/4: 228's final admission gate caught the missing record after implementation review | Existing admission check refused it; evidence corrected and check rerun | NO_CHANGE; reopen if an incomplete lesson passes that check |

## Exact source examples

All paths and hashes refer to the input snapshot above:

| Source | SHA-256 |
|---|---|
| docs/sdlc/changes/227-harness-guidance-gaps/plan.md | ddeee9ece2bdd48aa7012735ecf9fbb36fcacbbb72e31ce8d940efa204a42644 |
| docs/sdlc/changes/229-eval-experiments-forget-rejected-ideas/verification.md | eca563c75402bd9d42b6d8998303b57be758e02fad95f8bf03ee544c7fa14dd9 |
| docs/sdlc/changes/230-a-cloned-native-cache-points-at-the-old-tree/verification.md | c780a6ca47c2734f2ed1690fb42a3a72d25d2b73aa3a4ba469c5c0d9c6f376f3 |
| docs/sdlc/changes/228-eval-runs-without-a-deadline/plan.md | 8b192b82a226d9ec71f27b6a43418540ee7dcad20c480d7da9cc02e590ba2063 |

## Existing memory inspected before proposing

| Memory / implementation | SHA-256 at snapshot |
|---|---|
| docs/sdlc/lessons.md | 5d6960a50fc692b1c5ae29dab3694cf6cb330bd58057d0d5bc6b9074738f6ade |
| scripts/seed-target.sh | 6cce1036d7cf86446102b49c6662fe722d64dcb10cd5e2a2bdb9456980da279c |
| scripts/run-evals.sh | 19b5f5c88bbf3959b088e6913b9263f9f014c2f4e12d998df18f4466f1090f2c |
| .claude/skills/sdlc/SKILL.md | 6547047be67a21cf7389cfe571f60715fa955c49431a2df277d48b162f53d791 |

Fixture startup and path-spelling corrections in 228/230 are recorded openly;
existing root-cause guidance already requires reproducing the intended failure,
so they do not justify another global instruction. Deliberately detached processes
remain outside the documented process-group cancellation guarantee.

Historical next-step text was treated as evidence, not current authority.
No new memory edit is proposed. No writes, tests, model CLIs, network calls or
private-archive reads were performed by the evaluator.
