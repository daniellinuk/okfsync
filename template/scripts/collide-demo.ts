/**
 * Collide-then-recover demo: two agents fight over concepts/brain.md.
 *
 * Agent A bagsies the concept. Agent B's claim fails (collision).
 * Agent A releases. Agent B reclaim succeeds. Lint stays green.
 *
 * Run from repo root: `bun run demo`
 * Or: `bun run --cwd template demo`
 */

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync, cpSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const templateRoot = join(import.meta.dir, "..");
const repoRoot = join(templateRoot, "..");

function bagsyBin(): string {
  if (process.env.BAGSY_BIN && existsSync(process.env.BAGSY_BIN)) {
    return process.env.BAGSY_BIN;
  }
  const release = join(repoRoot, "cli", "target", "release", "bagsy");
  const debug = join(repoRoot, "cli", "target", "debug", "bagsy");
  if (existsSync(release)) return release;
  if (existsSync(debug)) return debug;
  // Try npm wrapper
  const npmBin = join(repoRoot, "npm", "bin", "bagsy.js");
  if (existsSync(npmBin)) return process.execPath; // special-cased below
  throw new Error(
    "bagsy binary not found — run `cargo build -p bagsy` (or `bun run build:cli`) first"
  );
}

function run(
  bin: string,
  args: string[],
  cwd: string,
  env: Record<string, string> = {}
): { ok: boolean; stdout: string; stderr: string; status: number | null } {
  const useNodeWrapper =
    bin === process.execPath || bin.endsWith("bagsy.js");
  const command = useNodeWrapper ? process.execPath : bin;
  const fullArgs = useNodeWrapper
    ? [join(repoRoot, "npm", "bin", "bagsy.js"), ...args]
    : args;

  const res = spawnSync(command, fullArgs, {
    cwd,
    env: { ...process.env, ...env },
    encoding: "utf8",
  });
  return {
    ok: res.status === 0,
    stdout: res.stdout ?? "",
    stderr: res.stderr ?? "",
    status: res.status,
  };
}

export function runCollideDemo(options: { keep?: boolean } = {}): {
  workDir: string;
  steps: { name: string; ok: boolean; detail: string }[];
} {
  const bin = bagsyBin();
  const workDir = join(
    tmpdir(),
    `bagsy-collide-${Date.now()}-${Math.random().toString(16).slice(2)}`
  );
  mkdirSync(workDir, { recursive: true });
  cpSync(templateRoot, workDir, {
    recursive: true,
    filter: (src) => !src.includes("node_modules") && !src.includes(".git"),
  });

  // Fresh git repo for branch claims
  spawnSync("git", ["init", "-b", "main"], { cwd: workDir });
  spawnSync("git", ["config", "user.email", "demo@bagsy.dev"], { cwd: workDir });
  spawnSync("git", ["config", "user.name", "bagsy-demo"], { cwd: workDir });
  spawnSync("git", ["add", "."], { cwd: workDir });
  spawnSync("git", ["commit", "-m", "chore: seed okf template"], { cwd: workDir });

  const steps: { name: string; ok: boolean; detail: string }[] = [];

  const step = (name: string, args: string[], env: Record<string, string>, expectOk: boolean) => {
    const res = run(bin, args, workDir, env);
    const ok = res.ok === expectOk;
    const detail = (res.stdout + res.stderr).trim();
    steps.push({ name, ok, detail });
    return res;
  };

  console.log(`bagsy collide demo → ${workDir}`);
  console.log(`binary: ${bin === process.execPath ? "npm wrapper" : bin}`);
  console.log();

  step("lint (clean)", ["lint"], {}, true);

  step(
    "agent-a claim brain",
    ["claim", "brain", "--agent", "agent-a"],
    { BAGSY_AGENT: "agent-a" },
    true
  );

  step(
    "agent-b claim brain (expect collision)",
    ["claim", "brain", "--agent", "agent-b", "--no-branch"],
    { BAGSY_AGENT: "agent-b" },
    false
  );

  step(
    "agent-a release brain",
    ["release", "brain", "--agent", "agent-a"],
    { BAGSY_AGENT: "agent-a" },
    true
  );

  step(
    "agent-b reclaim brain",
    ["claim", "brain", "--agent", "agent-b", "--no-branch"],
    { BAGSY_AGENT: "agent-b" },
    true
  );

  step("lint after recover", ["lint"], {}, true);

  step(
    "agent-b release",
    ["release", "brain", "--agent", "agent-b"],
    { BAGSY_AGENT: "agent-b" },
    true
  );

  let failed = 0;
  for (const s of steps) {
    const mark = s.ok ? "ok" : "FAIL";
    if (!s.ok) failed++;
    console.log(`[${mark}] ${s.name}`);
    if (!s.ok || process.env.BAGSY_DEMO_VERBOSE === "1") {
      if (s.detail) console.log(`       ${s.detail.split("\n")[0]}`);
    }
  }

  console.log();
  if (failed) {
    console.error(`demo failed: ${failed} step(s)`);
    if (!options.keep) {
      // leave dir for inspection on failure
      console.error(`work dir kept: ${workDir}`);
    }
    process.exitCode = 1;
  } else {
    console.log("collide → recover succeeded. Agents don't clobber the brain.");
    if (!options.keep && process.env.BAGSY_DEMO_KEEP !== "1") {
      rmSync(workDir, { recursive: true, force: true });
    } else {
      console.log(`work dir: ${workDir}`);
    }
  }

  return { workDir, steps };
}

if (import.meta.main) {
  runCollideDemo();
}
