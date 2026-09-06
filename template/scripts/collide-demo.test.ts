import { describe, expect, test } from "bun:test";
import { runCollideDemo } from "./collide-demo";

describe("collide-then-recover", () => {
  test("two agents collide then recover", async () => {
    const { steps } = await runCollideDemo({ keep: false });
    expect(steps.every((s) => s.ok)).toBe(true);
    expect(steps.map((s) => s.name)).toContain(
      "agent-b claim brain (expect collision)"
    );
  }, 60_000);
});
