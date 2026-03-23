#!/usr/bin/env node

/**
 * ChronoKV CLI
 *
 * Command-line interface for interacting with a ChronoKV server.
 * Uses the chronokv-sdk for all API communication.
 */

import { Command } from "commander";
import { ChronoKVClient } from "chronokv-sdk/src/client";
import { bytesToString } from "chronokv-sdk/src/types";

const program = new Command();

program
  .name("chronokv")
  .description("CLI for ChronoKV time-series key-value store")
  .version("0.1.0")
  .option("-s, --server <url>", "Server URL", "http://localhost:3000");

program
  .command("put <key> <value>")
  .description("Put a key-value pair")
  .option("--ttl <seconds>", "Time-to-live in seconds")
  .action(async (key: string, value: string, options: { ttl?: string }) => {
    const serverUrl = program.opts().server;
    const client = new ChronoKVClient(serverUrl);

    try {
      const ttl = options.ttl ? parseInt(options.ttl) : undefined;
      const result = await client.put(key, value, ttl);
      console.log(`OK: ${result.key} at timestamp ${result.timestamp}`);
    } catch (err: any) {
      console.error(`Error: ${err.message}`);
      process.exit(1);
    }
  });

program
  .command("get <key>")
  .description("Get the latest value for a key")
  .action(async (key: string) => {
    const serverUrl = program.opts().server;
    const client = new ChronoKVClient(serverUrl);

    try {
      const result = await client.get(key);
      const value = bytesToString(result.value);
      console.log(`${result.key}: ${value}`);
      console.log(`  timestamp: ${result.timestamp}`);
      console.log(`  version: ${result.version}`);
    } catch (err: any) {
      console.error(`Error: ${err.message}`);
      process.exit(1);
    }
  });

program
  .command("delete <key>")
  .description("Delete a key")
  .action(async (key: string) => {
    const serverUrl = program.opts().server;
    const client = new ChronoKVClient(serverUrl);

    try {
      const result = await client.delete(key);
      console.log(`Deleted: ${result.key}`);
    } catch (err: any) {
      console.error(`Error: ${err.message}`);
      process.exit(1);
    }
  });

program
  .command("query")
  .description("Query entries with optional filtering")
  .option("--prefix <prefix>", "Key prefix filter")
  .option("--start <timestamp>", "Start timestamp")
  .option("--end <timestamp>", "End timestamp")
  .option("--limit <n>", "Result limit")
  .option("--last-minutes <n>", "Query last N minutes")
  .action(async (options: any) => {
    const serverUrl = program.opts().server;
    const client = new ChronoKVClient(serverUrl);

    try {
      let params: any = {};

      if (options.lastMinutes) {
        const now = Date.now();
        const minutes = parseInt(options.lastMinutes);
        params.start = now - minutes * 60 * 1000; // milliseconds
        params.end = now;
      } else {
        if (options.start) params.start = parseFloat(options.start);
        if (options.end) params.end = parseFloat(options.end);
      }

      if (options.prefix) params.prefix = options.prefix;
      if (options.limit) params.limit = parseInt(options.limit);

      const result = await client.query(params);
      console.log(`Found ${result.total_count} entries:`);

      for (const entry of result.entries) {
        const value = bytesToString(entry.value);
        console.log(`  ${entry.key}: ${value} (ts=${entry.timestamp})`);
      }
    } catch (err: any) {
      console.error(`Error: ${err.message}`);
      process.exit(1);
    }
  });

program
  .command("health")
  .description("Check server health")
  .action(async () => {
    const serverUrl = program.opts().server;
    const client = new ChronoKVClient(serverUrl);

    try {
      const result = await client.health();
      console.log(`Server: ${result.status} (v${result.version})`);
    } catch (err: any) {
      console.error(`Error: ${err.message}`);
      process.exit(1);
    }
  });

program.parse();
