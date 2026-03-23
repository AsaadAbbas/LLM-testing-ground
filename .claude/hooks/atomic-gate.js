#!/usr/bin/env node
// V5 Atomic Gate Hook — PreToolUse on Write|Edit
// Enforces Atomic decomposition for non-trivial work.
//
// Gate logic:
//   <=1 file AND <=10 lines → approve (trivial)
//   .tasks/ exists in cwd → approve (Atomic already ran)
//   Otherwise → deny + inject RED-first reminder
//
// Exclusions: .planning/*, .tasks/*, CLAUDE.md, root *.md

const fs = require('fs');
const path = require('path');
const os = require('os');

let input = '';
const stdinTimeout = setTimeout(() => process.exit(0), 4000);
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  clearTimeout(stdinTimeout);
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name;
    const cwd = data.cwd || process.cwd();
    const sessionId = data.session_id || 'unknown';

    if (toolName !== 'Write' && toolName !== 'Edit') {
      process.exit(0);
    }

    const filePath = data.tool_input?.file_path || '';

    // Exclusions: state/doc files
    if (filePath.includes('.planning/') || filePath.includes('.planning\\') ||
        filePath.includes('.tasks/') || filePath.includes('.tasks\\') ||
        filePath.includes('.claude/') || filePath.includes('.claude\\') ||
        path.basename(filePath) === 'CLAUDE.md' ||
        (path.dirname(filePath) === cwd && filePath.endsWith('.md'))) {
      process.exit(0);
    }

    // Load session state
    const statePath = path.join(os.tmpdir(), `claude-atomic-${sessionId}.json`);
    let state = { files: [], totalLines: 0 };
    if (fs.existsSync(statePath)) {
      try { state = JSON.parse(fs.readFileSync(statePath, 'utf8')); } catch {}
    }

    if (!state.files.includes(filePath)) {
      state.files.push(filePath);
    }

    const content = data.tool_input?.content || data.tool_input?.new_string || '';
    state.totalLines += content.split('\n').length;
    fs.writeFileSync(statePath, JSON.stringify(state));

    // Gate 1: Trivial work
    if (state.files.length <= 1 && state.totalLines <= 10) {
      process.exit(0);
    }

    // Gate 2: Atomic already ran
    if (fs.existsSync(path.join(cwd, '.tasks'))) {
      // Approve, but inject RED-first reminder
      const output = {
        hookSpecificOutput: {
          hookEventName: 'PreToolUse',
          additionalContext: 'Reminder: write a FAILING test first (RED), then implement the fix (GREEN). Verify the test fails before implementing.',
        },
      };
      process.stdout.write(JSON.stringify(output));
      process.exit(0);
    }

    // Gate 3: Deny
    const output = {
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        permissionDecision: 'deny',
        permissionDecisionReason:
          `Non-trivial work (${state.files.length} files, ${state.totalLines} lines). ` +
          'Run /atomic first to decompose into .tasks/ before code changes.',
        additionalContext:
          'The Atomic skill (/atomic) decomposes work into per-task context folders. ' +
          'Once .tasks/ exists, all edits are approved. For trivial work, use /gsd:fast.',
      },
    };
    process.stdout.write(JSON.stringify(output));
  } catch {
    process.exit(0);
  }
});
