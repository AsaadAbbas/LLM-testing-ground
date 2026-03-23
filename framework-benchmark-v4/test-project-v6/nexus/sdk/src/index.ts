/**
 * Nexus API Gateway TypeScript SDK
 */

export interface NexusRequest {
  method: string;
  path: string;
  headers?: Record<string, string>;
  body?: unknown;
  queryParams?: Record<string, string>;
}

export interface NexusResponse {
  status: number;
  headers: Record<string, string>;
  body: unknown;
}

export interface AdminEvent {
  event_type: string;
  payload: unknown;
}

export interface PipelineMetrics {
  total_requests: number;
  total_errors: number;
  total_rejections: number;
  avg_latency_ms: number;
  active_connections: number;
  middleware_timings: Record<string, number>;
}

export interface RouteConfig {
  method: string;
  path: string;
  backend: string;
  settings?: Record<string, unknown>;
}

export interface NexusConfig {
  server: { host: string; port: number };
  logging: { enabled: boolean };
  cors: {
    enabled: boolean;
    permissive: boolean;
    allowed_origins: string[];
  };
  auth: { enabled: boolean; api_keys: string[] };
  rate_limit: {
    enabled: boolean;
    max_requests: number;
    refill_rate: number;
  };
  transform: {
    enabled: boolean;
    envelope: boolean;
    strip_fields: string[];
  };
  routes: RouteConfig[];
}

/**
 * Nexus Gateway Client
 */
export class NexusClient {
  private baseUrl: string;
  private apiKey?: string;
  private defaultHeaders: Record<string, string>;

  constructor(baseUrl: string, apiKey?: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.apiKey = apiKey;
    this.defaultHeaders = {
      "Content-Type": "application/json",
    };
  }

  private async request(
    method: string,
    path: string,
    body?: unknown,
    headers?: Record<string, string>
  ): Promise<NexusResponse> {
    const allHeaders = { ...this.defaultHeaders, ...headers };

    if (this.apiKey) {
      allHeaders["Authorization"] = `Bearer ${this.apiKey}`;
    }

    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      method,
      headers: allHeaders,
      body: body ? JSON.stringify(body) : undefined,
    });

    const responseHeaders: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      responseHeaders[key] = value;
    });

    let responseBody: unknown;
    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("json")) {
      responseBody = await response.json();
    } else {
      responseBody = await response.text();
    }

    return {
      status: response.status,
      headers: responseHeaders,
      body: responseBody,
    };
  }

  async get(path: string): Promise<NexusResponse> {
    return this.request("GET", path);
  }

  async post(path: string, body: unknown): Promise<NexusResponse> {
    return this.request("POST", path, body);
  }

  async put(path: string, body: unknown): Promise<NexusResponse> {
    return this.request("PUT", path, body);
  }

  async delete(path: string): Promise<NexusResponse> {
    return this.request("DELETE", path);
  }

  /** Get gateway metrics from admin API */
  async getMetrics(): Promise<PipelineMetrics> {
    const resp = await this.get("/admin/metrics");
    return resp.body as PipelineMetrics;
  }

  /** Get current gateway config from admin API */
  async getConfig(): Promise<NexusConfig> {
    const resp = await this.get("/admin/config");
    return resp.body as NexusConfig;
  }

  /** Add a route dynamically via admin API */
  async addRoute(route: RouteConfig): Promise<NexusResponse> {
    return this.post("/admin/routes", route);
  }

  /** Reload gateway configuration */
  async reloadConfig(config: NexusConfig): Promise<NexusResponse> {
    return this.post("/admin/reload", config);
  }

  /** Subscribe to admin events via SSE */
  subscribeEvents(onEvent: (event: AdminEvent) => void): AbortController {
    const controller = new AbortController();
    const url = `${this.baseUrl}/admin/events`;

    // Note: This is a simplified SSE client.
    // In production, use a proper EventSource with reconnection.
    fetch(url, {
      headers: this.apiKey
        ? { Authorization: `Bearer ${this.apiKey}` }
        : {},
      signal: controller.signal,
    })
      .then(async (response) => {
        const reader = response.body?.getReader();
        if (!reader) return;

        const decoder = new TextDecoder();
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const text = decoder.decode(value, { stream: true });
          const lines = text.split("\n").filter((l) => l.startsWith("data:"));

          for (const line of lines) {
            try {
              const data = JSON.parse(line.slice(5));
              onEvent(data as AdminEvent);
            } catch {
              // Ignore parse errors
            }
          }
        }
      })
      .catch(() => {
        // Connection closed
      });

    return controller;
  }
}
