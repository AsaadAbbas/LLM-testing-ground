# LLM Framework Benchmark

A rigorous, research-grounded benchmarking study comparing AI coding pipeline configurations. Six benchmark rounds, four configurations, twenty quality dimensions, three specialist reviewer agents, fifty-six research sources.

## The Question

> Does wrapping an LLM in a framework (skills, hooks, CLAUDE.md, GSD) produce better code than the raw model alone?

## The Answer

**No — not for code quality. Yes — for process discipline.**

Raw Claude Opus 4.6 with zero augmentation scored **431/500 (86.2%)** on a 20-dimension quality assessment, winning 17 of 20 dimensions. The fully augmented V5 pipeline scored 412/500 (82.4%). The pipeline's advantage is entirely in security discipline and process artifacts — evidence trails, task tracking, phased commits. On raw architectural quality and testing depth, the unaugmented model is better.

---

## Results at a Glance

### Quality Assessment (V6 Nexus Benchmark, /500)

| Rank | Config | Security | Architecture | Testing | Total | Pct |
|------|--------|:--------:|:------------:|:-------:|:-----:|:---:|
| 1 | Raw Opus 4.6 | 125 | **167** | **139** | **431** | **86.2%** |
| 2 | V5 Pipeline | **129** | 157 | 126 | 412 | 82.4% |
| 3 | Dry CLAUDE.md v1 | 109 | 149 | 113 | 371 | 74.2% |
| 4 | GSD-only | 84 | 139 | 93 | 316 | 63.2% |

### Functional Benchmark (Cumulative, 6 Rounds, /402)

| Config | V1 | V2 | V3 | V4 | V5 | V6 | Total | Avg |
|--------|---:|---:|---:|---:|---:|---:|------:|:---:|
| V5 Pipeline | 27 | 27 | 36 | 74 | 77 | 88 | **329** | **82%** |
| Dry CLAUDE.md v1 | 24 | 24 | 36 | 73 | 74 | 84 | 315 | 78% |
| Raw Opus 4.6 | — | — | — | — | 72 | 81 | 153 | 77% |
| GSD-only | 22 | 22 | 30 | 66 | 67 | 78 | 285 | 71% |

---

## Configurations

| Config | What It Is | Strengths | Weaknesses |
|--------|-----------|-----------|------------|
| **V5 Full Pipeline** | CLAUDE.md v5 + 6 hooks + Atomic skill + GSD framework + all skills | Security discipline, evidence trails, process auditability | 50% more tokens, may constrain freeform reasoning |
| **Raw GSD** | GSD framework only, no CLAUDE.md or hooks | Fastest execution, wave-based parallelism | Weakest security, no documentation |
| **Dry CLAUDE.md v1** | Original 561-line autonomous execution prompt, no framework | Best documentation (16 Docs files), good balance | Monolithic commits, no hook enforcement |
| **Raw Opus 4.6** | Zero augmentation — no CLAUDE.md, no hooks, no skills | Highest code quality, deepest fixes, most tests | No process artifacts, no auditability |

---

## Test Projects

### ChronoKV (V4/V5 Benchmarks)
A time-series key-value store in Rust with a TypeScript SDK.
- 5 Rust crates + TypeScript SDK/CLI (19 files, 2,625 LOC)
- 7 unlabeled bugs with 3 compounding pairs
- 4 cross-cutting features (TTL, batch writes, aggregations, follower routing)

### Nexus (V6 Benchmark)
A plugin-based API gateway / middleware pipeline framework.
- 5 Rust crates + TypeScript SDK (12 files, 1,749 LOC)
- 7 bugs targeting Opus 4.6's specific weaknesses (auth timing attack, middleware ordering, integer overflow, path matching bypass, serde encoding mismatch)
- 5 architectural features (body size limit, ETag caching, health checks, hot config reload, admin API)

---

## Key Findings

### 1. Raw Model Produces Highest-Quality Code
Raw Opus 4.6 won 17/20 quality dimensions with zero augmentation. It produced the deepest architectural fix (refactoring `AdminEvent` from a serde-tagged enum to a flat struct), the most tests (121 in V6), and was the only config to fix the compaction tombstone bug in V5.

### 2. Pipelines Add Security, Not Capability
The V5 pipeline's 3 dimension wins are all security-specific: `std::hint::black_box()` for constant-time auth, SSE buffering, body-limit-first ordering. CLAUDE.md security rules provide measurable security improvements the raw model doesn't naturally produce.

### 3. Prescribed Patterns May Create Tunnel Vision
In V5, Raw Opus was the ONLY config to fix the semantic "wrong field" compaction bug — a bug all 3 augmented configs missed across two benchmark rounds. Prescribed investigation patterns may direct attention along methodology rails rather than letting the model reason freely about code semantics.

### 4. Functional Benchmarks Miss Quality Differences
All 4 configs fixed 7/7 bugs in V6 — the functional benchmark couldn't differentiate them. The quality assessment revealed a 115-point spread (431 vs 316) in HOW they fixed them.

### 5. The Pipeline's Value Is Insurance
The V5 pipeline uses 50% more tokens for 7% more functional points. Its advantage is process artifacts: evidence trails, task tracking, phased commits, reproducible methodology. Whether that's worth the overhead depends on whether you need auditability or just results.

---

## V5 Pipeline Architecture

```
User Input
  |
  v
Atomic Gate Hook (PreToolUse on Write|Edit)
  - <=1 file AND <=10 lines -> pass (trivial)
  - .tasks/ exists -> pass (Atomic already ran)
  - Otherwise -> deny (must run /atomic first)
  |
  v
/atomic (9-phase decomposition -> .tasks/)
  |
  v
SubagentStart Hook -> injects quality rules into every subagent
  |
  v
GSD or Issue-by-Issue Execution
  - Each subagent gets operator folder (.tasks/.operators/{id}/)
  - CONTEXT.md, DECISIONS.md, VERIFICATION.md per operator
  |
  v
Evidence Packs per task -> Verification Gate -> Commit
```

### Hook System

| Hook | Event | Purpose |
|------|-------|---------|
| `atomic-gate.js` | PreToolUse (Write/Edit) | Blocks non-trivial edits until `.tasks/` exists |
| `commit-enforcer.js` | PreToolUse (Bash) | Warns on >8 staged files per commit |
| `verification-gate.js` | UserPromptSubmit | Injects verification reminders |
| `cross-language-check.js` | PostToolUse (Edit/Write) | Reminds about Rust/TS alignment on shared types |
| `subagent-quality-inject.js` | SubagentStart | Creates operator folders + injects quality rules |
| `subagent-cleanup.js` | SubagentStop | Merges artifacts, deletes temp folders |

---

## Methodology

### Benchmark Design
- **Research-grounded**: 56+ sources from FeatureBench (ICLR 2026), SWE-bench Pro, METR HCAST, ProjDevBench, Terminal-Bench, Anthropic Context Engineering, and more
- **Unlabeled bugs**: No `// BUG #N` comments — agents discover bugs through analysis
- **Identical prompt**: Every config receives the same unstructured braindump
- **Cross-language**: Rust backend + TypeScript SDK tests serialization alignment
- **Compounding bugs**: Some bugs interact, requiring sequential reasoning

### Grading
- **Functional**: 40 assertions across 5 categories scored /100
- **Quality**: 20 dimensions across 3 specialist domains scored /500
  - Security Architect (7 dimensions): auth timing, input validation, rate limiting, info disclosure, admin API, cross-language, pipeline security
  - Software Architect (7 dimensions): module boundaries, API design, concurrency, data model, error handling, extensibility, cross-language contract
  - Test Architect (6 dimensions): regression coverage, feature completeness, negative testing, anti-triviality, integration, infrastructure

### Specialist Reviewers
Each quality dimension is scored 0-25 with specific `file:line` citations required. Three independent specialist agents review ALL 4 implementations comparatively — not one agent per implementation, but one agent per domain across all implementations.

---

## Research Sources (Top 20)

| # | Source | Key Finding |
|---|--------|-------------|
| 1 | [FeatureBench (ICLR 2026)](https://arxiv.org/abs/2602.10975) | 11% pass rate; 790 LOC, 15.7 files per task |
| 2 | [SWE-bench Pro](https://arxiv.org/abs/2509.16941) | 23.3% best; multi-file industrial tasks |
| 3 | [METR HCAST](https://metr.org/time-horizons/) | Logistic curve: <10% at 4h+ human time |
| 4 | [ProjDevBench](https://arxiv.org/abs/2602.01655) | 27% acceptance; system design gap |
| 5 | [Terminal-Bench 2.0](https://www.tbench.ai/) | 50% ceiling; 16% on hard tasks |
| 6 | [Anthropic Context Engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) | Context rot; Write/Select/Compress/Isolate |
| 7 | [OpenDev](https://arxiv.org/abs/2603.05344) | Hook enforcement; 88% prompt cache savings |
| 8 | [DependEval (ACL 2025)](https://aclanthology.org/2025.findings-acl.373.pdf) | Cross-file dependency is #1 LLM failure |
| 9 | [GSD Framework](https://github.com/gsd-build/get-shit-done) | Wave-based parallel execution; 57 commands |
| 10 | [Claude Code Hooks](https://code.claude.com/docs/en/hooks) | 24 hook events; PreToolUse can block/allow |
| 11 | [EvalPlus](https://evalplus.github.io/leaderboard.html) | 80x tests -> 19-29% drop |
| 12 | [Rust-SWE-bench](https://arxiv.org/abs/2602.22764) | 21.2% resolve; borrow checker is barrier |
| 13 | [RustAssistant](https://arxiv.org/abs/2308.05177) | <30% fix rate on lifetime errors |
| 14 | [Context Rot (Chroma)](https://research.trychroma.com/context-rot) | 11/12 models degrade at 32k tokens |
| 15 | [LongCLI-Bench](https://arxiv.org/abs/2602.14337) | <20%; multiplicative failure P=p^N |
| 16 | [MemoryArena](https://arxiv.org/abs/2602.16313) | Recall != application in agentic settings |
| 17 | [CodeGlance](https://arxiv.org/abs/2602.13962) | Unseen reasoning patterns 6x harder |
| 18 | [Aider Polyglot](https://aider.chat/docs/leaderboards/) | Multi-language has lowest pass rates |
| 19 | [LiveCodeBench](https://livecodebench.github.io/) | Contamination-free evaluation |
| 20 | [SWE-bench+](https://arxiv.org/abs/2410.06992) | 36pp inflation from weak tests |

---

## Repository Structure

```
LLM-testing-ground/
├── .claude/hooks/                          # V5 pipeline hooks (6 files)
├── framework-benchmark/                    # V1 benchmark (TaskFlow)
│   └── results/BENCHMARK-REPORT.md
├── framework-benchmark-v2/                 # V2 benchmark (SyncBoard)
│   ├── CLAUDE-v3-enhanced.md              # CLAUDE.md v4 (392 lines)
│   └── results/BENCHMARK-V2-REPORT.md
├── framework-benchmark-v3/                 # V3 benchmark (CrateSync)
│   ├── test-project/cratesync/            # Rust workspace + TS CLI
│   └── results/BENCHMARK-V3-REPORT.md
├── framework-benchmark-v4/                 # V4-V6 benchmarks
│   ├── CLAUDE-v5-enhanced.md              # CLAUDE.md v5 (208 lines)
│   ├── test-project/chronokv/             # ChronoKV (V4/V5)
│   ├── test-project-v6/nexus/             # Nexus (V6)
│   ├── eval-harness/benchmark-v4.json     # 40-assertion grading rubric
│   └── results/
│       ├── FINAL-REPORT.md                # Comprehensive summary
│       ├── V6-QUALITY-ASSESSMENT-500.md   # 20-dimension /500 assessment
│       ├── ARCHITECT-REVIEW-AND-V5-PLAN.md
│       ├── BENCHMARK-V4-REPORT.md
│       ├── BENCHMARK-V5-REPORT.md
│       └── BENCHMARK-V6-REPORT.md
└── README.md
```

---

## License

This research and all benchmark artifacts are provided for educational and research purposes.
