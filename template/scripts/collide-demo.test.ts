import { describe, expect, test } from "bun:test";
import { runCollideDemo } from "./collide-demo";

describe("wiki propose", () => {
  test("two agents propose; latest wins; no claim command", async () => {
    const { steps } = await runCollideDemo({ keep: false });
    expect(steps.every((s) => s.ok)).toBe(true);
    expect(steps.map((s) => s.name)).toContain("claim is not a command");
    expect(steps.map((s) => s.name)).toContain("delete is not a command");
    expect(steps.map((s) => s.name)).toContain("gardener is not a command");
  }, 60_000);
});
