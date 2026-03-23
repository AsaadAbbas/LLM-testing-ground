# V4 Benchmark: Architect-Review Code Quality Assessment (/500)

## Executive Summary

Three independent Principal Architect agents reviewed every line of every diff (~1,700 lines each) across the 3 benchmark configurations, scoring against a 25-metric rubric (each 0-20, total /500).

### Final Scores

| Rank | Config | Score | Pct | Strongest | Weakest |
|------|--------|------:|----:|-----------|---------|
| **1** | **v4 Pipeline** | **387/500** | **77.4%** | Correctness (81%) | Process (70%) |
| 2 | GSD-only | 341/500 | 68.2% | Correctness (76%) | Process (46%) |
| 3 | Dry CLAUDE.md v1 | 340/500 | 68.0% | Architecture (76%) | Process (57%) |

### Category Breakdown

| Category (each /100) | v4 Pipeline | GSD-only | Dry CLAUDE.md | Winner |
|----------------------|:-----------:|:--------:|:-------------:|--------|
| **Correctness** | **81** | 76 | 73 | v4 Pipeline |
| **Code Quality** | **79** | 73 | 70 | v4 Pipeline |
| **Architecture** | **80** | 76 | 76 | v4 Pipeline |
| **Testing** | **77** | 67 | 66 | v4 Pipeline |
| **Process & Docs** | **70** | 46 | 57 | v4 Pipeline |

**The v4 pipeline wins every single category.** Its lead is largest in Process (+13 over dry CLAUDE.md, +24 over GSD) and Testing (+10/+11). Its lead is narrowest in Architecture (+4 over both).

### Per-Metric Scores (25 metrics, each /20)

#### Correctness
| # | Metric | v4 | GSD | Dry |
|---|--------|---:|----:|----:|
| 1 | Bug Fix Correctness | **18** | 16 | 16 |
| 2 | Feature Completeness | 16 | **17** | 16 |
| 3 | Edge Case Handling | **14** | 13 | 12 |
| 4 | Cross-Language Contract | **17** | 16 | 15 |
| 5 | Regression Safety | **16** | 14 | 14 |

#### Code Quality
| # | Metric | v4 | GSD | Dry |
|---|--------|---:|----:|----:|
| 6 | Idiomatic Rust | **17** | 15 | 16 |
| 7 | Idiomatic TypeScript | 15 | 15 | 13 |
| 8 | Error Handling | 15 | 14 | 13 |
| 9 | Naming & Readability | **18** | 16 | 16 |
| 10 | Minimal Change Principle | 14 | 13 | 12 |

#### Architecture
| # | Metric | v4 | GSD | Dry |
|---|--------|---:|----:|----:|
| 11 | Module Boundaries | **17** | 16 | 16 |
| 12 | API Design | 16 | 16 | 15 |
| 13 | Concurrency Safety | 16 | 15 | 16 |
| 14 | Data Model Integrity | 15 | 14 | 14 |
| 15 | Separation of Concerns | 16 | 15 | 15 |

#### Testing
| # | Metric | v4 | GSD | Dry |
|---|--------|---:|----:|----:|
| 16 | Test Coverage | 17 | 16 | 15 |
| 17 | Test Quality | **17** | 15 | 16 |
| 18 | Test Isolation | **18** | **17** | **17** |
| 19 | Negative Testing | 13 | 11 | 10 |
| 20 | Integration Testing | 12 | 8 | 8 |

#### Process & Documentation
| # | Metric | v4 | GSD | Dry |
|---|--------|---:|----:|----:|
| 21 | Commit Discipline | 8 | 4 | 4 |
| 22 | State Documentation | **17** | 6 | 14 |
| 23 | Investigation Depth | **18** | 14 | 15 |
| 24 | Verification Evidence | 10 | 10 | 14 |
| 25 | Compounding Bug Recognition | **17** | 12 | 10 |

---

### Key Findings from Architect Reviews

**1. v4 Pipeline's quality layer produces measurably better code**
The CLAUDE.md v4 evidence-grounding rules show up in the scores: Bug Fix Correctness (18 vs 16/16), Naming & Readability (18 vs 16/16), Investigation Depth (18 vs 14/15), Compounding Bug Recognition (17 vs 12/10). The quality layer doesn't just produce documentation — it produces better root cause analysis and more precise fixes.

**2. All 3 configs share the same critical weakness: monolithic commits**
Commit Discipline scored 8/4/4. Even the v4 pipeline (which scored highest at 8) only marginally improved. Despite CLAUDE.md v4 mandating "one atomic commit per task," the agent bundled everything into a single commit. This is the strongest evidence yet that commit discipline requires hook enforcement, not prompt mandates.

**3. Negative and integration testing are universally weak**
Negative Testing: 13/11/10. Integration Testing: 12/8/8. No config tested error paths comprehensively, and none produced cross-language integration tests. This is consistent with FeatureBench's finding that AssertionError (logic gaps despite executability) is the dominant failure mode.

**4. GSD-only had the best individual bug fix (replication term)**
GSD scored 16/20 on Bug Fix Correctness while v4 scored 18 — but GSD was the ONLY config to correctly fix the replication catch-up term (using `current_term` instead of original commit term). The v4 pipeline and dry CLAUDE.md both left this bug in place.

**5. Dry CLAUDE.md v1 produced the best verification evidence**
Verification Evidence: 14/20 for dry CLAUDE.md vs 10/10 for the others. The SESSION_LOG.md included per-crate test counts. However, it still lacked `cargo clippy` and raw command output.

**6. Feature completeness gap in v4 pipeline**
The architect review found that v4 pipeline's FEAT-4 (follower read routing) was implemented as library-only — `can_serve_reads()` and `set_follower_reads()` exist but are never called from server routes. GSD and dry CLAUDE.md both wired this into the server.

---

### Universal Weaknesses (All 3 Configs)

| Weakness | v4 | GSD | Dry | Research Basis |
|----------|---:|----:|----:|----------------|
| Batch atomicity is illusory | Yes | Yes | Yes | WAL writes are individual; no rollback on partial failure |
| No HTTP integration tests | Yes | Yes | Yes | FeatureBench: cross-file integration is #1 failure |
| No cross-language tests | Yes | Yes | Yes | Aider Polyglot: multi-language has lowest pass rates |
| Compaction tombstone bug missed | Yes | Yes | Yes | Semantic bugs are frontier-level hard |
| `let _ =` silences replication errors | Yes | Yes | Yes | Error swallowing is a top code smell |
| `any` in CLI TypeScript | Yes | Yes | Yes | Type erasure at boundaries |
| TTL logic duplicated in 3 places | Yes | Yes | Yes | DRY violation; single `Entry::is_expired()` needed |

---

### Combined Scoring: Benchmark (/100) + Architect Review (/500) = /600

| Config | Benchmark | Architect | Combined | Combined % |
|--------|----------:|----------:|---------:|-----------:|
| **v4 Pipeline** | 74 | 387 | **461** | **76.8%** |
| Dry CLAUDE.md | 73 | 340 | 413 | 68.8% |
| GSD-only | 66 | 341 | 407 | 67.8% |

The v4 pipeline's lead widens from 1 point (benchmark only) to **48 points** when code quality is factored in. The CLAUDE.md v4 quality layer's impact is most visible in code quality metrics that the functional benchmark doesn't capture: naming, investigation depth, test quality, and compounding bug recognition.

---

---

# V5 Pipeline Improvement Plan: Research-Backed Redesign

## Context

The v4 pipeline (Atomic + CLAUDE.md v4 + GSD) scored 387/500 (77.4%) on architect review and 74/100 on functional benchmark. While it won every category, systematic weaknesses emerged that are addressable through research-backed changes. This plan maps each weakness to specific research findings and proposes concrete improvements.

## Current V4 Architecture (Full Map)

```
User Input (braindump/spec/task)
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│  ATOMIC GATE HOOK (PreToolUse on Write|Edit)             │
│  .claude/hooks/atomic-gate.js                            │
│  Gate: <=1 file AND <=10 lines → pass                    │
│  Otherwise → deny until .tasks/ exists                   │
│  STATUS: Implemented but doesn't fire in subagents       │
└──────────────────────────────────────────────────────────┘
       │ (if non-trivial)
       ▼
┌──────────────────────────────────────────────────────────┐
│  ATOMIC SKILL (/atomic)                                  │
│  ~/.claude/skills/atomic/SKILL.md (245 lines)            │
│  + references/ (5 files, 736 lines)                      │
│                                                          │
│  9 Phases: Parse → Grasp Gate → Engineering Analysis →   │
│  Dependency Model → Breakdown → Architecture Validation →│
│  Fairness Audit → Generate Artifacts → Summary           │
│                                                          │
│  Output: .tasks/ directory                               │
│    ├── PRD.md (for GSD --auto)                           │
│    ├── MANIFEST.md (task inventory)                      │
│    ├── DEPENDENCY-GRAPH.md (Mermaid)                     │
│    ├── ANALYSIS-LOG.md                                   │
│    ├── TASK-NNN/context.md (per-task evidence)           │
│    └── phases/phase-N-PRD.md (for GSD --prd)             │
│                                                          │
│  STATUS: Never invoked in V3 or V4 benchmarks            │
└──────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│  ENHANCED CLAUDE.md v4 (Cross-Cutting Quality Layer)     │
│  framework-benchmark-v2/CLAUDE-v3-enhanced.md (392 lines)│
│                                                          │
│  17 Sections:                                            │
│  ✓ Evidence-Grounding Rules     ✓ Investigation Protocol │
│  ✓ Self-Audit Checkpoints       ✓ Scope Discipline       │
│  ✓ State Document Quality       ✓ Anti-Test-Gaming       │
│  ✓ Brownfield Awareness         ✓ Rust/TS Standards      │
│  ✓ Available Tools              ✓ CLI-First MCP-Last     │
│  ✓ Working Memory Management    ✓ Execution Discipline   │
│  ✓ Execution Modes             ✓ Prompt Injection        │
│  ✓ Safety                       ✓ Atomic-First Workflow  │
│  ✓ Relationship to GSD                                   │
│                                                          │
│  STATUS: Read but not fully followed. Pre-coding gates,  │
│  RED-first, fairness audits, atomic commits all ignored.  │
└──────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────┐
│  GSD FRAMEWORK (Execution Layer)                         │
│  .claude/get-shit-done/ (v1.28.0)                        │
│  57 commands, 18 agents, 5 hooks                         │
│                                                          │
│  Core: new-project → discuss → plan → execute → verify   │
│  Wave-based parallel execution with fresh 200K contexts   │
│  .planning/ state directory                              │
│                                                          │
│  STATUS: Available but commands not invoked in V4.        │
│  Agent worked directly instead of using GSD workflow.     │
└──────────────────────────────────────────────────────────┘
```

## Weakness-to-Research Map (14 Identified Weaknesses)

### W1: Atomic Skill Never Invoked (V3 + V4)
- **Benchmark evidence**: A-hook created .tasks/TRACKER.md manually, did NOT invoke /atomic. V3: "mandatory, not optional" mandate ignored.
- **Architect score impact**: Investigation Depth 18/20 (good without Atomic) but Compounding Bug Recognition 17/20 (would be higher with structured decomposition)
- **Research**: OpenDev arXiv:2603.05344 — "Layer 5 Lifecycle Hooks are the only reliable enforcement." METR — "frontier models increasingly engage in specification gaming rather than following instructions."
- **Root cause**: Subagent environments don't process Claude Code hooks. Even in real sessions, prompt mandates are treated as advisory.

### W2: Monolithic Commits (8/20 — worst metric)
- **Benchmark evidence**: All 3 configs produced 1 commit. 14 files, 1,281 insertions in v4 pipeline.
- **Research**: GSD framework docs — "Every task commits immediately with phase-aware messages, enabling precise git bisect." OpenDev — "surgical atomic commits per task."
- **Root cause**: No enforcement mechanism. CLAUDE.md says "one atomic commit per task" but nothing prevents bundling.

### W3: No RED-First Testing (0/3 configs)
- **Benchmark evidence**: D-8 Test-first: Fail for all 3 configs. Tests written alongside or after implementation.
- **Research**: test-architect skill — "Tests must fail before implementation (RED first). A test that has never been red has never proven it can detect a fault."
- **Root cause**: Writing a failing test then fixing it requires 2 edit cycles per fix. The agent optimizes for efficiency by writing fix + test together.

### W4: No Fairness Audits (0/3 configs)
- **Benchmark evidence**: D-9 Fairness audit: Fail for all configs. No evidence of "could tests pass while feature broken?"
- **Research**: CLAUDE.md v4 — "Fairness audit after implementation: ask how could these tests pass while the feature is broken?" FeatureBench — "43% pass rate vs 11% resolved" = tests passing doesn't mean feature works.
- **Root cause**: Fairness auditing is an abstract reasoning step with no concrete output artifact. Easy to skip.

### W5: Compaction Tombstone Bug Universally Missed
- **Benchmark evidence**: A-3 and B-3: Fail for all 3 configs. `entry.timestamp` vs `entry.deleted_at`.
- **Research**: FeatureBench — "AssertionError (logic gaps) is most common after dependency resolution." CodeGlance — "Unseen functions (novel reasoning) 6x harder than familiar APIs."
- **Root cause**: The bug is a "wrong field" semantic error. Code runs, tests pass, behavior is subtly wrong in a specific scenario. No existing test exposes it.

### W6: Integration Testing Gap (12/20)
- **Benchmark evidence**: No HTTP integration tests, no cross-language tests, no replication integration tests.
- **Research**: FeatureBench — "Cross-file dependency resolution is #1 failure mode." DependEval — "LLMs lack prior knowledge for hierarchical structure organization." CrossCodeEval EM 10.8.
- **Root cause**: Integration tests require running the server and making HTTP requests. More complex to set up than unit tests.

### W7: Negative Testing Weak (13/20)
- **Benchmark evidence**: No tests for invalid inputs, empty batches, NaN, overflow, concurrent races.
- **Research**: EvalPlus — "80x more test cases; performance drops 19-29%." SWE-bench+ — "47.93% of resolved issues were incorrect due to weak tests."
- **Root cause**: Negative tests require adversarial thinking about edge cases. The agent focuses on happy path.

### W8: Feature Completeness Gap — FEAT-4 Not Wired
- **Benchmark evidence**: v4 pipeline implemented `can_serve_reads()` as library-only, never called from routes.
- **Research**: ProjDevBench — "agents build functional frameworks but omit critical features." E2EDevBench — "25% gap between test pass rates and requirement fulfillment."
- **Root cause**: Agent implemented the abstraction but didn't trace through to the integration point.

### W9: Batch Atomicity Illusory
- **Benchmark evidence**: All 3 configs' `batch_put` writes individual WAL entries. Partial failure = partial write.
- **Research**: ProjDevBench — "specification misalignment: models omit critical features despite building functional frameworks."
- **Root cause**: True atomicity requires WAL format changes (batch entry type). Agent took the simpler path.

### W10: Error Swallowing (`let _ =`)
- **Benchmark evidence**: All 3 configs: `let _ = state.replication.commit_entry(entry).await;` in 3-4 places.
- **Research**: "Survey of Bugs in AI-Generated Code" — Reliability bugs from swallowed errors are category 2.
- **Root cause**: Replication errors are non-critical for request handling. Agent treats them as fire-and-forget.

### W11: TTL Logic Duplicated in 3 Places
- **Benchmark evidence**: TTL expiry checked in engine get(), memtable scan(), and compaction.
- **Research**: DRY principle. OpenDev — "separation of concerns" is a core design principle.
- **Root cause**: Agent added TTL filtering at each call site instead of a single `Entry::is_expired()` method.

### W12: Context Rot in Long Sessions
- **Benchmark evidence**: v4 pipeline used 137,926 tokens (40% more than GSD). Instructions in CLAUDE.md were increasingly ignored as context grew.
- **Research**: Anthropic Context Engineering — "n² attention degradation." Context Rot research — "11/12 models drop below 50% at 32k tokens." LongCLI-Bench — "majority of tasks stall at <30% completion."
- **Root cause**: CLAUDE.md v4 is 392 lines. Loaded upfront, it consumes ~2% of context. As context grows, its instructions decay.

### W13: Verification Evidence Weak (10/20)
- **Benchmark evidence**: TRACKER.md claims test counts but provides no command output.
- **Research**: CLAUDE.md v4 — "Never claim 'all tests pass' without running the equivalent." ProjDevBench dual evaluation.
- **Root cause**: Agent writes about verification rather than pasting actual command output.

### W14: GSD Workflows Not Used
- **Benchmark evidence**: No .planning/ directory created. No GSD commands invoked.
- **Research**: GSD — "fresh 200K per executor prevents context rot." Wave-based execution parallelizes independent tasks.
- **Root cause**: Agent determined it could handle the work directly. GSD overhead seemed unnecessary for single-session work.

---

## Improvement Plan: V5 Pipeline Design

### Improvement 1: Multi-Hook Enforcement System
**Addresses**: W1 (Atomic not invoked), W2 (monolithic commits), W3 (no RED-first), W13 (no verification)
**Research basis**: OpenDev arXiv:2603.05344 Layer 5; Claude Code hooks docs (24 events); METR reward hacking

**Design**: 4 hooks working in concert:

```
Hook 1: atomic-gate.js (PreToolUse on Write|Edit) [EXISTS]
  Gate: <=1 file AND <=10 lines → pass, else check .tasks/
  IMPROVE: Also inject additionalContext reminding about RED-first

Hook 2: commit-enforcer.js (PreToolUse on Bash matching "git commit")
  Gate: Count files in staging area. If >5 files → deny with
  "Split into per-task commits. Use TASK-NNN prefix."
  Research: GSD "every task commits immediately"

Hook 3: verification-gate.js (UserPromptSubmit)
  Inject context: "Before claiming completion, run:
  cargo test && cargo clippy && tsc --noEmit
  Paste actual output, not summaries."
  Research: ProjDevBench dual evaluation

Hook 4: test-first-reminder.js (PostToolUse on Edit matching test files)
  When a test file is edited, inject: "Verify this test FAILS
  before implementing the fix. RED-first is mandatory."
  Research: test-architect fairness audit
```

**Expected impact**: +30-40 points on Process category (46→76-86/100)

### Improvement 2: CLAUDE.md v5 — Restructured for Context Efficiency
**Addresses**: W12 (context rot), W4 (no fairness audits), W10 (error swallowing)
**Research basis**: Anthropic Context Engineering — "smallest set of high-signal tokens"; Claude Code docs — "150-200 instruction limit"; OpenDev — "prompt caching: static first, variable last"

**Design changes**:

```
CLAUDE.md v5 structure (target: <200 lines, down from 392):

1. HARD RULES (first, cached, never compacted):
   - Evidence-grounding: cite file:line (3 lines)
   - RED-first: test must fail before fix (2 lines)
   - One commit per task (2 lines)
   - Never `let _ =` — log or handle errors (2 lines)
   - Never duplicate logic — extract to methods (2 lines)

2. GATES (executable checkpoints):
   - Pre-coding: cargo check && cargo clippy
   - Post-fix: fairness audit question (1 line template)
   - Post-task: evidence pack template (5 lines)
   - Pre-completion: verification command output

3. STACK STANDARDS (Rust + TypeScript) — condensed to 10 lines
   - Rust: Result over panic, no blind clone, check borrows
   - TS: no any, strict null, check type alignment

4. TOOL AWARENESS — condensed to 5 lines
   - LSP for Rust/TS, CLI before MCP, skills for deep work

5. RELATIONSHIP TABLE — keep the 18-row table (essential)
```

**Key principle**: Move behavioral rules to hooks (enforceable) and keep CLAUDE.md for quality heuristics only (advisory). The current 392-line CLAUDE.md tries to do both — hooks should handle enforcement, CLAUDE.md should handle judgment.

**Expected impact**: +15 on Code Quality (79→94), +10 on Process (70→80)

### Improvement 3: Structured Pre-Investigation Protocol
**Addresses**: W5 (compaction bug missed), W8 (FEAT-4 not wired), W6 (integration gap)
**Research basis**: FeatureBench — "NameError from cross-file dependency is #1 failure"; DependEval; Atomic Phase 2 "Codebase Grasp Gate"

**Design**: Before ANY code modification, require a written "Understanding Map" that traces:

```markdown
## Understanding Map: [task name]

### Data Flow Trace
[key] → put() → WAL.append() → memtable.insert()
       → get() → memtable.get_latest() → filter tombstones → check TTL
       → scan() → memtable.scan() → filter range → filter tombstones
       → compact() → check retention → check TTL

### Invariants I Must Preserve
1. get_latest(key) returns newest non-tombstone, non-expired entry
2. scan() and TimeRange::contains() use same boundary semantics
3. WAL checksum covers ALL fields that affect correctness
4. Replication catch-up uses current term, not original
5. Timestamps are seconds (Rust) ↔ seconds (TypeScript)

### What Could Go Wrong
- [ ] Checked: compaction.rs retention uses correct timestamp field
- [ ] Checked: all query layers agree on inclusive/exclusive bounds
- [ ] Checked: new feature doesn't bypass existing invariants
```

**Enforce via**: Atomic Phase 2 (Codebase Grasp Gate) already requires this. The improvement is making it a HOOK requirement — block Write|Edit until an understanding map exists in `.tasks/`.

**Expected impact**: +15 on Bug Fix Correctness (18→20), +10 on Edge Cases (14→18)

### Improvement 4: Negative Test Generation Protocol
**Addresses**: W7 (negative testing weak), W9 (batch atomicity), W5 (semantic bugs)
**Research basis**: EvalPlus — "80x more tests, 19-29% performance drop"; Meta mutation-guided test generation (FSE 2025); test-architect "fairness audit"

**Design**: After each bug fix or feature, generate 3 categories of tests:

```
Category 1: Happy Path (what we already do)
Category 2: Adversarial Inputs
  - Empty/null/zero values
  - Overflow (u64::MAX, f64::MAX, very long keys)
  - Concurrent access (spawn 10 tasks, race conditions)
  - Boundary values (exactly at TTL expiry, exactly at range end)
Category 3: Semantic Correctness
  - "Wrong field" tests: verify the RIGHT field is used
    e.g., "delete old entry, compact, verify tombstone survives"
  - "Integration mismatch" tests: verify layers agree
    e.g., "query via API, compare to direct engine query"
  - "Cross-language round-trip" tests
    e.g., "write via Rust, read via TypeScript SDK, compare"
```

**Enforce via**: test-architect skill invocation (already in CLAUDE.md v4 but never invoked). Add to post-implementation hook: inject "Run fairness audit: name a scenario where tests pass but feature is broken."

**Expected impact**: +14 on Negative Testing (13→18), +8 on Integration Testing (12→16)

### Improvement 5: GSD Integration via Auto-Routing
**Addresses**: W14 (GSD not used), W2 (monolithic commits), W12 (context rot)
**Research basis**: GSD framework — "fresh 200K per executor prevents context rot"; Anthropic Context Engineering — "sub-agent architectures return condensed summaries"; METR — time horizon scales with proper context management

**Design**: Instead of relying on the agent to voluntarily invoke GSD, auto-route through it:

```
UserPromptSubmit hook: Classify input complexity
  IF complex (>3 tasks OR >5 files):
    Inject: "This is complex work. Use /gsd:new-project --auto
    to set up proper phase-based execution with fresh executor contexts."
  IF medium (2-3 tasks):
    Inject: "Use /gsd:quick --full for structured execution."
  IF trivial:
    Inject: "Use /gsd:fast for this one-liner."
```

**Key insight from research**: GSD's value is NOT its documentation overhead — it's the **fresh 200K context per executor**. This prevents the context rot that caused instructions to be ignored as the v4 pipeline's context grew to 137K tokens.

**Expected impact**: +8 on Process (70→78), prevents context rot degradation

### Improvement 6: Cross-Language Contract Verification Hook
**Addresses**: W6 (integration testing), W8 (FEAT-4 not wired)
**Research basis**: FeatureBench cross-file dependency; Aider Polyglot multi-language difficulty; CrossPL FFI evaluation (19.5%)

**Design**: PostToolUse hook on Edit matching `types.ts|protocol.json|core/src/lib.rs`:

```
When shared type files are modified, inject:
"You modified a cross-language type. Verify:
1. Rust serde serialization matches TypeScript parsing
2. All API endpoints use the updated types
3. SDK methods handle the new type correctly
4. Run both cargo test AND npm test"
```

**Expected impact**: +5 on Cross-Language Contract (17→19)

### Improvement 7: Evidence-Backed Verification Template
**Addresses**: W13 (verification evidence weak)
**Research basis**: ProjDevBench dual evaluation; CLAUDE.md v4 "never claim tests pass without running"

**Design**: Replace the current evidence pack template with a MANDATORY command-output section:

```markdown
## Verification: TASK-NNN

### Commands Run (paste actual output)
$ cargo test 2>&1 | tail -20
[PASTE OUTPUT HERE — not a summary]

$ cargo clippy 2>&1
[PASTE OUTPUT HERE]

$ cd sdk && npx tsc --noEmit 2>&1
[PASTE OUTPUT HERE]

### Fairness Audit
Q: How could tests pass while this feature is broken?
A: [specific scenario]
Action: [test added or "no plausible scenario"]
```

**Enforce via**: PostToolUse hook on Bash matching "git commit" — inject "Include verification output in evidence pack before committing."

**Expected impact**: +8 on Verification Evidence (10→18)

---

## Expected V5 Scores (Projected)

| Category | V4 Actual | V5 Projected | Delta | Source of Improvement |
|----------|----------:|-----------:|------:|----------------------|
| Correctness | 81 | 90 | +9 | Understanding maps (W5), integration tests (W6) |
| Code Quality | 79 | 88 | +9 | CLAUDE.md v5 (W10, W11), no `any` hook |
| Architecture | 80 | 85 | +5 | Module boundary enforcement, DRY |
| Testing | 77 | 88 | +11 | Negative test protocol (W7), integration (W6) |
| Process | 70 | 88 | +18 | 4-hook system (W1-W3), verification (W13) |
| **TOTAL** | **387** | **439** | **+52** | — |

**Projected improvement**: 387/500 → 439/500 (77.4% → 87.8%)

---

## Implementation Priority (by expected point gain)

| Priority | Improvement | Points | Effort | Files to Change |
|----------|------------|-------:|--------|-----------------|
| **P0** | Multi-hook enforcement (4 hooks) | +30 | Medium | `.claude/hooks/` (4 new files), `settings.json` |
| **P1** | CLAUDE.md v5 restructure | +25 | Medium | `CLAUDE-v5-enhanced.md` (new, <200 lines) |
| **P2** | Negative test generation protocol | +14 | Low | Atomic `references/phases.md` Phase 7 update |
| **P3** | Understanding map requirement | +15 | Low | Atomic `references/task-template.md` update |
| **P4** | GSD auto-routing | +8 | Low | `UserPromptSubmit` hook addition |
| **P5** | Cross-language verification hook | +5 | Low | `PostToolUse` hook addition |
| **P6** | Evidence verification template | +8 | Low | CLAUDE.md v5 template section |

---

## Research Citations Index

| Citation | Used In | Key Finding |
|----------|---------|-------------|
| OpenDev arXiv:2603.05344 | W1, W2, Imp1 | Lifecycle hooks are only reliable enforcement; 88% prompt cache savings |
| Anthropic Context Engineering | W12, Imp2, Imp5 | n² attention degradation; Write/Select/Compress/Isolate |
| FeatureBench ICLR 2026 | W5, W6, Imp3 | Cross-file dependency #1 failure; 790 LOC, 15.7 files avg |
| METR HCAST | W1, Imp5 | Frontier models game specifications; time horizon logistic curve |
| EvalPlus/SWE-bench+ | W7, Imp4 | 80x more tests → 19-29% drop; 47.93% were false passes |
| test-architect skill | W3, W4, Imp4 | RED-first; fairness audit "how could tests pass while broken?" |
| GSD Framework | W2, W14, Imp5 | Fresh 200K per executor; wave-based parallel; atomic commits |
| DependEval ACL 2025 | W5, Imp3 | Cross-file dependency resolution is #1 LLM failure mode |
| ProjDevBench arXiv:2602.01655 | W8, W9, Imp7 | 27% acceptance; spec misalignment; dual evaluation |
| CodeGlance arXiv:2602.13962 | W5 | Unseen reasoning patterns 6x harder |
| Claude Code Hooks Docs | Imp1-7 | 24 events; PreToolUse deny/allow; UserPromptSubmit inject |
| Meta Mutation Testing FSE 2025 | Imp4 | Mutation-guided test generation; 73% engineer acceptance |
| RustAssistant ICSE 2025 | W5 | <30% fix rate on lifetime errors; multi-location fixes |
| Context Rot (Chroma Research) | W12 | 11/12 models drop below 50% at 32k tokens |
| LongCLI-Bench arXiv:2602.14337 | W12 | Multiplicative failure: P=p^N; failures front-loaded |

