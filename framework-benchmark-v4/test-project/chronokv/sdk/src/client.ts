import {
  ApiResponse,
  GetResponse,
  PutRequest,
  PutResponse,
  QueryParams,
  QueryResponse,
  stringToBytes,
} from "./types";

/**
 * ChronoKV HTTP client.
 *
 * Communicates with the ChronoKV server over HTTP.
 */
export class ChronoKVClient {
  private baseUrl: string;
  private apiKey?: string;

  constructor(baseUrl: string, apiKey?: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.apiKey = apiKey;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this.apiKey) {
      headers["Authorization"] = `Bearer ${this.apiKey}`;
    }

    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`HTTP ${response.status}: ${text}`);
    }

    const json = (await response.json()) as ApiResponse<T>;

    if (!json.success) {
      throw new Error(json.error || "Unknown error");
    }

    return json.data!;
  }

  /**
   * Put a value for a key.
   */
  async put(key: string, value: string, ttl?: number): Promise<PutResponse> {
    const body: PutRequest = {
      key,
      value: stringToBytes(value),
      ttl,
    };

    return this.request<PutResponse>("POST", `/api/v1/kv/${key}`, body);
  }

  /**
   * Get the latest value for a key.
   */
  async get(key: string): Promise<GetResponse> {
    return this.request<GetResponse>("GET", `/api/v1/kv/${key}`);
  }

  /**
   * Delete a key.
   */
  async delete(key: string): Promise<PutResponse> {
    return this.request<PutResponse>("DELETE", `/api/v1/kv/${key}`);
  }

  /**
   * Query entries with optional time range and key prefix filtering.
   */
  async query(params: QueryParams): Promise<QueryResponse> {
    const searchParams = new URLSearchParams();
    if (params.start !== undefined) searchParams.set("start", String(params.start));
    if (params.end !== undefined) searchParams.set("end", String(params.end));
    if (params.limit !== undefined) searchParams.set("limit", String(params.limit));
    if (params.prefix !== undefined) searchParams.set("prefix", params.prefix);

    const queryString = searchParams.toString();
    const path = queryString ? `/api/v1/query?${queryString}` : "/api/v1/query";

    return this.request<QueryResponse>("GET", path);
  }

  /**
   * Query entries within a time window specified by JavaScript Dates.
   *
   * Converts Date objects to API timestamps for the query.
   */
  async queryByDateRange(
    startDate: Date,
    endDate: Date,
    options?: { prefix?: string; limit?: number }
  ): Promise<QueryResponse> {
    return this.query({
      start: startDate.getTime(), // milliseconds
      end: endDate.getTime(), // milliseconds
      prefix: options?.prefix,
      limit: options?.limit,
    });
  }

  /**
   * Health check.
   */
  async health(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/api/v1/health`);
    return (await response.json()) as { status: string; version: string };
  }
}
