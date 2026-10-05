#!/usr/bin/env node

/**
 * worktree-shipping-guard.mjs - Stop Hook
 *
 * Advisory Stop notice (systemMessage) when worktrees owned by this session still hold
 * uncommitted changes or unmerged commits. Never blocks the turn (harness-budget#stop_block_allowlist).
 * Suggests committing uncommitted changes first, and `/create-pr ship-worktree` only upon explicit approval.
 *
 * Policy SSOT: internal-rule (worktree-auto-ship.md) + internal-rule (worktree-session-ownership.md)
 *
 * Behavior:
 *   - user abort / context limit → passthrough (respects user intent)
 *   - stop_hook_active=true → passthrough (prevents stop-hook loop)
 *   - background_tasks not empty → passthrough (session incomplete)
 *   - .tmp/create-pr-active fresh (30min) → passthrough (/create-pr in progress)
 *   - All worktrees match one of the following → passthrough
 *       · main branch (first entry in worktree list)
 *       · hotfix/* / hotfix-* branch (escape hatch)
 *       · not owned by this session (other session / orphan — internal-rule filter)
 *       · attempt marker fresh (5min) — clean worktree with ship already attempted once
 *       · origin/<ship base>..HEAD is empty and no uncommitted changes (base: .cli/lib/ship-base-branch.mjs)
 *   - Owned + uncommitted changes in 1+ worktrees → NOTICE, deduped by content-hash marker
 *       (Defect 4) — unlike the ship-attempt marker below, this one is content-gated, not
 *       presence-gated: an unchanged uncommitted state is suppressed within the TTL window,
 *       but any actual change (file count, plan state) re-surfaces immediately even inside it.
 *   - Owned + clean + 0 ahead + tip landed on base by a merge (isLandedOnBase) → attempt marker +
 *       NOTICE naming `cleanup-worktree` (a merged worktree that was never removed)
 *   - Owned + unmerged commits in 1+ worktrees → create attempt marker + NOTICE
 *       · pipeline run active → deferred to a stderr notice instead (see isPipelineRunActive)
 *   - error → passthrough (internal-rule fail-open)
 */

import { existsSync, statSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import {
  output,
  safeHookMainWithProfile,
  readStdin,
  isUserAbort,
  isContextLimitStop,
  resolveProjectDir,
  safeGit,
  shellQuote,
} from '../lib/utils.mjs';
import { HookOutput } from '../lib/hook-output.mjs';
import {
  resolveWorktreePlanPath,
  parseWorktreeList,
} from '../lib/worktree-plan-path.mjs';
import { readOwnerLease } from '../lib/worktree-owner-lease.mjs';
import {
  readWorktreePlanChecklistStatus,
  formatPlanChecklistStatus,
} from '../lib/worktree-plan-status.mjs';
import {
  buildFileTree,
  loadReviewPanelConfig,
  renderReviewPanel,
  parseNameStatusRows,
} from '../lib/worktree-ship-report.mjs';
// Worktree root evaluation SSOT .
import { resolveWorktreeRoot } from '../lib/worktree-path.mjs';
import { formatReviewRemedy } from '../lib/quality-gate-labels.mjs';
// Trunk branch evaluation SSOT
import { enforcesWorktreeOnAllBranches, isTrunkBranch, loadWorktreePolicy } from '../lib/trunk-branch.mjs';
// Ship base SSOT — every base-ref list in this file derives from it (see shipBaseRefsFor).
import { resolveShipBaseBranch, resolveShipBaseRefs } from '../lib/ship-base-branch.mjs';
// Lease lifetime and path come from the SSOT — this guard already honoured the TTL, but held its
// own copy of the number, so a future change here would silently diverge from the other consumers.
import { CREATE_PR_ACTIVE_TTL_MS, createPrLeasePath } from '../lib/create-pr-lease.mjs';
// Pre-ship step state SSOT . Read-only — the guard never advances the sequence.
// Perspective 1 only: the deployed guard is the separate lightweight template under
// project-scaffolder/templates/hooks/, so this import never reaches a scaffold .
import {
  PRE_SHIP_STEPS,
  answerStorePath,
  readAnswerStore,
} from '../../.claude/scripts/lib/pre-ship-steps.mjs';
// Active-run query SSOT . active.json is worktree_local, so the read is
// pinned to the Stop session's projectDir rather than the module-load-frozen PROJECT_DIR.
import { getActiveRunId, withProjectDirOverride } from '../lib/layout-resolver.mjs';
export { parseWorktreeList };

const ATTEMPT_MARKER_TTL_MS = 5 * 60 * 1000; // 5 min — report to user and pause after 1 attempt
// Defect 4 (contract-reconciliation-harness, 2026-09-23): separate marker family from
// ATTEMPT_MARKER_TTL_MS above. That one is deliberately never touched for commit_required
// candidates (internal-rule — uncommitted work must stay visible). This one exists precisely
// for that class: it gates on *content* (has anything about this uncommitted state actually
// changed), not on presence, so the ~1.1KB panel stops re-rendering verbatim every Stop turn
// while still re-firing the moment the file count or plan state moves.
const UNCOMMITTED_NOTICE_TTL_MS = 5 * 60 * 1000;
const ESCAPE_HATCH_PATTERNS = [/^hotfix\//, /^hotfix-/];

export const STALENESS_BEHIND_THRESHOLD = 20;
export const STALENESS_AGE_DAYS_THRESHOLD = 7;

/**
 * File mtime freshness check. Returns false on absence/stat error.
 */
export function isFresh(absPath, ttlMs) {
  if (!existsSync(absPath)) return false;
  try {
    const ageMs = Date.now() - statSync(absPath).mtime.getTime();
    return ageMs <= ttlMs;
  } catch {
    return false;
  }
}

export function isEscapeHatchBranch(branch) {
  if (!branch) return false;
  return ESCAPE_HATCH_PATTERNS.some((p) => p.test(branch));
}

/**
 * Ship-base refs for a worktree, adapted to this hook's string-argument git runner.
 *
 * Every base-ref list in this file used to be a local `['origin/main', 'main']`. On a repository that
 * ships onto another branch that made all of them measure the branch's own work *plus* every
 * unreleased base commit — `countUnmergedCommits` over-reported, and `collectReviewSnapshot` fed the
 * same inflation into the review panel a human approves against. Resolution now lives in
 * `.cli/lib/ship-base-branch.mjs`.
 *
 * The refs reach `safeGit`'s /bin/sh string, so every interpolation below also goes through
 * `shellQuote` — the resolver rejects non-ref-shaped config values, and quoting is the second layer.
 *
 * @param {string} worktreePath
 * @param {Function} [_safeGit] Injected in tests.
 * @returns {string[]}
 */
export function shipBaseRefsFor(worktreePath, _safeGit = safeGit) {
  return resolveShipBaseRefs(worktreePath, {
    // safeGit reports failure as null; resolveShipBaseRefs expects a throw, and treats it as
    // "project root unknown" → documented fallback refs.
    gitFn: (cwd, args) => {
      // args are the resolver's own constants (no config value), so no quoting is needed here.
      const out = _safeGit(args.join(' '), cwd);
      if (out === null || out === undefined) throw new Error(`git ${args.join(' ')} failed`);
      return out;
    },
  });
}

/** Copy-pasteable shell word: plain when it needs no quoting, shellQuote'd otherwise. */
function shellWord(value) {
  return /^[\w./-]+$/.test(value) ? value : shellQuote(value);
}

/**
 * Counts unmerged commits in worktree.
 */
export function countUnmergedCommits(worktreePath) {
  for (const base of shipBaseRefsFor(worktreePath)) {
    const out = safeGit(`rev-list --count ${shellQuote(`${base}..HEAD`)}`, worktreePath);
    if (out !== null && /^\d+$/.test(out.trim())) return parseInt(out.trim(), 10);
  }
  return 0;
}

/**
 * Whether the worktree's tip was landed on base by a merge commit: the tip is a non-first parent of
 * a merge on base's *first-parent* chain in `<tip>..<base>`. First-parent only: a tip that became a
 * non-first parent of a merge inside some other branch (a PR branch that merged base in via
 * "Update branch", then landed) was never itself landed — that worktree is fresh, not merged. "0 commits ahead" alone also describes a worktree created a minute ago
 * and not yet committed to, and a session that stops there to ask a question must not be told to
 * delete it.
 *
 * Cost: the walk is bounded by `<tip>..<base>` (only commits newer than the tip), never the whole
 * history. Not detected, by design: a fast-forward landing (indistinguishable from a fresh
 * worktree) and a squash landing (the tip never becomes reachable from base; ship-worktree removes
 * those worktrees itself).
 *
 * @param {string} worktreePath
 * @param {{_safeGit?: Function}} [opts]
 */
export function isLandedOnBase(worktreePath, opts = {}) {
  const _safeGit = opts._safeGit || safeGit;
  const tip = (_safeGit('rev-parse --verify HEAD', worktreePath) || '').trim();
  if (!/^[0-9a-f]{7,64}$/.test(tip)) return false;
  for (const base of shipBaseRefsFor(worktreePath, _safeGit)) {
    const out = _safeGit(`rev-list --first-parent --merges --parents ${shellQuote(`${tip}..${base}`)}`, worktreePath);
    if (out === null || out === undefined) continue; // ref does not resolve here → next candidate
    return out.split('\n').some((line) => line.trim().split(/\s+/).slice(2).includes(tip));
  }
  return false;
}

/**
 * Counts uncommitted changes in worktree.
 */
export function countUncommittedChanges(worktreePath) {
  const out = safeGit('status --porcelain --untracked-files=all', worktreePath);
  if (out === null) return 0;
  if (!out.trim()) return 0;
  return out.split('\n').filter((line) => line.trim().length > 0).length;
}

/**
 * Fingerprint of *which* paths are uncommitted and how (porcelain status + path, sorted). The
 * dedup hash needs this, not the count: committing one file while touching another keeps the count
 * but is a different state, and a count-only hash would silently hide it for the whole TTL.
 */
export function uncommittedFingerprint(worktreePath) {
  const out = safeGit('status --porcelain --untracked-files=all', worktreePath);
  if (out === null) return null;
  const lines = out.split('\n').filter((line) => line.trim().length > 0).sort();
  return createHash('sha256').update(lines.join('\n')).digest('hex').slice(0, 16);
}

/**
 * Measures worktree base freshness (Layer 2 — measurement only, no additional blocking).
 */
export function measureStaleness(worktreePath, opts = {}) {
  const _safeGit = opts._safeGit || safeGit;
  const _now = opts._now || Date.now;
  for (const base of shipBaseRefsFor(worktreePath, _safeGit)) {
    const behindOut = _safeGit(`rev-list --count ${shellQuote(`HEAD..${base}`)}`, worktreePath);
    if (behindOut === null || !/^\d+$/.test(behindOut.trim())) continue;
    const mergeBaseOut = _safeGit(`merge-base HEAD ${shellQuote(base)}`, worktreePath);
    if (mergeBaseOut === null || !mergeBaseOut.trim()) continue;
    const tsOut = _safeGit(`log -1 --format=%ct ${shellQuote(mergeBaseOut.trim())}`, worktreePath);
    if (tsOut === null || !/^\d+$/.test(tsOut.trim())) continue;
    const tsSec = parseInt(tsOut.trim(), 10);
    const ageDays = Math.floor((_now() / 1000 - tsSec) / 86400);
    return {
      base,
      behind: parseInt(behindOut.trim(), 10),
      merge_base_age_days: ageDays,
    };
  }
  return null;
}

/**
 * Evaluates staleness metrics against threshold.
 */
export function evaluateStaleness(staleness) {
  if (!staleness) return [];
  const reasons = [];
  if (staleness.behind > STALENESS_BEHIND_THRESHOLD) {
    reasons.push(`behind ${staleness.behind} commits (>${STALENESS_BEHIND_THRESHOLD})`);
  }
  if (staleness.merge_base_age_days > STALENESS_AGE_DAYS_THRESHOLD) {
    reasons.push(
      `base ${staleness.merge_base_age_days}d old (>${STALENESS_AGE_DAYS_THRESHOLD}d)`,
    );
  }
  return reasons;
}

/**
 * Attempt marker path.
 */
export function attemptMarkerPath(projectDir, branch) {
  const safe = (branch || 'unknown').replace(/[\/\\]/g, '__');
  return join(projectDir, '.tmp', `worktree-shipping-attempted-${safe}`);
}

export function touchAttemptMarker(projectDir, branch) {
  const path = attemptMarkerPath(projectDir, branch);
  try {
    mkdirSync(join(projectDir, '.tmp'), { recursive: true });
    writeFileSync(path, `${new Date().toISOString()}\n`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Determines whether to touch escape attempt marker for candidate.
 */
export function shouldTouchAttemptMarker(candidate) {
  if (!candidate) return false;
  return !candidate.commit_required;
}

/**
 * Uncommitted-notice dedup marker path (Defect 4). Distinct file family from
 * `attemptMarkerPath` — see the constant comment above for why the two cannot share one marker.
 */
export function uncommittedNoticeMarkerPath(projectDir, branch) {
  const safe = (branch || 'unknown').replace(/[\/\\]/g, '__');
  return join(projectDir, '.tmp', `worktree-shipping-uncommitted-notice-${safe}`);
}

/**
 * Stable hash of exactly the candidate fields that change the rendered uncommitted-notice text.
 * Deliberately excludes anything timestamp-like (staleness, mtimes) — those must never cause a
 * spurious re-notification on their own.
 */
export function hashUncommittedState(candidate) {
  const payload = JSON.stringify({
    uncommitted: candidate.uncommitted,
    uncommitted_fingerprint: candidate.uncommitted_fingerprint,
    commits: candidate.commits,
    plan_missing: candidate.plan_missing,
  });
  return createHash('sha256').update(payload).digest('hex').slice(0, 16);
}

export function readUncommittedNoticeMarker(projectDir, branch) {
  try {
    const raw = readFileSync(uncommittedNoticeMarkerPath(projectDir, branch), 'utf-8');
    const [hash, isoTs] = raw.split('\n');
    if (!hash || !isoTs) return null;
    const ts = Date.parse(isoTs.trim());
    if (Number.isNaN(ts)) return null;
    return { hash: hash.trim(), ts };
  } catch {
    return null;
  }
}

export function touchUncommittedNoticeMarker(projectDir, branch, hash) {
  const path = uncommittedNoticeMarkerPath(projectDir, branch);
  try {
    mkdirSync(join(projectDir, '.tmp'), { recursive: true });
    writeFileSync(path, `${hash}\n${new Date().toISOString()}\n`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Whether the full uncommitted-changes notice should be suppressed for this candidate (Defect 4).
 * Both conditions must hold: the marker must be within TTL *and* match the current state hash.
 * A hash mismatch inside the TTL window means something actually changed (files staged/added,
 * PLAN.md authored) — that is new information the marker was never meant to hide, so it always
 * re-surfaces regardless of how recently the last notice fired.
 */
export function shouldSuppressUncommittedNotice(projectDir, candidate, opts = {}) {
  const _now = opts._now || Date.now;
  const _read = opts._readMarker || readUncommittedNoticeMarker;
  const marker = _read(projectDir, candidate.branch);
  if (!marker) return false;
  if (_now() - marker.ts > UNCOMMITTED_NOTICE_TTL_MS) return false;
  return marker.hash === hashUncommittedState(candidate);
}

/**
 * Checks whether PLAN.md exists in worktree.
 */
export function isPlanPresent(worktreePath, opts = {}) {
  const _existsSync = opts._existsSync || existsSync;
  return _existsSync(resolveWorktreePlanPath(worktreePath));
}

export function collectReviewSnapshot(worktreePath, opts = {}) {
  const _safeGit = opts._safeGit || safeGit;
  const baseCandidates = opts.baseCandidates || shipBaseRefsFor(worktreePath, _safeGit);
  let base = null;
  let git_log = '';

  for (const candidateBase of baseCandidates) {
    const out = _safeGit(`log --oneline ${shellQuote(`${candidateBase}..HEAD`)}`, worktreePath);
    if (out !== null) {
      base = candidateBase;
      git_log = out.trim();
      break;
    }
  }

  const commit_count_out = base
    ? _safeGit(`rev-list --count ${shellQuote(`${base}..HEAD`)}`, worktreePath)
    : null;
  const nameStatusText = base
    ? _safeGit(`diff --name-status ${shellQuote(`${base}...HEAD`)}`, worktreePath)
    : null;
  const changedFilesText = base
    ? _safeGit(`diff --name-only ${shellQuote(`${base}...HEAD`)}`, worktreePath)
    : null;

  const changed_files = (changedFilesText || '')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);
  const commit_count =
    commit_count_out && /^\d+$/.test(commit_count_out.trim())
      ? parseInt(commit_count_out.trim(), 10)
      : null;

  return {
    // When no candidate resolved there is nothing to compare against; report the ref we *intended*
    // to use rather than a literal, so the panel never names a branch this project does not ship to.
    base_branch: base || baseCandidates[0],
    git_log,
    commit_count,
    commit_range: base ? `${base}..HEAD` : null,
    latest_commit: git_log ? git_log.split('\n')[0] : null,
    changed_files,
    changed_files_tree: buildFileTree(changed_files),
    changed_rows: parseNameStatusRows(nameStatusText || ''),
  };
}

/**
 * Reads the owning session id of the worktree's `.session-owner` lease (identity only, no expiry).
 * The mailbox key is always derived from the worktree path — the same key the tracker and
 * `worktree-session-owner-guard` write with — so `branch` is ignored on purpose: for branches with
 * more than two segments (`feature/a/b`) the real branch and the path-derived key diverge.
 */
export function readSessionOwner(worktreePath, _branch = null) {
  return readOwnerLease(worktreePath, null).owner;
}

/**
 * Classifies whether this Stop session owns the worktree (internal-rule 2-Layer model).
 *
 * Layer 2 honours lease expiry: only a **live** lease counts. An expired lease (whoever wrote it) is
 * `orphan` — nobody is demonstrably working there, so the Stop gate must not demand that this
 * session ship it, nor label it as another live session's.
 *
 * @param {string} wtPath
 * @param {string|null} branch - unused for the lease lookup (path-derived key, see readSessionOwner)
 * @param {{sessionId?: string, cwd?: string, _resolveWorktreeRoot?: Function, _readSessionOwner?: Function, _readOwnerLease?: Function}} opts
 *   `_readSessionOwner` (legacy test seam) returns an id and is treated as a live lease.
 * @returns {'owned'|'other'|'orphan'}
 */
export function classifyOwnership(wtPath, branch, opts = {}) {
  const _resolveWorktreeRoot = opts._resolveWorktreeRoot || resolveWorktreeRoot;
  const _readOwnerLease =
    opts._readOwnerLease ||
    (opts._readSessionOwner
      ? (p) => ({ owner: opts._readSessionOwner(p, branch), fresh: true })
      : (p) => readOwnerLease(p, null));
  const sessionId = opts.sessionId;

  // Layer 1 — cwd-confinement (deterministic)
  const cwdWt = _resolveWorktreeRoot(opts.cwd || '');
  if (cwdWt && cwdWt === wtPath) return 'owned';

  // Layer 2 — live ownership lease
  const lease = _readOwnerLease(wtPath);
  if (lease?.owner && lease.fresh && sessionId) {
    return lease.owner === sessionId ? 'owned' : 'other';
  }
  return 'orphan';
}

/**
 * Whether a pipeline run is mid-flight in this project.
 *
 * Rule 1's premise — "commit before the final response" — assumes the turn being gated *is* a
 * final response. During a run it is not: the stage gates hand the turn back to the human as a
 * question (internal-rule-A explicitly forbids auto-advancing past a Guided Checkpoint), so
 * every one of those turns hits Stop. The guard then republished an unrelated ship Panel on top
 * of the checkpoint the human was supposed to answer — measured at ~4k tokens per fire, for
 * worktrees whose work predates the run and that the human had already been shown.
 *
 * Only the clean-tree class is deferred; see the `commit_required` split in evaluate(). The
 * *uncommitted* class survives this branch untouched, because that risk is real regardless of
 * what else the session is doing — work not yet in a commit can still be lost.
 *
 * Fails toward the existing behaviour: any error means "no run", i.e. the guard blocks as before.
 * A silent suppression is the one outcome worth spending a false block to avoid.
 *
 * @param {string} projectDir
 * @returns {boolean}
 */
export function isPipelineRunActive(projectDir) {
  try {
    return withProjectDirOverride(projectDir, () => getActiveRunId()) !== null;
  } catch {
    return false;
  }
}

/**
 * `git worktree list` entries that are never ship candidates: detached HEAD, trunk branches, escape-hatch
 * branches, the project checkout itself — and, with opt-in `enforce_worktree_all_branches`, the main
 * worktree on whatever branch it is (that checkout counts as trunk then).
 *
 * The main worktree is identified by position (git always lists it first), not by `wt.path === projectDir`:
 * that is a plain string compare, and git prints realpaths, so a projectDir reached through a symlink
 * (macOS `/var` vs `/private/var`, a symlinked checkout) never equals it. The positional check is limited
 * to the opt-in so key-off decisions stay exactly as before.
 */
function isNeverShipCandidate(wt, index, projectDir, trunkPolicy) {
  if (!wt.branch) return true; // detached HEAD
  if (isTrunkBranch(wt.branch, trunkPolicy)) return true;
  if (isEscapeHatchBranch(wt.branch)) return true;
  if (wt.path === projectDir) return true;
  return index === 0 && enforcesWorktreeOnAllBranches(trunkPolicy);
}

export function evaluate(projectDir, opts = {}) {
  const _now = opts._now || Date.now;
  const _isFresh = opts._isFresh || isFresh;
  const _safeGit = opts._safeGit || safeGit;
  const _countUnmerged = opts._countUnmerged || countUnmergedCommits;
  const _countUncommitted = opts._countUncommitted || countUncommittedChanges;
  const _uncommittedFingerprint = opts._uncommittedFingerprint || uncommittedFingerprint;
  const _isPlanPresent = opts._isPlanPresent || ((p) => isPlanPresent(p, opts));
  const _readPlanChecklistStatus =
    opts._readPlanChecklistStatus ||
    ((wtPath, branch) => {
      if (Object.prototype.hasOwnProperty.call(opts, '_isPlanPresent')) {
        const present = _isPlanPresent(wtPath);
        return {
          present,
          complete: present,
          unreadable: false,
          unchecked_count: present ? 0 : null,
          unchecked: [],
          plan_path: resolveWorktreePlanPath(wtPath, branch),
        };
      }
      return readWorktreePlanChecklistStatus(wtPath, branch);
    });
  const _collectReviewSnapshot =
    opts._collectReviewSnapshot ||
    ((wtPath, branch) => collectReviewSnapshot(wtPath, { _safeGit, branch }));
  const _measureStaleness =
    opts._measureStaleness || ((p) => measureStaleness(p, { _safeGit, _now }));
  const _classifyOwnership =
    opts._classifyOwnership ||
    ((wtPath, branch) => classifyOwnership(wtPath, branch, { sessionId: opts.sessionId, cwd: opts.cwd }));
  const _humanFlowStarted = opts._humanFlowStarted || humanFlowStarted;
  const _isPipelineRunActive = opts._isPipelineRunActive || isPipelineRunActive;
  const _isLandedOnBase = opts._isLandedOnBase || ((p) => isLandedOnBase(p, { _safeGit }));

  // Pass if /create-pr is active
  const activeFlag = createPrLeasePath(projectDir);
  if (_isFresh(activeFlag, CREATE_PR_ACTIVE_TTL_MS)) {
    return { block: false, candidates: [], skipped_attempt: [], not_owned: [], deferred_run_active: [], reason: 'create-pr-active fresh' };
  }

  const wtOut = _safeGit('worktree list --porcelain', projectDir);
  if (wtOut === null) {
    return { block: false, candidates: [], skipped_attempt: [], not_owned: [], deferred_run_active: [], reason: 'worktree list failed' };
  }

  const worktrees = parseWorktreeList(wtOut);
  const candidates = [];
  const skipped_attempt = [];
  const not_owned = [];
  const trunkPolicy = loadWorktreePolicy(projectDir);

  for (const [index, wt] of worktrees.entries()) {
    if (isNeverShipCandidate(wt, index, projectDir, trunkPolicy)) continue;

    const uncommitted = _countUncommitted(wt.path);
    const commits = _countUnmerged(wt.path);
    // Clean and nothing ahead used to mean "finished". A merged worktree that was never removed is
    // exactly that, and it is what a manual `--no-ff` landing leaves, so it is reported too.
    const landed = commits === 0 && uncommitted === 0 && _isLandedOnBase(wt.path);
    if (commits === 0 && uncommitted === 0 && !landed) continue;

    const ownership = _classifyOwnership(wt.path, wt.branch);
    if (ownership !== 'owned') {
      not_owned.push({ path: wt.path, branch: wt.branch, commits, uncommitted, ownership, landed });
      continue;
    }

    if (landed) {
      const entry = { path: wt.path, branch: wt.branch, commits, uncommitted, stale_reasons: [] };
      if (_isFresh(attemptMarkerPath(projectDir, wt.branch), ATTEMPT_MARKER_TTL_MS)) skipped_attempt.push(entry);
      else candidates.push({ ...entry, commit_required: false, landed: true, plan_missing: false });
      continue;
    }

    const staleness = _measureStaleness(wt.path);
    const stale_reasons = evaluateStaleness(staleness);

    if (uncommitted > 0) {
      const plan_status = commits > 0 ? _readPlanChecklistStatus(wt.path, wt.branch) : null;
      candidates.push({
        path: wt.path,
        branch: wt.branch,
        commits,
        uncommitted,
        uncommitted_fingerprint: _uncommittedFingerprint(wt.path),
        commit_required: true,
        plan_missing: commits > 0 ? !(plan_status?.present ?? _isPlanPresent(wt.path)) : false,
        plan_status,
        review_ready: false,
        staleness,
        stale_reasons,
      });
      continue;
    }

    const marker = attemptMarkerPath(projectDir, wt.branch);
    if (_isFresh(marker, ATTEMPT_MARKER_TTL_MS)) {
      skipped_attempt.push({
        path: wt.path,
        branch: wt.branch,
        commits,
        uncommitted,
        staleness,
        stale_reasons,
      });
      continue;
    }

    const plan_status = _readPlanChecklistStatus(wt.path, wt.branch);
    const review_ready = plan_status.present === true && plan_status.complete === true;

    candidates.push({
      path: wt.path,
      branch: wt.branch,
      commits,
      uncommitted,
      commit_required: false,
      plan_missing: !plan_status.present,
      plan_status,
      review_ready,
      review: review_ready ? _collectReviewSnapshot(wt.path, wt.branch) : null,
      human_flow_started: review_ready ? _humanFlowStarted(wt.path, wt.branch) : false,
      staleness,
      stale_reasons,
    });
  }

  if (candidates.length === 0) {
    return { block: false, candidates: [], skipped_attempt, not_owned, deferred_run_active: [], reason: 'no unmerged worktree' };
  }

  // Split on the axis that actually carries risk, not on "a worktree exists". Rationale in
  // isPipelineRunActive. Committed work is recoverable and the human has already seen it;
  // uncommitted work is neither, so it keeps blocking even mid-run.
  // One predicate drives both halves so the partition cannot drift; matching on the predicate
  // rather than on array membership also keeps this independent of object identity.
  const runActive = _isPipelineRunActive(projectDir);
  const isDeferred = (c) => runActive && !c.commit_required;
  const deferred_run_active = candidates.filter(isDeferred);
  const blocking = candidates.filter((c) => !isDeferred(c));

  if (blocking.length === 0) {
    return {
      block: false,
      candidates: [],
      skipped_attempt,
      not_owned,
      deferred_run_active,
      reason: 'pipeline run active — clean worktrees deferred to notice',
    };
  }

  return {
    block: true,
    candidates: blocking,
    skipped_attempt,
    not_owned,
    deferred_run_active,
    reason: `${blocking.length} worktree(s) need completion`,
  };
}

/**
 * Whether any Rule 13 human step (deck review / approval) already holds an answer.
 *
 * The full panel is only worth its length while nothing has been shown to the human yet.
 * Once an answer exists that premise is gone — the deck and the PROOF were built and the
 * human already read them. Re-dumping the panel plus the 4-step procedure then teaches a
 * sequence already 7 steps in: measured at ~4k tokens per fire for zero decision value
 * (observed 2026-08-23, fired twice while steps 7 and 8 awaited an answer).
 *
 * Staleness is deliberately ignored. An invalidated answer does not undo the fact that the
 * human saw the change; what they need at that point is *what moved*, not the panel again.
 * Freshness is the runner's call (`evaluateSteps`) and is not re-implemented here.
 */
export function humanFlowStarted(worktreeAbs, branch) {
  try {
    const store = readAnswerStore(answerStorePath(worktreeAbs, branch));
    return PRE_SHIP_STEPS.some((s) => s.kind === 'human' && store[s.id]);
  } catch {
    return false; // fail-open — when in doubt, emit the full panel 
  }
}

export function relativizePath(absPath, projectDir) {
  // With the separator: a sibling `<project>-worktrees/x` shares the prefix and is not inside it.
  return absPath.startsWith(`${projectDir}/`) ? absPath.slice(projectDir.length + 1) : absPath;
}

/** Section naming merged-but-present worktrees and the exact removal command for each. */
function buildLandedSection(projectDir, landed, shipBase) {
  if (landed.length === 0) return [];
  const lines = ['', `Already merged into ${shipBase} but still present — remove them now, from the main checkout:`];
  for (const c of landed) {
    const rel = relativizePath(c.path, projectDir);
    lines.push(`  - ${rel}  (branch=${c.branch})`);
    lines.push(`    node .claude/scripts/create-pr/ops.mjs cleanup-worktree --worktree "${rel}"`);
  }
  lines.push('  A merged worktree left behind is left for good: no other session may remove it .');
  return lines;
}

export function buildBlockMessage(projectDir, candidates) {
  // Fallback base named in the panel / rebase hint — the configured ship base, never a literal.
  const shipBase = resolveShipBaseBranch(projectDir);
  // Project panel config (sections / labels / approval choices / line budget); absent → built-in.
  const panel = loadReviewPanelConfig(projectDir);
  const isReviewReady = (c) =>
    !c.commit_required &&
    (c.review_ready === true || (c.review_ready === undefined && c.plan_missing === false));
  const landed = candidates.filter((c) => c.landed);
  const unlanded = candidates.filter((c) => !c.landed);
  const reviewReady = unlanded.filter((c) => isReviewReady(c));
  const planBlocked = unlanded.filter((c) => !c.commit_required && !isReviewReady(c));
  const lines = [
    '[worktree-shipping-guard] Uncompleted worktree work remains (advisory — the session is not blocked).',
    '',
    'Policy : worktree changes should be committed promptly — uncommitted work is not safe against loss.',
    'Committed work ships via /create-pr ship-worktree once the approval step passes (human for T2 one-way doors, automatic for reviewed T0/T1 — internal-rule) → squash merge → cleanup.',
    'False-positive prevention: Only worktrees owned by this session with uncommitted changes or unmerged commits are targeted. READ-ONLY sessions with no changes/commits pass through.',
    '',
    'Target worktree(s):',
  ];
  for (const c of unlanded) {
    const rel = relativizePath(c.path, projectDir);
    const planTag = c.plan_missing ? ' [PLAN.md missing — authoring required]' : '';
    const dirtyTag = c.uncommitted > 0 ? `, uncommitted=${c.uncommitted} file(s)` : '';
    const actionTag = c.commit_required ? ' [commit required]' : '';
    lines.push(`  - ${rel}  (branch=${c.branch}, unmerged=${c.commits} commit${dirtyTag})${actionTag}${planTag}`);
  }
  lines.push(...buildLandedSection(projectDir, landed, shipBase));
  if (candidates.some((c) => c.commit_required)) {
    lines.push('');
    lines.push('Uncommitted changes (not yet safe against loss):');
    for (const c of candidates.filter((item) => item.commit_required)) {
      lines.push(`  - ${relativizePath(c.path, projectDir)} (branch=${c.branch}): ${c.uncommitted} file(s) not committed`);
    }
    lines.push('  Commit only the files you own before shipping, e.g. inside that worktree:');
    lines.push('    git status --short && git add <owned-files-only> && git commit -m "<Conventional Commits>"');
  }
  if (candidates.some((c) => c.plan_missing)) {
    lines.push('');
    lines.push('Worktrees missing PLAN.md (M1 stop loop avoidance):');
    lines.push('  - ship-worktree performs PLAN.md unchecked checkbox validation — rejects if PLAN.md is missing.');
    lines.push('  - Location: `.tmp/worktree-<safeBranch>/PLAN.md` (not worktree root, internal-rule/internal-rule — prevents merge leakage).');
    lines.push('  - Author PLAN.md first (goals / checklist / verification / handoff) before calling ship.');
    lines.push('  - Alternatively, discard the worktree after confirming user intent.');
  }
  if (planBlocked.length > 0) {
    lines.push('');
    lines.push('Worktrees with incomplete/unverified PLAN.md checklists:');
    for (const c of planBlocked) {
      const status = c.plan_status ? formatPlanChecklistStatus(c.plan_status, panel.labels) : 'PLAN.md status unverified';
      lines.push(`  - ${relativizePath(c.path, projectDir)}: ${status}`);
      if (Array.isArray(c.plan_status?.unchecked) && c.plan_status.unchecked.length > 0) {
        for (const item of c.plan_status.unchecked.slice(0, 5)) lines.push(`    ${item}`);
      }
    }
    lines.push('  → Approval request reviews are only output for worktrees where all PLAN.md checklist items are complete.');
    lines.push('  → Complete or drop items in PLAN.md first, then call Stop again or run verify-plan.');
  }
  lines.push('');
  if (reviewReady.length > 0) {
    // "output the review below" only holds when a panel actually follows. Emitting it when
    // every candidate resolved to resume guidance would point at a panel that is not there.
    if (reviewReady.some((c) => !c.human_flow_started)) {
      lines.push('For worktrees with commits and completed PLAN.md checklists, output the review below to user; if `pre-ship-steps.mjs next` shows the approval step AWAITING, obtain explicit approval.');
      lines.push(`Run the following command only after the approval step is OK — the user's "${panel.approvalChoices[0]}" or an auto-approval (idempotent — reuses PR if already existing):`);
      lines.push('');
    }
    lines.push('  OPS="node .claude/scripts/create-pr/ops.mjs"');
    for (const c of reviewReady) {
      const rel = relativizePath(c.path, projectDir);
      const review = c.review || {};
      if (c.human_flow_started) {
        lines.push('');
        lines.push(...buildResumeGuidance(rel, c));
        continue;
      }
      lines.push('');
      const rendered = renderReviewPanel(panel, {
        worktreePath: rel,
        branch: c.branch,
        baseBranch: review.base_branch || shipBase,
        planPath: c.plan_status?.plan_path || resolveWorktreePlanPath(c.path, c.branch),
        planChecklist: c.plan_status ? formatPlanChecklistStatus(c.plan_status, panel.labels) : panel.labels.plan_complete,
        worktreeStatus: `clean, unmerged=${c.commits} commit`,
        gitLog: review.git_log,
        commitCount: review.commit_count ?? c.commits,
        commitRange: review.commit_range || `origin/${shipBase}..HEAD`,
        latestCommit: review.latest_commit,
        changedFilesTree: review.changed_files_tree || panel.labels.collection_required,
        changedRows: review.changed_rows || [],
      });
      lines.push(rendered.text, ...rendered.notices);
      lines.push('');
      lines.push('Procedure (approval order is enforced by step runner — internal-rule):');
      lines.push(`  1. node .claude/scripts/pre-ship-steps.mjs next --worktree "${rel}"`);
      lines.push('     → Only the current step is output. Clear automated check blockers first.');
      lines.push('  2. Upon reaching human judgement steps (deck review / approval), pass questions verbatim to user,');
      lines.push('     and record their exact response (repeat until all steps pass):');
      lines.push(
        `     node .claude/scripts/pre-ship-steps.mjs answer --worktree "${rel}" --step <id> --value "<what user actually said>"`,
      );
      lines.push(`  3. node .claude/scripts/mark-pre-ship-confirmed.mjs "${c.branch}" --quality <label>`);
      lines.push(
        `     label: agent_go / skill_review_pass — run ${formatReviewRemedy()} /`,
      );
      lines.push(
        '            self_review_pass (no secondary review tools available) / trivial_skip (≤2 files + ≤20 LOC + non-substantive).',
      );
      lines.push(
        '     → Select the one actually executed. The first two require execution records in PROOF gates[] for marker creation.',
      );
      lines.push('     → Rejected before completing all 8 steps (exit 1). Approval is recorded in step 2.');
      lines.push(`  4. $OPS ship-worktree --worktree "${rel}" --title "<Conventional Commits>" --body "<Summary>"`);
    }
  } else if (unlanded.length > 0) {
    lines.push('No worktrees currently eligible for outputting approval request reviews. Commit dirty changes or complete PLAN.md checklists.');
  }
  // Layer 2 — base staleness warning
  const stale = candidates.filter((c) => Array.isArray(c.stale_reasons) && c.stale_reasons.length);
  if (stale.length > 0) {
    lines.push('');
    lines.push('base freshness warning (Layer 2 — non-blocking, post-gate handled by ship-worktree non-FF check):');
    for (const c of stale) {
      lines.push(`  - ${relativizePath(c.path, projectDir)}: ${c.stale_reasons.join(', ')}`);
    }
    lines.push('  → Use standard entry point for next worktree: make wt.new BR=<branch> (or node .claude/scripts/worktree-new.mjs --branch <branch>)');
    lines.push(
      `  → Recommend rebasing current worktree before ship: git fetch origin ${shellWord(shipBase)} && git rebase ${shellWord(`origin/${shipBase}`)}`,
    );
  }
  lines.push('');
  lines.push('This notice is suppressed for 5 minutes after it is shown (attempt marker).');
  lines.push('hotfix/* branch worktrees are exempt.');
  return lines.join('\n');
}

/**
 * Short resume guidance for a worktree whose human steps are already under way.
 *
 * The question text is not duplicated here — the runner owns the current step and its
 * wording, and a copy drifts silently the moment a step definition changes (the same drift
 * class `pre-ship-guidance-coherence` exists to catch). The change itself already lives in
 * the generated deck and PLAN.md, so it is not dumped again either.
 */
export function buildResumeGuidance(rel, candidate) {
  return [
    `Pre-ship flow already in progress — ${rel} (${candidate.branch}, ${candidate.commits} commit).`,
    'Full panel omitted: it was already presented and the human has answered at least one step.',
    'Continue from the runner — it owns the current step and its verbatim question:',
    `  1. node .claude/scripts/pre-ship-steps.mjs next --worktree "${rel}"`,
    '     → Pass a human step question to the user verbatim. Never answer on their behalf.',
    `  2. node .claude/scripts/pre-ship-steps.mjs answer --worktree "${rel}" --step <id> --value "<what user actually said>"`,
    `  3. node .claude/scripts/mark-pre-ship-confirmed.mjs "${candidate.branch}" --quality <label>`,
    `  4. $OPS ship-worktree --worktree "${rel}" --title "<Conventional Commits>" --body "<Summary>"`,
    'If HEAD moved since those answers, the runner reports them invalidated — report what changed',
    'and record fresh answers ; do not reuse the earlier response.',
  ];
}

export function emitSkippedAttemptNotice(skipped, write = (m) => process.stderr.write(m)) {
  if (!Array.isArray(skipped) || skipped.length === 0) return;
  const lines = ['[worktree-shipping-guard] passthrough — 5-minute attempt marker fresh (notice suppressed):'];
  for (const c of skipped) {
    const staleTag =
      Array.isArray(c.stale_reasons) && c.stale_reasons.length
        ? ` [stale: ${c.stale_reasons.join(', ')}]`
        : '';
    lines.push(`  - branch=${c.branch}, unmerged=${c.commits} commit (path=${c.path})${staleTag}`);
  }
  lines.push('  → The notice reappears after marker expiration. To handle immediately, delete marker and re-enter.');
  write(`${lines.join('\n')}\n`);
}

/**
 * stderr, never `reason`. A Stop hook's `reason` is the next instruction Claude executes, so
 * putting a standing backlog there turns every stage gate into a ship errand. stderr keeps the
 * same facts visible to the human without competing with the checkpoint they are answering.
 */
export function emitRunActiveNotice(deferred, write = (m) => process.stderr.write(m)) {
  if (!Array.isArray(deferred) || deferred.length === 0) return;
  const lines = [
    '[worktree-shipping-guard] passthrough — pipeline run active, committed worktrees deferred :',
  ];
  for (const c of deferred) {
    lines.push(`  - branch=${c.branch}, unmerged=${c.commits} commit (path=${c.path})`);
  }
  lines.push('  → Blocks again once the run reaches idle. To ship now, say so and the pre-ship flow resumes.');
  write(`${lines.join('\n')}\n`);
}

export function emitNotOwnedNotice(notOwned, write = (m) => process.stderr.write(m)) {
  if (!Array.isArray(notOwned) || notOwned.length === 0) return;
  const lines = [
    '[worktree-shipping-guard] passthrough — Incomplete work in worktrees not owned by this session (not blocked, internal-rule):',
  ];
  for (const c of notOwned) {
    const dirty = c.uncommitted > 0 ? `, uncommitted=${c.uncommitted} file(s)` : '';
    const tag = c.ownership === 'other' ? 'owned by other session' : 'orphan (unknown owning session)';
    const state = c.landed ? 'merged, not removed' : `unmerged=${c.commits} commit${dirty}`;
    lines.push(`  - branch=${c.branch}, ${state} [${tag}] (path=${c.path})`);
  }
  lines.push('  → Ship from the session that created that worktree, or if it is your work, proceed inside that worktree.');
  write(`${lines.join('\n')}\n`);
}

/**
 * Defect 4: stderr companion for uncommitted candidates suppressed by
 * `shouldSuppressUncommittedNotice` — keeps the fact visible to a human reading logs without
 * repeating the full ~1.1KB panel for state they have already been shown unchanged.
 */
export function emitDedupedUncommittedNotice(deduped, write = (m) => process.stderr.write(m)) {
  if (!Array.isArray(deduped) || deduped.length === 0) return;
  const lines = [
    '[worktree-shipping-guard] passthrough — uncommitted-changes notice unchanged since last shown:',
  ];
  for (const c of deduped) {
    lines.push(`  - branch=${c.branch}, uncommitted=${c.uncommitted} file(s) (path=${c.path})`);
  }
  lines.push('  → Reappears once the file count/plan state changes, or after the cooldown window elapses.');
  write(`${lines.join('\n')}\n`);
}

export function shouldEarlyPassthrough(data) {
  if (isUserAbort(data) || isContextLimitStop(data)) return true;
  if (data?.stop_hook_active) return true;
  if (Array.isArray(data?.background_tasks) && data.background_tasks.length > 0) return true;
  return false;
}

/**
 * Splits candidates into re-notified (`visible`) and unchanged-uncommitted (`deduped`), touching the
 * dedup marker for each visible commit_required candidate. `opts` carries the test seams.
 */
function partitionUncommittedNotices(projectDir, candidates, opts) {
  const _shouldSuppress = opts._shouldSuppressUncommittedNotice || shouldSuppressUncommittedNotice;
  const _touchNoticeMarker = opts._touchUncommittedNoticeMarker || touchUncommittedNoticeMarker;
  const deduped = [];
  const visible = [];
  for (const c of candidates) {
    if (c.commit_required && _shouldSuppress(projectDir, c)) {
      deduped.push(c);
      continue;
    }
    visible.push(c);
    if (c.commit_required) _touchNoticeMarker(projectDir, c.branch, hashUncommittedState(c));
  }
  return { deduped, visible };
}

export async function run(data, opts = {}) {
  const _evaluate = opts._evaluate || evaluate;
  try {
    if (shouldEarlyPassthrough(data)) {
      return HookOutput.passthrough();
    }
    const projectDir = resolveProjectDir(data);
    const verdict = _evaluate(projectDir, { sessionId: data?.session_id, cwd: data?.cwd });

    emitNotOwnedNotice(verdict.not_owned);
    emitRunActiveNotice(verdict.deferred_run_active);

    if (!verdict.block) {
      emitSkippedAttemptNotice(verdict.skipped_attempt);
      return HookOutput.passthrough();
    }

    // Defect 4: commit_required candidates never touch `attemptMarkerPath` (Rule 2 keeps them
    // visible), so absent this step the same ~1.1KB panel re-rendered verbatim on every Stop
    // turn until the user committed. Split on content-hash dedup instead — a candidate whose
    // uncommitted state is unchanged since the last fresh marker drops to a one-line stderr
    // reminder; anything new (or expired) still gets the full panel.
    const { deduped, visible } = partitionUncommittedNotices(projectDir, verdict.candidates, opts);
    emitDedupedUncommittedNotice(deduped);

    for (const c of visible) {
      if (shouldTouchAttemptMarker(c)) touchAttemptMarker(projectDir, c.branch);
    }

    if (visible.length === 0) {
      return HookOutput.passthrough();
    }
    return HookOutput.notice(buildBlockMessage(projectDir, visible));
  } catch (e) {
    // Defect 1: a self-caught crash must not read as a clean evaluation — errorNotice() surfaces
    // it to the user and tags `hookError` so hook-orchestrator#executeHook still records it as an
    // error even though run() itself never threw.
    return HookOutput.errorNotice('worktree-shipping-guard', e);
  }
}

if (!globalThis.__HOOK_ORCHESTRATOR__) {
  safeHookMainWithProfile('worktree-shipping-guard', async () => {
    const data = await readStdin();
    return output(await run(data));
  });
}
