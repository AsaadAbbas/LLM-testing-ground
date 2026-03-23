import { Command } from 'commander';
import { CrateSyncClient } from '../client.js';
import { formatVersion } from '../protocol.js';

export function createInspectCommand(): Command {
  return new Command('inspect')
    .description('Inspect a package from the registry')
    .argument('<name>', 'Package name')
    .option('-s, --server <url>', 'Server URL', 'http://localhost:3100')
    .action(async (name: string, options: { server: string }) => {
      const client = new CrateSyncClient(options.server);

      try {
        const pkg = await client.getPackage(name);

        console.log(`Package: ${pkg.name}`);
        console.log(`Versions:`);

        for (const manifest of pkg.versions) {
          // BUG #2 surfaces here: formatVersion returns undefined
          // because Rust sends { major, minor, patch } not { version: "..." }
          const versionStr = formatVersion(manifest.version);
          console.log(`  ${versionStr} — ${manifest.description || 'No description'}`);

          if (manifest.dependencies.length > 0) {
            console.log(`    Dependencies:`);
            for (const dep of manifest.dependencies) {
              console.log(`      ${dep.name} ${dep.version_req}`);
            }
          }
        }
      } catch (error) {
        console.error('Inspect failed:', error instanceof Error ? error.message : error);
        process.exit(1);
      }
    });
}
