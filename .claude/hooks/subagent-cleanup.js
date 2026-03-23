#!/usr/bin/env node
// V5.1 Subagent Cleanup Hook — SubagentStop
//
// When a subagent finishes:
// 1. Reads which TASK-NNN the operator was working on (from ROADMAP.md)
// 2. Merges key artifacts to .tasks/TASK-NNN/ (permanent)
// 3. Deletes the operator temp folder .tasks/.operators/{agent_id}/

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
    const agentId = data.agent_id;

    if (!agentId) {
      process.exit(0);
    }

    const operatorDir = path.join(cwd, '.tasks', '.operators', agentId);

    if (!fs.existsSync(operatorDir)) {
      process.exit(0);
    }

    // Try to find which TASK-NNN this operator was working on
    let taskId = null;
    const roadmapPath = path.join(operatorDir, 'ROADMAP.md');
    if (fs.existsSync(roadmapPath)) {
      const roadmap = fs.readFileSync(roadmapPath, 'utf8');
      const match = roadmap.match(/TASK-(\d+)/);
      if (match) {
        taskId = `TASK-${match[1]}`;
      }
    }

    // If we found a task, merge artifacts
    if (taskId) {
      const taskDir = path.join(cwd, '.tasks', taskId);
      if (fs.existsSync(taskDir)) {
        const filesToMerge = ['VERIFICATION.md', 'DECISIONS.md', 'CONTEXT.md'];

        for (const file of filesToMerge) {
          const src = path.join(operatorDir, file);
          if (fs.existsSync(src)) {
            const content = fs.readFileSync(src, 'utf8');
            // Only merge if the file has been written to (not just skeleton)
            if (content.length > 200) {
              const destName = file.toLowerCase().replace('.md', '') + '.md';
              const dest = path.join(taskDir, destName);
              fs.writeFileSync(dest, content);
            }
          }
        }
      }
    }

    // Delete the operator temp folder
    try {
      fs.rmSync(operatorDir, { recursive: true, force: true });
    } catch {
      // Best effort cleanup
    }

    process.exit(0);
  } catch {
    process.exit(0);
  }
});
