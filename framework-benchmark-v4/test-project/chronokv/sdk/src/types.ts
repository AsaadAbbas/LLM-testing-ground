/**
 * ChronoKV TypeScript SDK Types
 *
 * These types mirror the Rust API contract. The server serializes
 * timestamps as f64 values representing time since epoch.
 */

/** API response wrapper matching the Rust ApiResponse struct. */
export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

/** Response from PUT operations. */
export interface PutResponse {
  key: string;
  timestamp: number;
}

/** Response from GET operations. */
export interface GetResponse {
  key: string;
  value: number[]; // byte array serialized as number array
  timestamp: number;
  version: number;
}

/** Response from query operations. */
export interface QueryResponse {
  entries: GetResponse[];
  total_count: number;
}

/** Request body for PUT operations. */
export interface PutRequest {
  key: string;
  value: number[];
  ttl?: number;
}

/** Query parameters for range queries. */
export interface QueryParams {
  start?: number;
  end?: number;
  limit?: number;
  prefix?: string;
}

/** Aggregation types. */
export type AggregationType = "Min" | "Max" | "Avg" | "Count" | "Sum";

/** Aggregation result. */
export interface AggregationResult {
  agg_type: AggregationType;
  value: number;
  count: number;
}

/** WebSocket key change event. */
export interface KeyChangeEvent {
  key: string;
  timestamp: number;
  event_type: string;
}

/** Convert a JavaScript Date to the timestamp format used by ChronoKV API. */
export function dateToTimestamp(date: Date): number {
  // JavaScript Date.getTime() returns milliseconds since epoch
  return date.getTime();
}

/** Convert a ChronoKV API timestamp to a JavaScript Date. */
export function timestampToDate(timestamp: number): Date {
  // API timestamps are treated as milliseconds for JavaScript compatibility
  return new Date(timestamp);
}

/** Convert a byte array from the API to a UTF-8 string. */
export function bytesToString(bytes: number[]): string {
  return Buffer.from(bytes).toString("utf-8");
}

/** Convert a UTF-8 string to a byte array for the API. */
export function stringToBytes(str: string): number[] {
  return Array.from(Buffer.from(str, "utf-8"));
}
