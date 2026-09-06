/**
 * Wiki demo: two agents propose to the same concept via bagsy serve.
 * Last write wins on disk; git on the server still has both commits.
 * The CLI cannot delete knowledge.
 */

import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync, cpSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createConnection } from "node:net";

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
  const res = spawnSync(bin, args, {
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

function waitPort(port: number, ms = 5000): Promise<void> {
  const deadline = Date.now() + ms;
  return new Promise((resolve, reject) => {
    const tryOnce = () => {
      const sock = createConnection({ host: "127.0.0.1", port }, () => {
        sock.end();
        resolve();
      });
      sock.on("error", () => {
        sock.destroy();
        if (Date.now() > deadline) reject(new Error(`serve did not bind :${port}`));
        else setTimeout(tryOnce, 25);
      });
    };
    tryOnce();
  });
}

export async function runCollideDemo(options: { keep?: boolean } = {}): Promise<{
  workDir: string;
  steps: { name: string; ok: boolean; detail: string }[];
}> {
  const bin = bagsyBin();
  const workDir = join(
    tmpdir(),
    `bagsy-wiki-${Date.now()}-${Math.random().toString(16).slice(2)}`
  );
  mkdirSync(workDir, { recursive: true });
  cpSync(templateRoot, workDir, {
    recursive: true,
    filter: (src) => !src.includes("node_modules") && !src.includes(".git"),
  });

  spawnSync("git", ["init", "-b", "main"], { cwd: workDir });
  spawnSync("git", ["config", "user.email", "demo@bagsy.dev"], { cwd: workDir });
  spawnSync("git", ["config", "user.name", "bagsy-demo"], { cwd: workDir });
  spawnSync("git", ["add", "."], { cwd: workDir });
  spawnSync("git", ["commit", "-m", "chore: seed okf template"], { cwd: workDir });

  const port = 20000 + Math.floor(Math.random() * 20000);
  const url = `http://127.0.0.1:${port}`;
  const server = spawn(bin, ["serve", "--root", workDir, "--bind", `127.0.0.1:${port}`], {
    cwd: workDir,
    stdio: "pipe",
  });

  const steps: { name: string; ok: boolean; detail: string }[] = [];
  const step = (
    name: string,
    args: string[],
    env: Record<string, string>,
    expectOk: boolean
  ) => {
    const res = run(bin, args, workDir, env);
    const ok = res.ok === expectOk;
    const detail = (res.stdout + res.stderr).trim();
    steps.push({ name, ok, detail });
    return res;
  };

  try {
    await waitPort(port);

    const tokA = JSON.parse(
      run(bin, ["--root", workDir, "token", "create", "--agent", "agent-a", "--json"], workDir)
        .stdout
    ).token;
    const tokB = JSON.parse(
      run(bin, ["--root", workDir, "token", "create", "--agent", "agent-b", "--json"], workDir)
        .stdout
    ).token;

    const envA = { BAGSY_URL: url, BAGSY_TOKEN: tokA };
    const envB = { BAGSY_URL: url, BAGSY_TOKEN: tokB };

    writeFileSync(
      join(workDir, "a.md"),
      "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nFrom agent A.\n"
    );
    writeFileSync(
      join(workDir, "b.md"),
      "---\ntype: Playbook\ntitle: Shared Brain\n---\n\nFrom agent B.\n"
    );

    console.log(`bagsy wiki demo → ${workDir}`);
    console.log(`binary: ${bin}`);
    console.log(`server: ${url}`);
    console.log();

    step("lint (clean)", ["lint", "--root", workDir], {}, true);
    step("agent-a get brain", ["get", "brain"], envA, true);
    step("agent-a propose brain", ["propose", "brain", "--file", "a.md"], envA, true);
    step("agent-b propose brain", ["propose", "brain", "--file", "b.md"], envB, true);
    const got = step("agent-b get brain (latest)", ["get", "brain"], envB, true);
    const latestOk = got.ok && got.detail.includes("From agent B");
    steps[steps.length - 1].ok = latestOk;
    step("lint after proposes", ["lint"], envB, true);
    step("claim is not a command", ["claim", "brain"], envA, false);
    step("delete is not a command", ["delete", "brain"], envA, false);
    step("gardener is not a command", ["gardener"], envA, false);

    const log = spawnSync("git", ["log", "--oneline"], { cwd: workDir, encoding: "utf8" });
    const historyOk =
      (log.stdout ?? "").includes("agent-a") && (log.stdout ?? "").includes("agent-b");
    steps.push({
      name: "git history keeps both proposes",
      ok: historyOk,
      detail: (log.stdout ?? "").trim(),
    });
  } finally {
    server.kill("SIGTERM");
  }

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
    console.error(`work dir kept: ${workDir}`);
    process.exitCode = 1;
  } else {
    console.log("wiki propose succeeded. Latest wins; git still has history.");
    if (!options.keep && process.env.BAGSY_DEMO_KEEP !== "1") {
      rmSync(workDir, { recursive: true, force: true });
    } else {
      console.log(`work dir: ${workDir}`);
    }
  }

  return { workDir, steps };
}

if (import.meta.main) {
  await runCollideDemo();
}
