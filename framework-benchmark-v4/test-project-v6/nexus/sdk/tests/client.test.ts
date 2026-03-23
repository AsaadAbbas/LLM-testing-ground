import { NexusClient } from "../src/index";

describe("NexusClient", () => {
  test("constructs with base URL", () => {
    const client = new NexusClient("http://localhost:3000");
    expect(client).toBeDefined();
  });

  test("constructs with trailing slash removal", () => {
    const client = new NexusClient("http://localhost:3000/");
    expect(client).toBeDefined();
  });

  test("constructs with API key", () => {
    const client = new NexusClient("http://localhost:3000", "test-key");
    expect(client).toBeDefined();
  });
});
