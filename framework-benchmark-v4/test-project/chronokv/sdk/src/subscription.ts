import { KeyChangeEvent } from "./types";
import WebSocket from "ws";

/**
 * WebSocket subscription client for key change notifications.
 *
 * Connects to the ChronoKV server's WebSocket endpoint and
 * receives real-time notifications when subscribed keys change.
 */
export class SubscriptionClient {
  private wsUrl: string;
  private ws: WebSocket | null = null;
  private handlers: Map<string, ((event: KeyChangeEvent) => void)[]> = new Map();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor(wsUrl: string) {
    this.wsUrl = wsUrl;
  }

  /**
   * Connect to the WebSocket endpoint.
   */
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.wsUrl);

        this.ws.on("open", () => {
          this.reconnectAttempts = 0;
          resolve();
        });

        this.ws.on("message", (data: WebSocket.RawData) => {
          try {
            const event: KeyChangeEvent = JSON.parse(data.toString());
            const keyHandlers = this.handlers.get(event.key) || [];
            for (const handler of keyHandlers) {
              handler(event);
            }
          } catch {
            // Ignore parse errors
          }
        });

        this.ws.on("close", () => {
          this.handleDisconnect();
        });

        this.ws.on("error", (err: Error) => {
          if (this.reconnectAttempts === 0) {
            reject(err);
          }
        });
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * Subscribe to changes on a specific key.
   */
  onKeyChange(key: string, handler: (event: KeyChangeEvent) => void): void {
    const handlers = this.handlers.get(key) || [];
    handlers.push(handler);
    this.handlers.set(key, handlers);

    // Send subscribe message to server
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ action: "subscribe", key }));
    }
  }

  /**
   * Unsubscribe from changes on a key.
   */
  removeKeyHandler(key: string): void {
    this.handlers.delete(key);

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ action: "unsubscribe", key }));
    }
  }

  /**
   * Disconnect from the WebSocket.
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.handlers.clear();
  }

  private handleDisconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.pow(2, this.reconnectAttempts) * 100;
      setTimeout(() => {
        this.connect().catch(() => {
          // Reconnection failed, will retry
        });
      }, delay);
    }
  }

  /**
   * Check if connected.
   */
  get connected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}
