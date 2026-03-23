#!/usr/bin/env node
// V5.1 Subagent Quality Injection + Operator Folder System
// Hook: SubagentStart
//
// 1. Creates isolated operator folder at .tasks/.operators/{agent_id}/
//    with V1-style context management files (TODO, CONTEXT, DECISIONS, etc.)
// 2. Injects quality rules into subagent context
// 3. Cleans up orphaned operator folders from crashed subagents
//
// Each subagent manages its own folder like a V1 operator manages Docs/Phases/NN/.
// No filename conflicts — agent_id is a unique UUID.

const fs = require('fs');
const path = require('path');

const QUALITY_RULES = `
## Quality Policy (injected by SubagentStart hook)

### Hard Rules
1. **Evidence-grounding**: Cite file:line for every claim. Read files before modifying.
2. **RED-first testing**: Write a FAILING test before implementing the fix.
3. **One commit per task**: Use TASK-NNN: prefix. Never bundle multiple tasks.
4. **No error swallowing**: Never \`let _ =\` to discard Results. Log or handle.
5. **No logic duplication**: Same check in 3+ places → extract to a method.
6. **Fairness audit**: After implementing, ask "How could tests pass while feature is broken?"

### Pre-Coding Gates
Before ANY code modification:
1. Run cargo check && cargo clippy (Rust) or tsc --noEmit (TypeScript)
2. Write understanding to your operator CONTEXT.md BEFORE coding
3. Invoke a domain skill: bugs→root-cause-analysis, tests→test-architect

### Negative Test Protocol
After each fix/feature, write 3 categories:
1. Happy path 2. Adversarial inputs (empty/null/overflow/boundary) 3. Semantic correctness (right field, layers agree, cross-language round-trip)

### Verification
Before claiming done, run and paste actual output:
cargo test, cargo clippy, tsc --noEmit — into your operator VERIFICATION.md

### Standards
- Rust: clippy clean, no blind .clone(), Result over panic, use rust-analyzer LSP
- TypeScript: no \`any\`, strict null, use LSP find-references
- Cross-language: verify Rust serialization matches TypeScript parsing
`.trim();

function createSkeletonFiles(operatorDir, agentId, agentType) {
  const files = {
    'ROADMAP.md': `# Operator Roadmap: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Assigned Task
[Read from .tasks/TASK-NNN/context.md — fill in after reading assignment]

## Status
in_progress

## Plan
1. [ ] Read context.md for architecture evidence
2. [ ] Write CONTEXT.md (understanding map)
3. [ ] Pre-coding gates (cargo check, skill loading)
4. [ ] Implement (RED-first)
5. [ ] Write VERIFICATION.md (actual output)
6. [ ] Fairness audit
7. [ ] Commit: TASK-NNN: [description]
`,
    'TODO.md': `# TODO: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Checklist
- [ ] Read assigned task context.md
- [ ] Write understanding in CONTEXT.md
- [ ] Run pre-coding gates
- [ ] Write failing test (RED)
- [ ] Implement fix/feature (GREEN)
- [ ] Write negative tests (adversarial + semantic)
- [ ] Run cargo test + cargo clippy + tsc --noEmit
- [ ] Paste actual output in VERIFICATION.md
- [ ] Fairness audit: "how could tests pass while broken?"
- [ ] Log decisions in DECISIONS.md
- [ ] Commit with TASK-NNN prefix
`,
    'CONTEXT.md': `# Context: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Data Flow Trace
[Trace how data moves through the system for your task]

## Invariants to Preserve
1. [invariant — file:line]

## What Could Go Wrong
- [ ] [concern to check]

## Conventions Discovered
- [patterns from reading the code]
`,
    'SOURCES.md': `# Sources: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Files Read
- [ ] [file — why it matters]

## Tests Consulted
- [ ] [test file — what it covers]
`,
    'DECISIONS.md': `# Decisions: ${agentType || 'executor'} (${agentId.slice(0, 8)})

[Log choices made during implementation]
`,
    'VERIFICATION.md': `# Verification: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Commands Run (paste actual output)

## Tests
| Test | Before | After | Notes |
|------|--------|-------|-------|

## Fairness Audit
Q: How could tests pass while feature is broken?
A:
`,
    'HANDOFF.md': `# Handoff: ${agentType || 'executor'} (${agentId.slice(0, 8)})

## Status
in_progress

## Next Action
[fill if interrupted]
`,
  };

  for (const [filename, content] of Object.entries(files)) {
    const filePath = path.join(operatorDir, filename);
    if (!fs.existsSync(filePath)) {
      fs.writeFileSync(filePath, content);
    }
  }
}

let input = '';
const stdinTimeout = setTimeout(() => process.exit(0), 4000);
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  clearTimeout(stdinTimeout);
  try {
    const data = JSON.parse(input);
    const cwd = data.cwd || process.cwd();
    const agentId = data.agent_id || `anon_${Date.now()}`;
    const agentType = data.agent_type || 'general';

    // Create operator folder
    const operatorsDir = path.join(cwd, '.tasks', '.operators');
    const operatorDir = path.join(operatorsDir, agentId);

    // Clean up orphaned operator folders (>1 hour old)
    if (fs.existsSync(operatorsDir)) {
      try {
        const entries = fs.readdirSync(operatorsDir);
        const oneHourAgo = Date.now() - 3600000;
        for (const entry of entries) {
          const entryPath = path.join(operatorsDir, entry);
          try {
            const stat = fs.statSync(entryPath);
            if (stat.isDirectory() && stat.mtimeMs < oneHourAgo) {
              fs.rmSync(entryPath, { recursive: true, force: true });
            }
          } catch {}
        }
      } catch {}
    }

    // Create this operator's folder with skeleton files
    fs.mkdirSync(operatorDir, { recursive: true });
    createSkeletonFiles(operatorDir, agentId, agentType);

    // Build context injection
    let context = QUALITY_RULES;

    context += `\n\n## Your Operator Workspace\nYour isolated operator folder is at: ${operatorDir}\n`;
    context += 'Write your understanding to CONTEXT.md BEFORE coding.\n';
    context += 'Log decisions in DECISIONS.md during implementation.\n';
    context += 'Paste actual command output in VERIFICATION.md after implementation.\n';
    context += 'Update TODO.md checklist as you progress.\n';

    // Check for CLAUDE.md
    const claudeMdPath = path.join(cwd, 'CLAUDE.md');
    if (fs.existsSync(claudeMdPath)) {
      context += `\nIMPORTANT: Read ${claudeMdPath} for full project quality policy.`;
    }

    // Check for .tasks/
    const tasksDir = path.join(cwd, '.tasks');
    if (fs.existsSync(tasksDir)) {
      context += '\n.tasks/ directory exists. Read the relevant TASK-NNN/context.md for architecture context.';
    }

    const output = {
      hookSpecificOutput: {
        hookEventName: 'SubagentStart',
        additionalContext: context,
      },
    };

    process.stdout.write(JSON.stringify(output));
  } catch {
    process.exit(0);
  }
});
