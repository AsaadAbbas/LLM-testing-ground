# Benchmark V4 Report: Full Pipeline 3-Way Eval on ChronoKV (Rust + TypeScript)

**Date**: 2026-03-23
**Model**: Claude Opus 4.6 (1M context)
**Task**: ChronoKV — Rust workspace (5 crates) + TypeScript SDK/CLI, 7 unlabeled bugs (3 compounding pairs), 4 cross-cutting features
**Difficulty**: FeatureBench-zone (7 unlabeled bugs, cross-language, compounding issues)
**Prompt**: Identical unstructured braindump for all 3 configs
**Research Basis**: 56 authoritative sources (FeatureBench, SWE-bench Pro, METR HCAST, ProjDevBench, Terminal-Bench, etc.)

---

## Executive Summary

All 3 configs completed the work. The **v4 pipeline produced the most tests (77) and the only structured task tracker with file:line citations**. **GSD-only was fastest and the only one to fully fix the replication term bug (Bug 5)**. **Dry CLAUDE.md produced the most documentation (ROADMAP, SESSION_LOG, phase folders) and found 2 extra bugs beyond the planted 7**. However, **NO config fixed the compaction tombstone bug (Bug 3)** — all 3 missed the `entry.timestamp` vs `entry.deleted_at` distinction. This validates the benchmark's difficulty design.

| Metric | v4 Pipeline | GSD-only | Dry CLAUDE.md v1 |
|--------|:-----------:|:--------:|:----------------:|
| **Duration** | 12.6 min | **9.2 min** | 10.1 min |
| **Tokens** | 137,926 | **98,748** | 109,519 |
| **Tool calls** | 108 | **71** | 94 |
| **Bugs found** | 7/7 | 7/7 | **9** (7+2 extra) |
| **Bugs correctly fixed** | 5/7 | **6/7** | 5/7 |
| **Features implemented** | 4/4 | 4/4 | 4/4 |
| **Rust tests** | **64** | 50 | 52 |
| **TypeScript tests** | 13 | 12 | **14** |
| **Total tests** | **77** | 62 | 66 |
| **State documents** | 1 (.tasks/TRACKER.md) | 0 | **5** (Docs/) |
| **Git commits** | 1 (monolithic) | 1 (monolithic) | 1 (monolithic) |
| **Atomic triggered?** | Partial (.tasks/ created) | N/A | N/A |
| **cargo clippy warnings** | 6 | 8 | 8 |
| **tsc --noEmit** | Pass | Pass | Pass |

---

## Scoring (40 assertions, 100 points)

### Category A: Bug Discovery (14 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| A-1 | WAL checksum omission | 2 | **Pass** | **Pass** | **Pass** |
| A-2 | Memtable version ordering | 2 | **Pass** | **Pass** | **Pass** |
| A-3 | Compaction tombstone drop | 2 | Fail | Fail | Fail |
| A-4 | Time-range boundary mismatch | 2 | **Pass** | **Pass** | **Pass** |
| A-5 | Replication catch-up wrong term | 2 | Partial | **Pass** | Partial |
| A-6 | Timestamp serialization mismatch | 2 | **Pass** | **Pass** | **Pass** |
| A-7 | WebSocket subscription leak | 2 | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **14** | **11** | **12** | **11** |

**Notes:**
- A-3: ALL configs missed the `entry.timestamp` vs `entry.deleted_at` distinction. The compaction still uses creation timestamp, not deletion timestamp. This was the most semantic bug — no test exposed it.
- A-5: GSD-only correctly identified and fixed the term issue (uses `current_term`). v4 and dry CLAUDE.md kept the original commit term in the catch-up response.

### Category B: Bug Fixing (24 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| B-1 | WAL checksum includes header | 3 | **Pass** | **Pass** | **Pass** |
| B-2 | Memtable ordering (last not next) | 3 | **Pass** | **Pass** | **Pass** |
| B-3 | Compaction uses deleted_at | 3 | Fail | Fail | Fail |
| B-4 | Time-range boundary consistent | 2 | **Pass** | **Pass** | **Pass** |
| B-5 | Replication uses current term | 3 | Fail | **Pass** | Fail |
| B-6 | Timestamp serialization aligned | 2 | **Pass** | **Pass** | **Pass** |
| B-7 | WebSocket leak fixed | 2 | **Pass** | **Pass** | **Pass** |
| B-8 | Bug 1+2 compound handled | 3 | **Pass** | **Pass** | **Pass** |
| B-9 | Bug 3+4 compound recognized | 3 | Fail | Fail | Fail |
| | **Subtotal** | **24** | **14** | **17** | **14** |

**Notes:**
- B-3: None fixed the actual tombstone retention logic. All added TTL expiry to compaction (feature work), but the tombstone `deleted_at` vs `timestamp` bug persists.
- B-5: Only GSD-only used `*self.current_term.read().await` instead of the original commit term.
- B-8: All 3 fixed both WAL and memtable, achieving correct version semantics.
- B-9: Since Bug 3 wasn't fixed by any config, the compound relationship was never recognized.

### Category C: Feature Implementation (20 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| C-1 | TTL end-to-end | 5 | **Pass** | **Pass** | **Pass** |
| C-2 | Batch writes with atomicity | 5 | **Pass** | **Pass** | **Pass** |
| C-3 | Query aggregations | 5 | **Pass** | **Pass** | **Pass** |
| C-4 | Follower read routing | 5 | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **20** | **20** | **20** | **20** |

All 3 configs successfully implemented all 4 features end-to-end across Rust and TypeScript.

### Category D: Pipeline-Specific Process Quality (22 points)

#### D1: Decomposition Quality (8 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| D-1 | Structured breakdown BEFORE coding | 3 | Partial (.tasks/TRACKER.md created but after/during coding) | Fail | Partial (Docs/ROADMAP.md created) |
| D-2 | Identifies all 11 work items | 2 | **Pass** (7 bugs + 4 features listed) | Fail (no breakdown doc) | **Pass** (9 bugs + 4 features) |
| D-3 | Per-task context with file:line | 3 | **Pass** (TRACKER.md has file:line for every bug) | Fail | Partial (ROADMAP has file names, not line numbers) |
| | **Subtotal** | **8** | **6** | **0** | **4** |

#### D2: Planning & State Management (6 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| D-4 | Phase-based execution plan | 2 | Fail (monolithic commit) | Fail | **Pass** (Phase 01 + Phase 02) |
| D-5 | State tracking with honest progress | 2 | **Pass** (TRACKER.md) | Fail | **Pass** (SESSION_LOG.md) |
| D-6 | Evidence/verification per work unit | 2 | **Pass** (tests listed per bug) | Fail | **Pass** (verification section) |
| | **Subtotal** | **6** | **4** | **0** | **6** |

#### D3: Execution Discipline (8 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| D-7 | Pre-coding gates (repo health check) | 2 | Partial (ran cargo test first) | Fail | Partial (inspected codebase) |
| D-8 | Test-first (RED before GREEN) | 2 | Fail (tests written alongside) | Fail | Fail |
| D-9 | Fairness audit evidence | 2 | Fail | Fail | Fail |
| D-10 | Cross-language contract verification | 2 | **Pass** (fixed SDK timestamps) | **Pass** | **Pass** |
| | **Subtotal** | **8** | **3** | **1** | **3** |

### Category E: Technical Rigor (20 points)

| ID | Assertion | Wt | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----|-----------|---:|:-----------:|:--------:|:-------------:|
| E-1 | ≥7 regression tests for bug fixes | 3 | **Pass** (38 new Rust tests) | **Pass** (24 new) | **Pass** (20 new) |
| E-2 | Tests actually catch bugs | 3 | **Pass** | **Pass** | **Pass** |
| E-3 | cargo test passes | 2 | **Pass** (64 tests) | **Pass** (50) | **Pass** (52) |
| E-4 | tsc --noEmit passes | 2 | **Pass** | **Pass** | **Pass** |
| E-5 | cargo clippy clean | 2 | Partial (6 warnings, pre-existing) | Partial (8) | Partial (8) |
| E-6 | No test-gaming | 2 | **Pass** | **Pass** | **Pass** |
| E-7 | Scope discipline | 2 | **Pass** | **Pass** | Partial (found extra bugs) |
| E-8 | Compounding bugs handled | 2 | Partial (1+2 yes, 3+4 no, 5+6 partial) | Partial (1+2 yes, 3+4 no, 5+6 yes) | Partial (1+2 yes, 3+4 no, 5+6 partial) |
| E-9 | Features interact with bug fixes | 2 | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **20** | **16** | **16** | **15** |

---

## Final Scores

| Category | Max | v4 Pipeline | GSD-only | Dry CLAUDE.md |
|----------|----:|:-----------:|:--------:|:-------------:|
| A: Bug Discovery | 14 | 11 | **12** | 11 |
| B: Bug Fixing | 24 | 14 | **17** | 14 |
| C: Features | 20 | 20 | 20 | 20 |
| D: Process Quality | 22 | **13** | 1 | **13** |
| E: Technical Rigor | 20 | **16** | 16 | 15 |
| **TOTAL** | **100** | **74** | **66** | **73** |

---

## Analysis

### V4 Pipeline Leads — But Barely (74/100)

The v4 pipeline produced the most tests (77 total, 38 new Rust tests), created a structured task tracker with file:line citations (.tasks/TRACKER.md), and demonstrated the most thorough investigation. However, it:
- Used 40% more tokens than GSD-only (137.9K vs 98.7K)
- Took 37% longer (12.6 min vs 9.2 min)
- Still delivered a monolithic commit (not phased)
- Did NOT invoke /atomic (created .tasks/TRACKER.md manually, not via the skill)
- Did NOT run fairness audits or RED-first testing
- Did NOT fix the replication term bug (Bug 5) — used original commit term

### GSD-Only: Best at Bug Fixing, Worst at Process (66/100)

GSD-only was the only config to correctly fix the replication catch-up term bug. It was also the fastest (9.2 min) and most token-efficient (98.7K). However, it produced ZERO state documents, no task breakdown, and no evidence packs — pure execution with no process overhead.

### Dry CLAUDE.md v1: Best Documentation (73/100)

The original CLAUDE.md v1 produced the most comprehensive documentation: Docs/ROADMAP.md, Docs/SESSION_LOG.md, and phase folders (01/TODO.md, 02/TODO.md). It found 2 extra bugs beyond the planted 7 (empty value in replication commit, CLI timestamp). However, its documentation was strong on structure but weak on file:line precision compared to the v4 pipeline's TRACKER.md.

### The Compaction Tombstone Bug: Universal Failure

Bug 3 (compaction uses `entry.timestamp` instead of `entry.deleted_at`) was missed by ALL 3 configs. This validates the benchmark design — semantic bugs that don't manifest in obvious test failures are genuinely hard for AI agents. All 3 configs added TTL expiry to compaction (the feature), but none noticed the pre-existing retention logic was using the wrong timestamp field.

### Atomic Skill: Partial Trigger

The v4 pipeline created `.tasks/TRACKER.md` but did NOT invoke `/atomic` — it created the file manually as a tracking document, not through the 9-phase decomposition process. The Atomic hook was configured but the subagent benchmark environment doesn't process Claude Code hooks. In a real session with the hook active, the agent would have been blocked from editing files until running `/atomic`.

---

## Cumulative Rankings (V1-V4)

| Config | V1 (29) | V2 (29) | V3 (44) | V4 (100) | Total (202) | Avg |
|--------|---------|---------|---------|----------|-------------|-----|
| **v4 Pipeline** | 27 | 27 | 36 | **74** | **164** | **81%** |
| **Dry CLAUDE.md** | 24 | 24 | 36 | **73** | **157** | **78%** |
| GSD-only | 22 | 22 | 30 | 66 | 140 | 69% |

**The v4 pipeline leads cumulatively** at 81% across all 4 benchmarks. The Dry CLAUDE.md v1 closes the gap to just 7 points (157 vs 164). GSD-only consistently trails on process quality but compensates with speed and correct bug fixes.

---

## V4 Benchmark Difficulty Assessment

| Metric | V3 (CrateSync) | V4 (ChronoKV) | Change |
|--------|:--------------:|:--------------:|:------:|
| Source files | 10 | 19 | +90% |
| Total LOC | ~800 | 2,625 | +228% |
| Bugs planted | 4 (labeled) | 7 (unlabeled) | +75% |
| Bugs fully fixed (best config) | 4/4 (100%) | 6/7 (86%) | -14% |
| Features | 3 | 4 | +33% |
| Assertions | 20 | 40 | +100% |
| Max possible | 44 | 100 | +127% |
| Best score | 36/44 (82%) | 74/100 (74%) | -8% |
| Duration (best) | 4.4 min | 9.2 min | +109% |
| Tokens (best) | 56,208 | 98,748 | +76% |

**The benchmark achieved its goals**: duration increased 2x, scores dropped 8 percentage points, and one bug (compaction tombstone) was missed by ALL configs — proving the unlabeled semantic bug design works.

---

## Key Findings

1. **Unlabeled bugs are dramatically harder** — V3 had "BUG #N" labels and all 4 bugs were found/fixed by all configs. V4 removed labels and one bug (compaction tombstone) was universally missed.

2. **Compounding bugs partially work** — Bug 1+2 compound was handled by all 3 (both fixed independently). Bug 3+4 and 5+6 compounds were not fully resolved because Bug 3 was missed and Bug 5 was only fixed by GSD-only.

3. **Process overhead vs execution speed** — v4 pipeline used 40% more tokens for 8 more points than GSD-only. The cost per quality point: v4=1,864 tokens/point vs GSD=1,496 tokens/point. GSD is more efficient per point.

4. **State documents differentiate configs** — The v4 pipeline and dry CLAUDE.md both produced state documents (13/22 process points each). GSD-only produced none (1/22). Documentation IS the differentiator.

5. **Monolithic commits persist** — Despite all configs having access to git, all 3 produced a single monolithic commit. None delivered phased, atomic commits per task. This was true in V3 as well.

6. **Test-first development didn't happen** — No config wrote failing tests before implementing fixes (RED-first). All wrote tests alongside or after implementation.

---

## Recommendations

1. **Hook enforcement is critical** — The Atomic hook couldn't fire in the subagent benchmark environment. For real usage, hook-level enforcement (not prompt mandates) is the only way to ensure decomposition happens first.

2. **Compaction tombstone pattern is genuinely hard** — Use more of these "wrong field" semantic bugs. They test whether the agent understands the SEMANTICS of the data model, not just the syntax.

3. **Phased commits need stronger mandates** — All configs produced monolithic commits. Consider a hook that rejects commits touching more than N files.

4. **V4 pipeline's quality edge is real but narrow** — 74 vs 73 (just 1 point ahead of dry CLAUDE.md). The CLAUDE.md v4 quality layer adds value mainly through evidence-grounding (file:line citations) and test quantity, not through process discipline.

---

## Research Sources

Benchmark design informed by: FeatureBench (ICLR 2026, 11% pass), SWE-bench Pro (23.3%), SWE-bench+ (36pp test inflation), ProjDevBench (27%), METR HCAST (logistic difficulty curve), Terminal-Bench 2.0 (16% hard), LongCLI-Bench (<20%), MemoryArena (recall≠application), DependEval (cross-file #1 failure), Rust-SWE-bench (21.2%), RustAssistant (<30% lifetime errors), Anthropic Context Engineering (context rot), OpenDev arXiv:2603.05344 (hook enforcement), and 40+ additional sources documented in the plan.
