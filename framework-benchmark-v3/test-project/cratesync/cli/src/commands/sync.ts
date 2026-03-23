import { Command } from 'commander';
import { CrateSyncClient } from '../client.js';
import { formatVersion } from '../protocol.js';

export function createSyncCommand(): Command {
  return new Command('sync')
    .description('Sync packages from registry')
    .argument('<packages...>', 'Package names to sync')
    .option('-s, --server <url>', 'Server URL', 'http://localhost:3100')
    .action(async (packages: string[], options: { server: string }) => {
      const client = new CrateSyncClient(options.server);

      console.log(`Syncing ${packages.length} package(s)...`);

      try {
        const result = await client.sync(packages);

        for (const name of result.synced) {
          console.log(`  ✓ ${name}`);
        }

        for (const name of result.failed) {
          console.log(`  ✗ ${name}`);
        }

        console.log(`\nSynced: ${result.synced.length}, Failed: ${result.failed.length}`);
      } catch (error) {
        console.error('Sync failed:', error instanceof Error ? error.message : error);
        process.exit(1);
      }
    });
}
