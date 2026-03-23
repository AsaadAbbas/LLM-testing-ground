# Benchmark V5 Report: 4-Way Pipeline Eval on ChronoKV

**Date**: 2026-03-23
**Model**: Claude Opus 4.6 (all configs)
**Task**: ChronoKV — 5 Rust crates + TypeScript SDK/CLI, 7 unlabeled bugs (3 compounding pairs), 4 cross-cutting features
**Prompt**: Identical unstructured braindump for all 4 configs

---

## Executive Summary

The **Raw Opus 4.6 baseline** (zero augmentation) was the ONLY config to address the compaction resurrection bug — the hardest planted bug that all V4 configs missed. Meanwhile, the **V5 pipeline** produced the most tests (72), the only multi-commit history, and the best evidence documentation. **Dry CLAUDE.md v1** produced 16 documentation files — by far the most comprehensive state management. **GSD-only** was the only config to correctly fix the replication catch-up term bug (again).

| Metric | V5 Pipeline | GSD-only | Dry CLAUDE.md v1 | Raw Opus 4.6 |
|--------|:-----------:|:--------:|:----------------:|:------------:|
| **Duration** | 13.1 min | 11.1 min | 12.4 min | **9.1 min** |
| **Tokens** | 135,694 | 105,198 | 115,094 | **90,353** |
| **Tool calls** | 112 | 91 | 116 | **82** |
| **Bugs found** | 8 | 8 | **10+** | 8 |
| **Compaction bug addressed?** | No | No | No | **Yes** (different approach) |
| **Replication term fixed?** | No (fixed deadlock) | **Yes** | No | No |
| **Features implemented** | 4/4 | 4/4 | 4/4 | 4/4 |
| **Rust tests** | **60** | 50 | 53 | 39 |
| **TS tests** | 12 | 12 | 10 | **18** |
| **Total tests** | **72** | 62 | 63 | 57 |
| **Git commits** | **2** (bugs + features) | 2 (state + work) | 1 (monolithic) | 1 (monolithic) |
| **State documents** | 1 (.tasks/TRACKER.md) | 1 (CHANGELOG.txt) | **16** (full Docs/) | 1 (CHANGELOG.txt) |
| **Evidence quality** | **Best** (BUG-NNN + file:line) | Minimal | Good (per-phase) | Minimal |

---

## Scoring (40 assertions, 100 points)

### Category A: Bug Discovery (14 points)

| ID | Assertion | Wt | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----|-----------|---:|:-----------:|:--------:|:-------------:|:--------:|
| A-1 | WAL checksum omission | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| A-2 | Memtable version ordering | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| A-3 | Compaction tombstone drop | 2 | Fail | Fail | Fail | **Pass** |
| A-4 | Time-range boundary mismatch | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| A-5 | Replication catch-up wrong term | 2 | Partial (deadlock) | **Pass** | Partial | Partial |
| A-6 | Timestamp serialization mismatch | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| A-7 | WebSocket subscription leak | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **14** | **11** | **12** | **11** | **13** |

**Key finding**: Raw Opus was the ONLY config to address Bug 3 (compaction resurrection). Its approach was different from the planted fix — instead of using `entry.deleted_at`, it tracked deleted keys and removed superseded PUT entries. This is arguably a MORE thorough fix.

### Category B: Bug Fixing (24 points)

| ID | Assertion | Wt | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----|-----------|---:|:-----------:|:--------:|:-------------:|:--------:|
| B-1 | WAL checksum includes header | 3 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-2 | Memtable ordering fixed | 3 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-3 | Compaction uses deleted_at | 3 | Fail | Fail | Fail | **Pass** |
| B-4 | Time-range boundary consistent | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-5 | Replication uses current term | 3 | Fail | **Pass** | Fail | Fail |
| B-6 | Timestamp serialization aligned | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-7 | WebSocket leak fixed | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-8 | Bug 1+2 compound handled | 3 | **Pass** | **Pass** | **Pass** | **Pass** |
| B-9 | Bug 3+4 compound recognized | 3 | Fail | Fail | Fail | **Pass** |
| | **Subtotal** | **24** | **14** | **17** | **14** | **20** |

### Category C: Feature Implementation (20 points)

| ID | Assertion | Wt | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----|-----------|---:|:-----------:|:--------:|:-------------:|:--------:|
| C-1 | TTL end-to-end | 5 | **Pass** | **Pass** | **Pass** | **Pass** |
| C-2 | Batch writes atomicity | 5 | **Pass** | **Pass** | **Pass** | **Pass** |
| C-3 | Query aggregations | 5 | **Pass** | **Pass** | **Pass** | **Pass** |
| C-4 | Follower read routing | 5 | **Pass** | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **20** | **20** | **20** | **20** | **20** |

### Category D: Process Quality (22 points)

| ID | Assertion | Wt | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----|-----------|---:|:-----------:|:--------:|:-------------:|:--------:|
| D-1 | Structured breakdown BEFORE coding | 3 | **Pass** (TRACKER.md) | Fail | Partial (ROADMAP) | Fail |
| D-2 | Identifies all 11 work items | 2 | **Pass** (8 bugs + 4 features) | Fail | **Pass** (10+ bugs + 4 feat) | Fail |
| D-3 | Per-task context with file:line | 3 | **Pass** (BUG-NNN + file:line) | Fail | Partial (file names) | Fail |
| D-4 | Phase-based execution plan | 2 | **Pass** (2 commits: bugs then features) | Fail | **Pass** (Phase 01 + 02) | Fail |
| D-5 | State tracking documents | 2 | **Pass** (TRACKER.md) | Partial (CHANGELOG) | **Pass** (SESSION_LOG + 16 files) | Partial (CHANGELOG) |
| D-6 | Evidence/verification per unit | 2 | **Pass** (tests per bug) | Fail | **Pass** (VERIFICATION.md per phase) | Fail |
| D-7 | Pre-coding gates | 2 | **Pass** (ran cargo test first) | Fail | Partial | Fail |
| D-8 | Test-first (RED before GREEN) | 2 | Fail | Fail | Fail | Fail |
| D-9 | Fairness audit evidence | 2 | Fail | Fail | Fail | Fail |
| D-10 | Cross-language contract verification | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **22** | **16** | **2** | **14** | **2** |

### Category E: Technical Rigor (20 points)

| ID | Assertion | Wt | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----|-----------|---:|:-----------:|:--------:|:-------------:|:--------:|
| E-1 | ≥7 regression tests | 3 | **Pass** (34 new) | **Pass** (30 new) | **Pass** (27 new) | **Pass** (24 new) |
| E-2 | Tests catch bugs | 3 | **Pass** | **Pass** | **Pass** | **Pass** |
| E-3 | cargo test passes | 2 | **Pass** (60) | **Pass** (50) | **Pass** (53) | **Pass** (39) |
| E-4 | tsc --noEmit passes | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| E-5 | cargo clippy clean | 2 | Partial | Partial | Partial | Partial |
| E-6 | No test-gaming | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| E-7 | Scope discipline | 2 | **Pass** | **Pass** | Partial (extras) | **Pass** |
| E-8 | Compounding bugs handled | 2 | Partial | Partial | Partial | **Pass** |
| E-9 | Features + bug fixes interact | 2 | **Pass** | **Pass** | **Pass** | **Pass** |
| | **Subtotal** | **20** | **16** | **16** | **15** | **17** |

---

## Final Scores

| Category | Max | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----------|----:|:-----------:|:--------:|:-------------:|:--------:|
| A: Bug Discovery | 14 | 11 | 12 | 11 | **13** |
| B: Bug Fixing | 24 | 14 | 17 | 14 | **20** |
| C: Features | 20 | 20 | 20 | 20 | 20 |
| D: Process | 22 | **16** | 2 | 14 | 2 |
| E: Technical | 20 | 16 | 16 | 15 | **17** |
| **TOTAL** | **100** | **77** | **67** | **74** | **72** |

---

## Analysis

### The Raw Opus Surprise

The most significant finding is that **Raw Opus 4.6 — with zero augmentation — scored 72/100**, just 5 points behind the full V5 pipeline. It was the ONLY config to address the compaction resurrection bug, doing so through an approach the benchmark designer didn't anticipate: tracking deleted keys and removing superseded PUT entries rather than using the `deleted_at` field.

This raises a fundamental question: **does the pipeline add enough value to justify its complexity?**

| Metric | V5 Pipeline | Raw Opus | Delta | Pipeline overhead |
|--------|:-----------:|:--------:|:-----:|:-----------------:|
| Score | 77 | 72 | +5 | +7% |
| Tokens | 135,694 | 90,353 | +45,341 | +50% |
| Duration | 13.1 min | 9.1 min | +4.0 min | +44% |
| Tests | 72 | 57 | +15 | +26% |

The V5 pipeline uses **50% more tokens for 7% more points**. Its advantage is entirely in Process (16 vs 2) — evidence documentation, task tracking, phased commits. On pure technical execution (Bug Fixing + Features + Technical Rigor), Raw Opus actually scores higher (57 vs 50 in B+C+E combined — wait, let me check: V5 B=14+C=20+E=16=50, Raw B=20+C=20+E=17=57). **Raw Opus beats V5 on technical execution by 7 points.**

### V5 Pipeline Improvements Over V4

| Metric | V4 Pipeline | V5 Pipeline | Change |
|--------|:-----------:|:-----------:|:------:|
| Score | 74 | 77 | +3 |
| Commits | 1 | **2** | Improved |
| Evidence quality | TRACKER.md | TRACKER.md with BUG-NNN | Same |
| Tests | 77 | 72 | -5 |
| Tokens | 137,926 | 135,694 | -2,232 |

The V5 pipeline improved on commit discipline (2 commits vs 1) but otherwise performed similarly. The hooks didn't fire in the subagent environment, so the enforcement gap persists.

### GSD-Only: Consistent Strengths

GSD-only again correctly fixed the replication catch-up term bug (the only config to do so in both V4 and V5). Its strength is raw bug-fixing correctness with minimal overhead.

### Dry CLAUDE.md v1: Documentation Champion (Again)

16 documentation files including per-phase CONTEXT.md, DECISIONS.md, VERIFICATION.md, HANDOFF.md, SOURCES.md — the full V1 operator folder system. This is exactly the pattern we're trying to give V5 subagents.

---

## Cumulative Rankings (V1-V5)

| Config | V1 (29) | V2 (29) | V3 (44) | V4 (100) | V5 (100) | Total (302) | Avg |
|--------|---------|---------|---------|----------|----------|-------------|-----|
| **V5 Pipeline** | 27 | 27 | 36 | 74 | **77** | **241** | **80%** |
| **Dry CLAUDE.md** | 24 | 24 | 36 | 73 | **74** | **231** | **77%** |
| **GSD-only** | 22 | 22 | 30 | 66 | **67** | **207** | **69%** |
| **Raw Opus** | — | — | — | — | **72** | **72** | **72%** |

Note: Raw Opus only participated in V5. Its 72% single-benchmark score exceeds GSD-only's 69% cumulative average.

---

## Key Findings

1. **Raw model baseline is surprisingly strong** — 72/100 with zero augmentation. The pipeline's 5-point advantage comes entirely from process discipline (documentation, task tracking, phased commits), not from technical execution quality.

2. **The compaction bug tells the real story** — All augmented configs (V5, GSD, dry CLAUDE.md) missed it. Raw Opus found and fixed it. This suggests the pipeline may introduce a form of "directed attention" that causes agents to follow prescribed investigation patterns rather than thinking freely about the code. The raw model had no prescribed methodology and just... read the code and fixed what was wrong.

3. **Hooks still don't fire in subagent environments** — The V5 pipeline's hooks (atomic-gate, commit-enforcer, verification-gate, cross-language-check) did not fire for the benchmark subagent. The 2-commit improvement was from CLAUDE.md v5's textual instructions, not hook enforcement.

4. **V1's documentation system IS the differentiator** — Dry CLAUDE.md v1's 16 Docs files are the strongest process artifact of any config. The V5.1 plan to give each subagent this system is validated by V1's consistent strong showing on process metrics.

5. **No config achieved RED-first or fairness audits** — D-8 and D-9 scored 0 across all 4 configs. These remain the hardest behavioral changes to induce through any mechanism (prompt, hook, or skill).

6. **Token efficiency matters** — Raw Opus: 1,254 tokens/point. GSD: 1,570. Dry CLAUDE.md: 1,555. V5 Pipeline: 1,762. The pipeline with the most augmentation is the least token-efficient.

---

## Recommendations

1. **The pipeline's value is PROCESS, not CODE QUALITY** — When choosing a config, the question is: do you need documentation, evidence trails, and structured task management? If yes, use V5. If you just need bugs fixed and features shipped, Raw Opus is faster and cheaper.

2. **Consider lighter augmentation** — A CLAUDE.md that's just 20-30 lines of hard rules (no tools section, no relationship table) might capture 80% of the process benefit at 20% of the token cost.

3. **The compaction bug finding is humbling** — Prescribed investigation protocols may create tunnel vision. The raw model's unstructured approach found a bug that 3 augmented configs missed. Consider adding "freeform investigation time" to the pipeline.

4. **Hook enforcement needs real-session testing** — The benchmark subagent environment doesn't process hooks. To validate hook value, test in actual Claude Code sessions, not benchmark subagents.

5. **V1's operator folder system for subagents is validated** — The dry CLAUDE.md v1's documentation discipline produces measurably better process scores. The V5.1 plan to give each subagent its own operator folder (via SubagentStart hook) is the right direction.
