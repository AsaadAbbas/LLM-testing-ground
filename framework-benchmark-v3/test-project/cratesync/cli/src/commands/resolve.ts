import { Command } from 'commander';

export function createResolveCommand(): Command {
  return new Command('resolve')
    .description('Resolve dependencies and generate lockfile')
    .argument('<packages...>', 'Root package names to resolve')
    .option('-s, --server <url>', 'Server URL', 'http://localhost:3100')
    .option('-o, --output <path>', 'Lockfile output path', 'cratesync.lock')
    .action(async (packages: string[], options: { server: string; output: string }) => {
      // TODO: Implement dependency resolution
      // 1. Call server's /resolve endpoint with root dependencies
      // 2. Display the dependency tree
      // 3. Write lockfile to options.output
      console.error('Error: resolve command not yet implemented');
      process.exit(1);
    });
}
