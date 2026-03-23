import { Command } from 'commander';
import { createSyncCommand } from './commands/sync.js';
import { createInspectCommand } from './commands/inspect.js';
import { createResolveCommand } from './commands/resolve.js';

const program = new Command();

program
  .name('cratesync')
  .description('CrateSync — Package registry sync tool')
  .version('0.1.0');

program.addCommand(createSyncCommand());
program.addCommand(createInspectCommand());
program.addCommand(createResolveCommand());

program.parse();
