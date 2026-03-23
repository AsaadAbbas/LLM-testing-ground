import type { Package, SyncRequest, SyncResponse, ResolveRequest, ResolveResponse } from './protocol.js';

export class CrateSyncClient {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:3100') {
    this.baseUrl = baseUrl;
  }

  async listPackages(): Promise<string[]> {
    const res = await fetch(`${this.baseUrl}/packages`);
    if (!res.ok) throw new Error(`Failed to list packages: ${res.status}`);
    return res.json();
  }

  async getPackage(name: string): Promise<Package> {
    const res = await fetch(`${this.baseUrl}/packages/${name}`);
    if (!res.ok) throw new Error(`Package not found: ${name}`);
    return res.json();
  }

  async sync(packages: string[]): Promise<SyncResponse> {
    const req: SyncRequest = { packages };
    const res = await fetch(`${this.baseUrl}/sync`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    if (!res.ok) throw new Error(`Sync failed: ${res.status}`);
    return res.json();
  }

  async resolve(dependencies: { name: string; version_req: string }[]): Promise<ResolveResponse> {
    const req: ResolveRequest = { root_dependencies: dependencies };
    const res = await fetch(`${this.baseUrl}/resolve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    if (!res.ok) throw new Error(`Resolution failed: ${res.status}`);
    return res.json();
  }

  async health(): Promise<{ status: string }> {
    const res = await fetch(`${this.baseUrl}/health`);
    return res.json();
  }
}
