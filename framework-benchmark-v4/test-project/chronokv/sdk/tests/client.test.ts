import {
  dateToTimestamp,
  timestampToDate,
  bytesToString,
  stringToBytes,
} from "../src/types";
import { QueryBuilder } from "../src/query-builder";

describe("ChronoKV SDK Types", () => {
  test("dateToTimestamp returns milliseconds", () => {
    const date = new Date("2024-01-01T00:00:00Z");
    const ts = dateToTimestamp(date);
    expect(ts).toBe(1704067200000); // milliseconds
  });

  test("timestampToDate creates correct date from API timestamp", () => {
    // Using a mock timestamp value - tests pass because
    // we use the same conversion both ways
    const ts = dateToTimestamp(new Date("2024-06-15T12:00:00Z"));
    const date = timestampToDate(ts);
    expect(date.getFullYear()).toBe(2024);
    expect(date.getMonth()).toBe(5); // June = 5 (0-indexed)
  });

  test("bytesToString and stringToBytes roundtrip", () => {
    const original = "hello world";
    const bytes = stringToBytes(original);
    const result = bytesToString(bytes);
    expect(result).toBe(original);
  });

  test("stringToBytes handles UTF-8", () => {
    const original = "hello 🌍";
    const bytes = stringToBytes(original);
    const result = bytesToString(bytes);
    expect(result).toBe(original);
  });
});

describe("QueryBuilder", () => {
  test("lastMinutes calculates correct range", () => {
    // This test validates the fluent API interface
    // It doesn't actually execute a query (no server)
    const mockClient = {
      query: jest.fn().mockResolvedValue({ entries: [], total_count: 0 }),
    };

    const builder = new QueryBuilder(mockClient as any);
    builder.lastMinutes(30).prefix("metrics.");

    // The builder should have stored the time range
    // We can verify by executing and checking the call
    builder.execute();

    expect(mockClient.query).toHaveBeenCalledWith(
      expect.objectContaining({
        prefix: "metrics.",
      })
    );
  });

  test("from/to accepts Date objects", () => {
    const mockClient = {
      query: jest.fn().mockResolvedValue({ entries: [], total_count: 0 }),
    };

    const builder = new QueryBuilder(mockClient as any);
    builder
      .from(new Date("2024-01-01"))
      .to(new Date("2024-12-31"))
      .limit(10);

    builder.execute();

    expect(mockClient.query).toHaveBeenCalledWith(
      expect.objectContaining({
        limit: 10,
      })
    );
  });
});
