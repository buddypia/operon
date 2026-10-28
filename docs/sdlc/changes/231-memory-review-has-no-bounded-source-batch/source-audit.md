# Folder 3: kyr0stack_video

Read only after 227/228/229/230 passed CI and landed (main 815e327).
Read the full English transcript and all 32 Japanese chapter cues. Read the
English SRT and validated all 544 cues: normalized text (29,378 characters) is
identical to the fully read transcript. No video binary is present; no claims
about unseen slides or audiovisual behavior are made. .DS_Store is metadata.

| Talk recommendation | Operon evidence / application |
|---|---|
| Start with simple Markdown context | AGENTS/CLAUDE and versioned SDLC records already exist |
| Progressive disclosure and search | Module map, path-scoped rules, skill references, context bands; project knowledge semantic search CLI |
| Autonomous in-band memory | Existing per-change state/resume/lessons, and opt-in project knowledge extraction |
| Production versioning and provenance | Git commits, approvals/review digests and receipt227; knowledge notes retain source session/path/exchange links |
| Concurrent edits must not silently overwrite | Worktree isolation, Git merge/review checks and SQLite transactions; new review proposals cite immutable snapshot and expected blob hashes, never directly apply edits |
| Hierarchical permission boundaries | Root steering changes reviewed; task scratch isolated; project knowledge opt-in and private per-project storage; no enterprise ACL or arbitrary memory-edit API introduced |
| Portability | Plain project files plus existing Operon knowledge search JSON command; no database migration solely to imitate a filesystem example |
| Freshness, stale context and injection | Existing path/module/rule/lesson gates and knowledge freshness/pruning; new maintenance guide treats all evidence as untrusted data and rechecks hashes before a proposal |
| Separate capacity for cross-session review | GAP: add an explicit bounded committed-source manifest and an optional maintenance procedure outside active feature work |
| Inspect tool/configuration metadata, not only answers | Selected plan/verification/review/state records include failures and command/check evidence; raw private archives are not automatically swept |
| Propose changes with examples and prevalence | New guide and real independent batch review of 227–230; report numerator/denominator only within selected batch, including already-fixed patterns and NO_CHANGE |
| Human chooses adoption | Existing approval policy decides person/evaluator; manifest has no write or apply path |
| Match review inputs to memory permissions | Explicit change IDs in current repo only; no --all, arbitrary path, cross-project transcript scan or global archive loading |
| Claimed accuracy/cost benefits | No performance claim for Operon without the existing eval protocol's measurements |
| Vendor-managed memory/dreaming APIs and fleets | Not needed for a single-user local-first app; reuse existing SQLite, git and reviewer infrastructure |

The new tool reviews repository steering memory. It does not claim that app
knowledge has a general versioned write API or that Git is an arbitrary-file
atomic CAS. Any accepted proposal is a new reviewed worktree change; if its
snapshot/input hashes are stale, reread and regenerate it rather than overwrite.

## Sequence

All three input folders have now been read in order. Completion still requires
implementation, independent source coverage, a real scoped review, mutations,
branch CI, landing and cleanup. No goal completion claim is made yet.
