import { ChronoKVClient } from "./client";
import { QueryResponse, AggregationType, dateToTimestamp } from "./types";

/**
 * Fluent query builder for ChronoKV queries.
 *
 * Provides a chainable API for constructing time-range queries
 * with optional key prefix filtering and result limits.
 */
export class QueryBuilder {
  private client: ChronoKVClient;
  private _prefix?: string;
  private _startTime?: number;
  private _endTime?: number;
  private _limit?: number;

  constructor(client: ChronoKVClient) {
    this.client = client;
  }

  /**
   * Filter by key prefix.
   */
  prefix(prefix: string): QueryBuilder {
    this._prefix = prefix;
    return this;
  }

  /**
   * Set the start of the time range (inclusive).
   * Accepts a Date object or a numeric timestamp.
   */
  from(time: Date | number): QueryBuilder {
    if (time instanceof Date) {
      this._startTime = dateToTimestamp(time);
    } else {
      this._startTime = time;
    }
    return this;
  }

  /**
   * Set the end of the time range (inclusive).
   * Accepts a Date object or a numeric timestamp.
   */
  to(time: Date | number): QueryBuilder {
    if (time instanceof Date) {
      this._endTime = dateToTimestamp(time);
    } else {
      this._endTime = time;
    }
    return this;
  }

  /**
   * Set the time range to the last N minutes from now.
   */
  lastMinutes(minutes: number): QueryBuilder {
    const now = new Date();
    this._endTime = dateToTimestamp(now);
    this._startTime = dateToTimestamp(new Date(now.getTime() - minutes * 60 * 1000));
    return this;
  }

  /**
   * Set the time range to the last N hours from now.
   */
  lastHours(hours: number): QueryBuilder {
    const now = new Date();
    this._endTime = dateToTimestamp(now);
    this._startTime = dateToTimestamp(new Date(now.getTime() - hours * 60 * 60 * 1000));
    return this;
  }

  /**
   * Limit the number of results.
   */
  limit(n: number): QueryBuilder {
    this._limit = n;
    return this;
  }

  /**
   * Execute the query and return results.
   */
  async execute(): Promise<QueryResponse> {
    return this.client.query({
      prefix: this._prefix,
      start: this._startTime,
      end: this._endTime,
      limit: this._limit,
    });
  }
}
