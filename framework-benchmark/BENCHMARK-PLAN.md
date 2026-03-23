# Framework Benchmark: GSD vs Raw CLAUDE.md

## What We're Comparing

| Dimension | GSD (Get Shit Done) | Raw CLAUDE.md |
|-----------|-------------------|---------------|
| **Type** | Runtime framework with CLI tools, subagents, templates | Pure prompt engineering (~5000 words) |
| **State Management** | .planning/ dir: STATE.md, ROADMAP.md, config.json, phase dirs | Docs/ dir: ROADMAP.md, BACKLOG.md, THREADS.md, SESSION_LOG.md |
| **Execution Model** | Wave-based parallel execution with specialized subagents | 8-step state machine (RECONCILE→...→ADVANCE) |
| **Verification** | Dedicated gsd-verifier subagent, 4-level verification patterns | VERIFICATION.md per phase, self-audit loop |
| **Planning** | discuss→research→plan→execute pipeline with separate agents | Phase folders with TODO.md, CONTEXT.md, SOURCES.md |
| **Tooling** | Node.js CLI (gsd-tools.cjs), 15+ agent types, ~40 commands | None - pure instructions |

## Evaluation Dimensions (10 Categories)

### D1: Brownfield Onboarding
Can the framework accurately assess an existing codebase?
- **Measures**: State accuracy, evidence grounding, correctly identifying what's done vs incomplete

### D2: Multi-Session Continuity
Can a fresh "session" resume from state files alone?
- **Measures**: State file completeness, next-action clarity, no repeated work

### D3: Complex Multi-Step Implementation
Can it decompose and execute a 5+ file feature?
- **Measures**: Code quality, task decomposition, atomic commits, working code

### D4: Debugging & Root Cause Analysis
Can it diagnose a subtle, multi-file bug?
- **Measures**: Diagnosis accuracy, fix correctness, evidence trail

### D5: Scope Discipline
Does it resist scope creep and stay focused?
- **Measures**: Files changed (fewer = better), no unrequested features, backlog capture

### D6: Verification Rigor
Does it actually verify claims with evidence?
- **Measures**: Verification file accuracy, no false "success" claims, test execution

### D7: Error Recovery
How does it handle mid-task failures (failing tests, broken deps)?
- **Measures**: Recovery strategy, state preservation, user communication

### D8: Ambiguous Requirements
Does it clarify or guess when requirements are vague?
- **Measures**: Questions asked, assumption documentation, decision logging

### D9: Planning Quality
Are plans actionable, complete, and correctly ordered?
- **Measures**: Task granularity, dependency awareness, no missing steps

### D10: State File Accuracy
Do the tracking files reflect repository reality?
- **Measures**: State-to-code alignment, no stale claims, correct completion status

## Test Project: "TaskFlow API"

A partially-built task management REST API with:
- **Backend**: Node.js/Express with 12 route handlers (7 working, 5 stub/broken)
- **Database**: SQLite with Knex.js migrations (partially applied)
- **Tests**: Jest test suite (15 tests, 4 failing due to bugs, 3 pending)
- **Frontend**: Minimal React app with 4 components (2 incomplete)
- **Planted bugs**: 3 subtle bugs (race condition, off-by-one, incorrect auth check)
- **Missing features**: User settings endpoint, task filtering, export to CSV
- **~40 files total**

## Eval Tasks (12 Tasks Across 10 Dimensions)

### EVAL-01: Brownfield Audit [D1]
**Prompt**: "You are starting work on this project for the first time. Assess the current state of the repository. What works? What's broken? What's missing? Produce your state tracking documents."
**Assertions**:
- Correctly identifies 7 working routes
- Correctly identifies 5 incomplete routes
- Finds all 3 planted bugs
- Identifies 4 failing tests and their causes
- State documents created and accurate

### EVAL-02: Session Handoff [D2]
**Prompt**: (After EVAL-01 state files exist) "A previous session audited this project and left state files. Continue from where they left off. What's the next action?"
**Assertions**:
- Reads state files before acting
- Identifies correct next action
- Doesn't re-do completed work
- Updates state files with new progress

### EVAL-03: Feature Implementation [D3]
**Prompt**: "Implement the user settings endpoint (GET/PUT /api/users/:id/settings) with the following requirements: users can set theme (light/dark), notification preferences (email/push/none), and timezone. Settings should persist in the database. Add tests."
**Assertions**:
- Migration created and applied
- Route handler works (GET returns settings, PUT updates)
- Input validation present
- Tests written and passing
- Atomic commits with meaningful messages
- State/roadmap updated

### EVAL-04: Multi-File Bug Hunt [D4]
**Prompt**: "Users report that when they create a task with a due date and then fetch their task list, sometimes tasks appear duplicated. The issue is intermittent. Investigate and fix."
**Assertions**:
- Correctly identifies race condition in task creation
- Fix addresses root cause (not just symptoms)
- Doesn't break other functionality
- Documents diagnosis process
- Adds regression test

### EVAL-05: Scope Discipline [D5]
**Prompt**: "Add a 'priority' field (low/medium/high/urgent) to tasks. By the way, while you're in there, the error handling could probably use some cleanup, and it would be nice to have pagination on the task list endpoint too."
**Assertions**:
- Adds priority field correctly
- Does NOT add pagination (out of scope)
- Does NOT refactor error handling (out of scope)
- Captures out-of-scope items in backlog/notes
- Minimal files changed

### EVAL-06: Verification Honesty [D6]
**Prompt**: "Implement the task filtering endpoint (GET /api/tasks?status=done&priority=high&assignee=user1). Verify that it works correctly."
**Assertions**:
- Implementation present
- Verification file exists with actual test results
- No false "all tests pass" without running them
- Edge cases acknowledged (empty results, invalid params)
- Verification distinguishes automated vs manual checks

### EVAL-07: Error Recovery [D7]
**Setup**: Deliberately break a dependency (rename a required module file)
**Prompt**: "Continue implementing the CSV export feature for tasks."
**Assertions**:
- Detects broken dependency before proceeding
- Doesn't silently work around it
- Either fixes or reports the issue
- State reflects the blocker
- Doesn't claim success despite errors

### EVAL-08: Ambiguous Requirements [D8]
**Prompt**: "Add notifications to the app."
**Assertions**:
- Asks clarifying questions (what kind? email? push? in-app? triggers?)
- Documents assumptions if proceeding
- Doesn't over-build without clarification
- Decision log captures the ambiguity resolution

### EVAL-09: Complex Planning [D9]
**Prompt**: "We need to add real-time collaboration: multiple users can view and edit the same task simultaneously, with live updates and conflict resolution. Plan this feature."
**Assertions**:
- Plan identifies WebSocket/SSE requirement
- Correctly sequences: schema changes → backend → frontend → testing
- Identifies risks (conflict resolution, scaling)
- Dependencies between steps are correct
- Realistic scope (doesn't trivialize complexity)

### EVAL-10: State Accuracy Under Drift [D10]
**Setup**: Manually modify 3 files after the framework has created state docs
**Prompt**: "Check if the project state is still accurate and reconcile any drift."
**Assertions**:
- Detects the manual modifications
- Updates state files to reflect current reality
- Doesn't overwrite manual changes without acknowledgment
- Reconciliation documented

### EVAL-11: Cross-Phase Integration [D3, D9]
**Prompt**: "Phase 1 added the priority field. Now implement phase 2: a dashboard endpoint (GET /api/dashboard) that returns task counts grouped by priority and status. This depends on phase 1's schema changes."
**Assertions**:
- Correctly references phase 1 schema
- SQL/query correctly uses priority column
- Handles case where phase 1 might not be complete
- Integration tested

### EVAL-12: Recovery From Bad State [D7, D10]
**Setup**: Corrupt state files (wrong completion status, stale phase info)
**Prompt**: "Resume work on this project."
**Assertions**:
- Detects state corruption
- Reconciles with actual repo state
- Repairs state files before proceeding
- Documents discrepancies found

## Scoring Rubric

Each eval scored 0-10 across these criteria:
- **Correctness** (0-3): Does the code/output actually work?
- **Process Quality** (0-3): Did it follow good engineering process?
- **State Management** (0-2): Are tracking files accurate and useful?
- **Efficiency** (0-2): Token usage, files touched, steps taken

**Total possible**: 120 points (12 evals × 10 points each)

## Execution Plan

1. Create the TaskFlow test project (realistic brownfield codebase)
2. Install GSD locally in workspace-gsd/
3. Add CLAUDE.md to workspace-claudemd/
4. Copy identical TaskFlow project to both workspaces
5. Run each eval as a subagent (with-skill = GSD, baseline = CLAUDE.md)
6. Grade outputs using assertion checking + qualitative review
7. Aggregate into comparison report
