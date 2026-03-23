# LLM Framework Benchmark: Comprehensive Pipeline Evaluation

## Overview

A rigorous, multi-round benchmarking study comparing 4 AI coding pipeline configurations across 6 benchmark rounds, graded by 3 specialist architect agents on 20 quality dimensions.

**Model**: Claude Opus 4.6 (1M context) for all configurations
**Research basis**: 56+ authoritative sources (FeatureBench ICLR 2026, SWE-bench Pro, METR HCAST, ProjDevBench, Terminal-Bench, Anthropic Context Engineering, OpenDev arXiv:2603.05344, and more)

---

## Configurations Tested

| Config | Components | Purpose |
|--------|-----------|---------|
| **V5 Full Pipeline** | CLAUDE.md v5 (208 lines) + 6 hooks + Atomic skill + GSD + all skills | Maximum augmentation |
| **Raw GSD** | GSD framework only. No CLAUDE.md, no hooks, no Atomic | Framework-only baseline |
| **Dry CLAUDE.md v1** | Original 561-line autonomous execution prompt. No GSD, no hooks | Prompt-only baseline |
| **Raw Opus 4.6** | Zero augmentation. No CLAUDE.md, no hooks, no skills, no framework | True model baseline |

---

## Final Results

### V6 Quality Assessment (/500) — 20 Dimensions, 3 Specialist Reviewers

| Rank | Config | Security (/175) | Architecture (/175) | Testing (/150) | **TOTAL** | **Pct** |
|------|--------|:---------------:|:-------------------:|:--------------:|:---------:|:-------:|
| **1** | **Raw Opus 4.6** | 125 | **167** | **139** | **431** | **86.2%** |
| **2** | V5 Pipeline | **129** | 157 | 126 | 412 | 82.4% |
| **3** | Dry CLAUDE.md | 109 | 149 | 113 | 371 | 74.2% |
| **4** | GSD-only | 84 | 139 | 93 | 316 | 63.2% |

### Cumulative Functional Rankings (6 Benchmarks, /402)

| Config | V1 | V2 | V3 | V4 | V5 | V6 | Total | Avg |
|--------|---:|---:|---:|---:|---:|---:|------:|----:|
| V5 Pipeline | 27 | 27 | 36 | 74 | 77 | 88 | **329** | **82%** |
| Dry CLAUDE.md | 24 | 24 | 36 | 73 | 74 | 84 | 315 | 78% |
| Raw Opus | -- | -- | -- | -- | 72 | 81 | 153 | 77% |
| GSD-only | 22 | 22 | 30 | 66 | 67 | 78 | 285 | 71% |

### Key Finding

**Raw Opus 4.6 — with zero augmentation — produces the highest-quality code (431/500, 86.2%), winning 17 of 20 quality dimensions.** The V5 pipeline adds security discipline (wins 3 security dimensions) and process insurance (evidence trails, task tracking), but does not improve raw code quality. The pipeline's value is auditability, not capability.

---

## Research Sources

56+ authoritative sources documented in individual benchmark reports. See `results/` directory for full citations.
