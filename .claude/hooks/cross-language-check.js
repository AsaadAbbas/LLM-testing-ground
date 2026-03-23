#!/usr/bin/env node
// V5 Cross-Language Check Hook — PostToolUse on Edit|Write
// When shared type files are modified, reminds about cross-language alignment.

const path = require('path');

// Files that define cross-language contracts
const SHARED_TYPE_PATTERNS = [
  'types.ts', 'protocol.json', 'protocol.ts',
  'core/src/lib.rs', 'shared/',
];

let input = '';
const stdinTimeout = setTimeout(() => process.exit(0), 4000);
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  clearTimeout(stdinTimeout);
  try {
    const data = JSON.parse(input);
    const toolName = data.tool_name;

    if (toolName !== 'Write' && toolName !== 'Edit') {
      process.exit(0);
    }

    const filePath = data.tool_input?.file_path || '';
    const basename = path.basename(filePath);

    const isSharedType = SHARED_TYPE_PATTERNS.some(pattern =>
      filePath.includes(pattern) || basename === pattern
    );

    if (isSharedType) {
      const output = {
        hookSpecificOutput: {
          hookEventName: 'PostToolUse',
          additionalContext:
            `Cross-language type file modified: ${basename}. ` +
            'Verify: (1) Rust serde serialization matches TypeScript parsing, ' +
            '(2) all API endpoints use the updated types, ' +
            '(3) SDK methods handle the new type correctly. ' +
            'Run both cargo test AND tsc --noEmit after changes.',
        },
      };
      process.stdout.write(JSON.stringify(output));
    }

    process.exit(0);
  } catch {
    process.exit(0);
  }
});
