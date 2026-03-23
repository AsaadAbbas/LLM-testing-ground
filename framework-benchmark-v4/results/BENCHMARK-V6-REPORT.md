# Benchmark V6 Report: 4-Way Eval on Nexus (Harder Middleware Pipeline)

**Date**: 2026-03-23
**Model**: Claude Opus 4.6 (all configs)
**Task**: Nexus — 5 Rust crates + TypeScript SDK. Plugin-based API gateway with 7 bugs targeting Opus's weaknesses + 5 architectural features.
**Difficulty**: Harder than ChronoKV — middleware ordering semantics, auth timing attacks, integer overflow, cross-language serde encoding.

---

## Executive Summary

**All 4 configs fixed ALL 7 bugs including the auth timing attack** — a significant finding. The harder Nexus benchmark did NOT differentiate configs on bug-finding ability. The differentiation came from: test volume, feature implementation depth, and process documentation.

**Raw Opus 4.6 produced the most tests (121)** and modified the core `AdminEvent` type to fix the cross-language serde issue — the deepest architectural change of any config. The V5 pipeline also scored 101 tests. All configs identified and applied constant-time comparison for the auth vulnerability.

| Metric | V5 Pipeline | GSD-only | Dry CLAUDE.md v1 | Raw Opus 4.6 |
|--------|:-----------:|:--------:|:----------------:|:------------:|
| **Duration** | 11.2 min | **9.8 min** | 10.2 min | 12.8 min |
| **Tokens** | 97,318 | 98,623 | **97,379** | 105,510 |
| **Tool calls** | 67 | 81 | 67 | 66 |
| **Bugs fixed** | 7/7 | 7/7 | 7/7 | 7/7 |
| **Features** | 5/5 | 5/5 | 5/5 | 5/5 |
| **Rust tests** | 84 | 66 | 69 | **96** |
| **TS tests** | 17 | 17 | 14 | **25** |
| **Total tests** | **101** | 83 | 83 | **121** |
| **Commits** | 2 | 2 | 2 | 2 |
| **Auth fix quality** | constant_time + black_box | constant_time XOR | constant_time XOR | constant_time + redacted keys |
| **AdminEvent fix** | Kept Rust tagging, fixed SDK parser | Kept Rust tagging, fixed SDK parser | Kept Rust tagging, fixed SDK parser | **Changed Rust type to flat struct** |

---

## Key Findings

### 1. All Configs Solved the Auth Timing Attack

This was expected to be the hardest bug — requiring security domain knowledge. All 4 configs:
- Identified the timing side-channel in `validate_key`'s early return
- Implemented constant-time byte comparison (XOR accumulation)
- Ensured all keys are checked regardless of match

The V5 pipeline additionally used `std::hint::black_box` to prevent compiler optimization. Raw Opus additionally redacted stored key values and rejected non-Bearer schemes.

### 2. Raw Opus Made the Deepest Architectural Fix

For Bug 7 (AdminEvent serde tagging), 3 configs kept the Rust `#[serde(tag, content)]` and fixed the TypeScript parser. Raw Opus **changed the Rust type itself** from an adjacently-tagged enum to a flat struct with `event_type` + `payload`, making it inherently TypeScript-compatible. This is the more maintainable fix — it eliminates the cross-language mismatch at the source.

### 3. Test Volume: Raw Opus Leads Again

| Config | Rust Tests | TS Tests | Total |
|--------|-----------|---------|-------|
| Raw Opus | **96** | **25** | **121** |
| V5 Pipeline | 84 | 17 | 101 |
| Dry CLAUDE.md | 69 | 14 | 83 |
| GSD-only | 66 | 17 | 83 |

Raw Opus produced 46% more tests than the average of the other 3 configs (121 vs 89 avg). This continues the V5 benchmark trend where raw Opus demonstrated strong technical execution without augmentation.

### 4. The Nexus Benchmark Was Still Too Easy

All 4 configs found and fixed ALL 7 bugs and implemented ALL 5 features. The benchmark failed to differentiate on core competency. The bugs (middleware ordering, path matching, integer truncation, timing attack) were all discoverable through careful code reading — even the security vulnerability.

To create a benchmark that Opus 4.6 genuinely struggles with, the task would need:
- **Cross-file trait implementation bugs** that require understanding Rust's orphan rules
- **Lifetime annotation errors** that require architectural restructuring (not local fixes)
- **Feature flag interaction bugs** that only manifest in workspace builds
- **Proc macro expansion bugs** that require reading generated code
- **Async Send/Sync bound violations** that cascade through trait bounds

These patterns have <30% fix rates per RustAssistant research but are much harder to embed as naturalistic bugs in a test fixture.

### 5. Token Efficiency Converged

All 4 configs used remarkably similar token counts (97-106K), unlike ChronoKV where the spread was 90-136K. This suggests the Nexus codebase's smaller size (12 files, 1749 LOC) created less opportunity for divergent investigation strategies.

---

## Scoring Summary (/100)

Based on the same 40-assertion rubric adapted for Nexus:

| Category | Max | V5 Pipeline | GSD-only | Dry CLAUDE.md | Raw Opus |
|----------|----:|:-----------:|:--------:|:-------------:|:--------:|
| Bug Discovery | 14 | 14 | 14 | 14 | 14 |
| Bug Fixing | 24 | 24 | 24 | 24 | 24 |
| Features | 20 | 20 | 20 | 20 | 20 |
| Process | 22 | 12 | 4 | 10 | 4 |
| Technical | 20 | 18 | 16 | 16 | 19 |
| **TOTAL** | **100** | **88** | **78** | **84** | **81** |

The V5 pipeline wins on Process (evidence documentation, task tracking). Raw Opus wins on Technical (most tests, deepest architectural fix). All configs tie on correctness (Bug Discovery + Bug Fixing + Features = 58/58 for all).

---

## Cumulative All-Benchmark Rankings

| Config | V1 | V2 | V3 | V4 | V5-ChronoKV | V6-Nexus | Total (/402) | Avg |
|--------|---:|---:|---:|---:|:----------:|:--------:|:------------:|----:|
| **V5 Pipeline** | 27 | 27 | 36 | 74 | 77 | **88** | **329** | **82%** |
| **Dry CLAUDE.md** | 24 | 24 | 36 | 73 | 74 | 84 | **315** | **78%** |
| **Raw Opus** | — | — | — | — | 72 | 81 | **153** | **77%** |
| **GSD-only** | 22 | 22 | 30 | 66 | 67 | 78 | **285** | **71%** |

Note: Raw Opus only participated in V5+V6. Its per-benchmark average (77%) is competitive with Dry CLAUDE.md (78%) and close to V5 Pipeline (82%).

---

## Conclusions

1. **The V5 pipeline's cumulative lead holds** at 82% across 6 benchmarks. Its advantage is process discipline — evidence documentation, task tracking, structured commits.

2. **Raw Opus 4.6 is remarkably strong unaugmented**. At 77% average across 2 benchmarks, it matches Dry CLAUDE.md's cumulative 78% and nearly matches V5 Pipeline on pure technical execution.

3. **Current benchmarks cannot make Opus fail on bugs**. Both ChronoKV and Nexus achieved 100% bug-fix rates across all configs. To create genuine differentiation, benchmarks would need bugs requiring deep type system understanding (lifetimes, trait bounds, proc macros) — patterns that are inherently harder to embed in test fixtures.

4. **The pipeline's value is insurance, not capability**. The augmented configs don't find MORE bugs or write BETTER code. They provide process artifacts (evidence trails, documentation, task tracking) that make the work auditable and resumable. Whether that's worth the token overhead depends on the use case.
