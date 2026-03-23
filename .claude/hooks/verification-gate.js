#!/usr/bin/env node
// V5 Verification Gate Hook — UserPromptSubmit
// Injects verification reminders based on conversation state.
// Also classifies input complexity for GSD routing.

const fs = require('fs');
const path = require('path');

let input = '';
const stdinTimeout = setTimeout(() => process.exit(0), 4000);
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  clearTimeout(stdinTimeout);
  try {
    const data = JSON.parse(input);
    const cwd = data.cwd || process.cwd();

    // Check if .tasks/ exists (work is in progress)
    const tasksExist = fs.existsSync(path.join(cwd, '.tasks'));

    // Check for completion signals in the user's message
    const userMessage = data.message?.content || '';
    const isCompletionClaim = /\b(done|complete|finished|fixed|implemented|all\s+pass)\b/i.test(userMessage);

    const context = [];

    // If user claims completion, remind about verification
    if (isCompletionClaim) {
      context.push(
        'Before confirming completion, run and paste actual output of: ' +
        '`cargo test`, `cargo clippy`, `tsc --noEmit`. ' +
        'Then answer: "How could tests pass while the feature is broken?" ' +
        'If you can name a scenario, add a test for it.'
      );
    }

    // If complex work without .tasks/, suggest Atomic
    if (!tasksExist && userMessage.length > 200) {
      const taskSignals = (userMessage.match(/\b(fix|implement|add|create|build|refactor|migrate)\b/gi) || []).length;
      if (taskSignals >= 3) {
        context.push(
          'This looks like complex work with multiple tasks. ' +
          'Consider running /atomic first to decompose into structured .tasks/ ' +
          'or /gsd:quick for smaller scoped work.'
        );
      }
    }

    if (context.length > 0) {
      const output = {
        hookSpecificOutput: {
          hookEventName: 'UserPromptSubmit',
          additionalContext: context.join('\n\n'),
        },
      };
      process.stdout.write(JSON.stringify(output));
    }

    process.exit(0);
  } catch {
    process.exit(0);
  }
});
