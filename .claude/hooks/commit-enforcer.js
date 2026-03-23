#!/usr/bin/env node
// V5 Commit Enforcer Hook — PreToolUse on Bash
// Advises against monolithic commits by checking staged file count.
//
// Advisory: >8 staged files → inject warning to split commits
// Uses execFileSync (no shell injection risk — fixed args only)

const { execFileSync } = require('child_process');

let input = '';
const stdinTimeout = setTimeout(() => process.exit(0), 4000);
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  clearTimeout(stdinTimeout);
  try {
    const data = JSON.parse(input);

    if (data.tool_name !== 'Bash') {
      process.exit(0);
    }

    const command = data.tool_input?.command || '';

    // Only intercept git commit commands
    if (!command.match(/git\s+commit/)) {
      process.exit(0);
    }

    // Skip amend and merge commits
    if (command.includes('--amend') || command.includes('merge')) {
      process.exit(0);
    }

    const cwd = data.cwd || process.cwd();

    // Count staged files using execFileSync (safe, no shell)
    let stagedCount = 0;
    try {
      const staged = execFileSync('git', ['diff', '--cached', '--name-only'], {
        cwd,
        encoding: 'utf8',
        timeout: 5000,
      }).trim();
      stagedCount = staged ? staged.split('\n').filter(Boolean).length : 0;
    } catch {
      process.exit(0);
    }

    const MAX_FILES = 8;

    if (stagedCount > MAX_FILES) {
      const output = {
        hookSpecificOutput: {
          hookEventName: 'PreToolUse',
          additionalContext:
            `Advisory: This commit stages ${stagedCount} files (guideline: <=${MAX_FILES}). ` +
            'Consider splitting into smaller per-task commits. Each task should get one ' +
            'commit with a TASK-NNN prefix. Stage only files relevant to one task.',
        },
      };
      process.stdout.write(JSON.stringify(output));
    }

    process.exit(0);
  } catch {
    process.exit(0);
  }
});
