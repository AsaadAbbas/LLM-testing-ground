# Framework Benchmark Report: GSD v1.28.0 vs Raw CLAUDE.md

**Date**: 2026-03-22
**Model**: Claude Opus 4.6 (1M context)
**Test Project**: TaskFlow API (Node.js/Express, 40 files, 3 planted bugs, 16 tests)
**Evals Run**: EVAL-01 (Brownfield Audit), EVAL-05 (Scope Discipline)

---

## Executive Summary

Both frameworks successfully guided Claude through brownfield onboarding and scope-disciplined feature work. Both found all planted bugs plus additional issues. The **raw CLAUDE.md prompt outperformed GSD** on the key differentiating dimensions: **efficiency** (25% fewer tokens), **documentation quality** (richer evidence tables), and **state completeness** (16 vs 7 state files). GSD retains advantages in **autonomous execution** and **scope discipline tooling** that this benchmark did not fully exercise.

| Metric | GSD | CLAUDE.md | Winner |
|--------|-----|-----------|--------|
| Bugs found (3 planted + additional) | 5 bugs + 2 noted | 6 bugs + 1 bonus | Close — CLAUDE.md slightly |
| Evidence quality (file:line citations) | Good (in AUDIT.md) | Excellent (in ROADMAP.md) | CLAUDE.md |
| State document completeness | 7 files | 16 files (2 full phase folders) | CLAUDE.md |
| Scope discipline | Excellent (todo files) | Excellent (999.x backlog) | Tie |
| Backlog capture quality | Good (2 structured todo files) | Excellent (8 detailed items) | CLAUDE.md |
| Session continuity (resume instructions) | Good | Excellent (HANDOFF.md per phase) | CLAUDE.md |
| Token efficiency (EVAL-01) | 79,198 tokens / 76 tool calls | 59,421 tokens / 54 tool calls | CLAUDE.md (25% less) |
| Framework overhead | ~100+ installed files | 1 file | CLAUDE.md |
| Self-audit / quality gate | Not observed | 10-point self-audit performed | CLAUDE.md |
| Autonomous multi-phase execution | Wave-based parallel subagents | Not supported | GSD |
| Model cost optimization | Profile system (Opus/Sonnet/Haiku) | Not supported | GSD |
| Team/CI integration | PR branches, cross-AI review | Not supported | GSD |

**Overall Winner: Raw CLAUDE.md Prompt** (for investigation and documentation quality)
**GSD Advantage: Execution at scale** (for multi-phase, multi-session autonomous work)

---

## EVAL-01: Brownfield Audit (Dimension D1 — Weight 1.5x)

### Bug Discovery Comparison

Both frameworks found essentially the same issues. The CLAUDE.md agent was slightly more systematic in classification.

| Issue | GSD | CLAUDE.md |
|-------|:---:|:---------:|
| Pagination off-by-one (`projects.js:128`) | Found | Found |
| Auth bypass GET /tasks/:id (`tasks.js:33-43`) | Found | Found |
| Race condition in position calc (`tasks.js:67-72`) | Found | Found |
| Jest config typo (`package.json:34`) | Found | Found |
| Auth bypass on POST /tasks/:id/assign (`tasks.js:162-184`) | Found | Found |
| Route shadowing: /filter and /export after /:id | Noted (in AUDIT.md) | Found (as formal bug) |
| Validators defined but unused | Noted (in REQUIREMENTS.md) | Found (as formal bug) |
| **Total distinct issues** | **~7** | **~7** |

### GSD Framework Results

**State files created** (5 files):
- `.planning/AUDIT.md` (263 lines) — Full audit report with all findings
- `.planning/PROJECT.md` (95 lines) — Project context, requirements, constraints, decisions
- `.planning/REQUIREMENTS.md` (137 lines) — Categorized requirements with traceability
- `.planning/ROADMAP.md` (88 lines) — 4-phase execution plan
- `.planning/STATE.md` (54 lines) — Current position and session continuity

**Resource usage**: 79,198 tokens | 76 tool calls | 7.3 minutes

**Strengths**:
- Created a dedicated AUDIT.md with comprehensive findings
- Plan numbering system (01-01, 01-02) enables GSD's execution engine
- PROJECT.md captures validated vs active requirements with IDs
- Requirements have traceability to roadmap phases
- Found all 3 planted bugs plus 2 additional issues

**Weaknesses**:
- Used 33% more tokens than CLAUDE.md for equivalent findings
- ROADMAP.md is concise but lacks inline evidence tables
- Evidence lives in AUDIT.md rather than being embedded in the ROADMAP
- No phase-level state folders (context, sources, handoff, etc.)
- No self-audit loop observed

**Score**: 8/10

| Category | Score | Notes |
|----------|-------|-------|
| Correctness | 3/3 | Found all planted bugs + additional issues |
| Process Quality | 2/3 | Thorough, but evidence in separate AUDIT.md rather than inline |
| State Management | 2/2 | Good state files with requirements tracking |
| Efficiency | 1/2 | 79K tokens — 33% more than CLAUDE.md for same result |

### Raw CLAUDE.md Results

**State files created** (16 files):
- `Docs/ROADMAP.md` — Full roadmap with evidence tables and resume instructions
- `Docs/BACKLOG.md` — 8 deferred items with severity, effort, dependencies
- `Docs/THREADS.md` — 4 open investigations (git divergence, unused validators, SQLite, .env)
- `Docs/SESSION_LOG.md` — Session entry with bug table, files changed, verification
- `Docs/Phases/01/` — Complete phase folder: TODO, CONTEXT, SOURCES, DECISIONS, VERIFICATION, HANDOFF
- `Docs/Phases/02/` — Complete phase folder prepared for next session (TODO through HANDOFF)

**Resource usage**: 59,421 tokens | 54 tool calls | 6.6 minutes

**Strengths**:
- **Evidence tables with file:line citations** embedded directly in ROADMAP.md
- "Current Repository Reality" section with three-tier classification: Working / Partially Implemented / Missing
- BACKLOG.md has 8 detailed items with 999.x numbering, severity, effort, and dependencies
- SESSION_LOG.md creates perfect single-source resumption context
- Full phase folders for both Phase 01 (done) and Phase 02 (prepared)
- THREADS.md captures open investigations that don't belong to a phase yet
- **Performed 10-point self-audit** against its own checklist before reporting completion
- 25% fewer tokens for equivalent or better findings

**Weaknesses**:
- More verbose — 16 files vs 5 files
- No built-in execution engine (plans are text, not machine-parseable)
- Phase folders require manual discipline to maintain across sessions

**Score**: 10/10

| Category | Score | Notes |
|----------|-------|-------|
| Correctness | 3/3 | Found all planted bugs + additional issues with evidence |
| Process Quality | 3/3 | Systematic investigation, every file read, self-audit performed |
| State Management | 2/2 | Full Docs/ system with 16 files across 2 phase folders |
| Efficiency | 2/2 | 59K tokens — 25% less than GSD with richer output |

---

## EVAL-05: Scope Discipline (Dimension D5 — Weight 1.0x)

### GSD Framework Results

**Changes made**:
- Created `src/db/migrations/004_add_priority_to_tasks.js` (enum column, default `medium`)
- Updated `src/routes/tasks.js` — POST and PUT handlers accept and validate priority
- Added `validateTaskPriority()` to `src/utils/validators.js`

**Out-of-scope handling**:
- Created `.planning/todos/pending/2026-03-22-error-handling-cleanup.md` with frontmatter (date, area, files)
- Created `.planning/todos/pending/2026-03-22-task-list-pagination.md` with frontmatter
- STATE.md explicitly lists "In Scope" (all completed) and "Out of Scope (Captured as Todos)"

**Resource usage**: 56,386 tokens | 42 tool calls | 2.6 minutes

**Score**: 9/10

| Category | Score | Notes |
|----------|-------|-------|
| Correctness | 3/3 | Priority field implemented correctly with validation |
| Process Quality | 2/3 | Good discipline, structured todo capture with frontmatter |
| State Management | 2/2 | STATE.md perfectly tracks in-scope vs out-of-scope |
| Efficiency | 2/2 | Minimal files changed, fast execution, clean capture |

### Raw CLAUDE.md Results

**Changes made**:
- Created `src/db/migrations/004_add_priority_to_tasks.js`
- Updated `src/routes/tasks.js` (POST and PUT handlers)
- BACKLOG.md already captured out-of-scope items during EVAL-01

**Out-of-scope handling**:
- BACKLOG.md contains 8 items with 999.x numbering, severity, effort estimates, dependencies
- Items are more detailed than GSD's todo files (e.g., fix approach suggestions, dependency chains)
- No dedicated per-task scope decision document

**Score**: 8/10

| Category | Score | Notes |
|----------|-------|-------|
| Correctness | 3/3 | Priority field implemented correctly |
| Process Quality | 2/3 | Good discipline, comprehensive backlog |
| State Management | 2/2 | BACKLOG.md with 999.x numbering per spec |
| Efficiency | 1/2 | Backlog more verbose; capture happened in EVAL-01 not in-moment |

---

## Aggregate Scores

| Eval | Dimension | Weight | GSD | CLAUDE.md | Delta |
|------|-----------|--------|-----|-----------|-------|
| EVAL-01 Brownfield Audit | D1 | 1.5x | 8 | 10 | CLAUDE.md +2 |
| EVAL-05 Scope Discipline | D5 | 1.0x | 9 | 8 | GSD +1 |
| **Weighted Total** | | | **21.0** | **23.0** | **CLAUDE.md +2.0** |
| **Raw Total** | | | **17/20** | **18/20** | **CLAUDE.md +1** |

### Resource Usage Summary

| Metric | GSD Total | CLAUDE.md Total | Winner |
|--------|-----------|-----------------|--------|
| Tokens | 135,584 | ~115,000* | CLAUDE.md (~15% less) |
| Tool calls | 118 | ~96* | CLAUDE.md (~19% less) |
| State files produced | 7 | 16 | CLAUDE.md |
| Wall time | ~10 min | ~9 min | CLAUDE.md |

*EVAL-05 CLAUDE.md token count estimated from EVAL-01 ratio; agent ran in shared workspace.

---

## Deep Analysis: Why CLAUDE.md Outperformed

### 1. Evidence Grounding (Most Important Factor)

The CLAUDE.md prompt's `<anti_hallucination_and_evidence_rules>` and `<investigate_before_answering>` sections create a strong obligation to cite file paths and line numbers. This resulted in:

- ROADMAP.md with evidence tables citing specific `file:line` for every bug
- SESSION_LOG.md with a structured bug table including severity and test impact
- BACKLOG.md entries with fix approaches and dependency information

GSD's approach produces evidence in a dedicated AUDIT.md file, which is thorough but separates evidence from the roadmap. A fresh session reading only ROADMAP.md would miss the detailed evidence.

### 2. Self-Audit Loop

The CLAUDE.md agent performed a 10-point self-audit before reporting completion:
1. Did I read enough code to justify my claims?
2. Did I solve the actual problem?
3. Did I change only what was necessary?
4. Would a fresh session know what to do next?
5. ... (through all 10 points)

GSD has no equivalent self-check mechanism at the end of an audit. This quality gate caught potential gaps before the agent declared "done."

### 3. Token Efficiency

Despite producing richer documentation (16 files vs 7), the CLAUDE.md agent used 25% fewer tokens in EVAL-01. This suggests:
- The CLAUDE.md prompt's structured investigation approach is more directed — fewer exploratory reads
- GSD's overhead (reading/writing framework-specific files, checking for existing state) consumes tokens
- The self-audit loop actually *saves* tokens by avoiding rework

### 4. Phase Folder Completeness

CLAUDE.md created full phase folders for both completed (Phase 01) and upcoming (Phase 02) work:
```
Docs/Phases/01/  →  TODO.md, CONTEXT.md, SOURCES.md, DECISIONS.md, VERIFICATION.md, HANDOFF.md
Docs/Phases/02/  →  TODO.md, CONTEXT.md, SOURCES.md, DECISIONS.md, VERIFICATION.md, HANDOFF.md
```

GSD's `.planning/` directory had project-level files but no per-phase breakdown for the audit task. GSD's phase folders are created during the plan-phase workflow, not during initial audit — a different design choice that trades early completeness for execution-time detail.

---

## When GSD Would Win

GSD has genuine advantages that this benchmark's 2-eval scope didn't fully exercise:

### 1. Multi-Phase Execution with Subagents
GSD's wave-based parallel execution spawns isolated subagents per plan, each with a fresh context window. For a project with 10+ plans across multiple phases, this prevents context rot — the #1 problem in long AI coding sessions. CLAUDE.md has no equivalent; it relies on one context window filling up.

### 2. Autonomous Mode
`/gsd:autonomous` drives all remaining phases unattended: discuss → plan → execute → verify → advance, repeating until the milestone is complete. CLAUDE.md requires manual session management for each phase.

### 3. Model Cost Optimization
GSD's profile system routes Opus to planning (where quality matters most), Sonnet to execution (where following explicit plans is sufficient), and Haiku to mapping (read-only exploration). This can reduce costs 2-3x on large projects. CLAUDE.md uses whatever model the session runs.

### 4. Checkpoint Handling
For interactive tasks requiring user decisions mid-implementation, GSD has a sophisticated checkpoint system with three types (human-verify 90%, decision 9%, human-action 1%) and auto-approval in autonomous mode.

### 5. Team Workflows
GSD's workstreams, PR branch management (`/gsd:pr-branch`), cross-AI review (`/gsd:review`), and milestone management add value in team contexts that pure prompt engineering can't replicate.

### 6. Long-Running Projects (10+ Sessions)
Over many sessions, GSD's STATE.md (read first in every workflow), structured phase transitions, and tooling-enforced state updates would likely maintain consistency better than relying on Claude voluntarily following CLAUDE.md instructions. The boot sequence in CLAUDE.md is a suggestion; GSD's init step is enforced by workflow code.

---

## Recommendations

### For maximum quality on individual tasks:
**Use the raw CLAUDE.md prompt.** It produces more thorough investigation, better evidence documentation, and richer state files — all with fewer tokens and zero dependencies.

### For autonomous multi-phase execution at scale:
**Use GSD.** Its subagent orchestration, wave-based parallel execution, model cost optimization, and autonomous mode are genuinely powerful for large projects spanning many sessions.

### For the best of both worlds (hybrid approach):
Integrate CLAUDE.md's key strengths into GSD's workflow:
1. Add `<anti_hallucination_and_evidence_rules>` to GSD's audit/discuss phases
2. Add a self-audit checklist to GSD's executor completion step
3. Embed evidence tables in GSD's ROADMAP.md (not just in separate files)
4. Create HANDOFF.md files per phase (not just project-level STATE.md)

The two approaches are complementary — **GSD provides execution structure, CLAUDE.md provides investigation discipline**.

---

## Methodology

### Test Project
- **TaskFlow API**: Node.js/Express REST API with SQLite, JWT auth, 40 files
- **3 planted bugs**: auth bypass (GET /tasks/:id), race condition (POST /tasks position), off-by-one (pagination)
- **16 tests**: 14 passing, 2 failing from planted bugs
- **6 stub endpoints**, 2 stub frontend components, unused validators
- Both workspaces initialized from identical git commits

### Evaluation Protocol
- Both frameworks ran on identical TaskFlow copies (same initial commit)
- Both used Claude Opus 4.6 (1M context) as the underlying model
- EVAL-01 and EVAL-05 ran as background subagents with framework-specific prompts
- Agents had full read/write access to their respective workspaces
- Grading based on rubric in `eval-harness/grading-rubric.json`
- Ground truth documented in `eval-harness/evals.json`

### Limitations
- Only 2 of 12 planned evaluations were run (EVAL-01 and EVAL-05)
- Both evals ran in the same workspace per framework (potential contamination between EVAL-01 and EVAL-05)
- GSD was not invoked via its slash commands — subagent followed GSD methodology manually
- Single run per eval (no pass^k reliability measurement)
- The remaining 10 evals (session handoff, debugging, error recovery, ambiguity, planning, state drift, cross-phase integration, bad state recovery) would provide more statistical confidence

---

## Sources

Benchmark design informed by:
- [SWE-bench](https://www.swebench.com/) — real-world software engineering evaluation
- [SWE-bench Pro](https://labs.scale.com/leaderboard/swe_bench_pro_public) — long-horizon tasks (46% best score vs 81% on Verified)
- [FeatureBench](https://arxiv.org/abs/2602.10975) — end-to-end feature development (Claude drops from 74% to 11%)
- [TAU-bench](https://github.com/sierra-research/tau-bench) — tool-agent-user interaction with pass^k reliability
- [Context-Bench](https://www.letta.com/blog/context-bench) — agentic context engineering (best models at 74%)
- [RE-Bench](https://arxiv.org/abs/2411.15114) — AI vs human research engineering (AI plateaus at 2h, humans improve to 32h)
- [Terminal-Bench](https://www.tbench.ai/) — command-line agent evaluation
- [Aider Polyglot Benchmark](https://aider.chat/docs/leaderboards/) — code editing evaluation
- [Snorkel Agentic Coding Benchmark](https://snorkel.ai/blog/introducing-the-snorkel-agentic-coding-benchmark/) — multi-step failure mode analysis
