/**
 * Template lint entry — prefers the kbsync binary, falls back to a tiny OKF check.
 */
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const repoRoot = join(root, "..");

function findKbsync(): string | null {
  if (process.env.KBSYNC_BIN && existsSync(process.env.KBSYNC_BIN)) {
    return process.env.KBSYNC_BIN;
  }
  for (const p of [
    join(repoRoot, "cli", "target", "release", "kbsync"),
    join(repoRoot, "cli", "target", "debug", "kbsync"),
  ]) {
    if (existsSync(p)) return p;
  }
  return null;
}

function walkMd(dir: string, out: string[] = []): string[] {
  if (!existsSync(dir)) return out;
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) walkMd(p, out);
    else if (name.endsWith(".md") && name !== "index.md" && name !== "log.md") {
      out.push(p);
    }
  }
  return out;
}

function fallbackLint(): number {
  const files = walkMd(join(root, "concepts"));
  let errors = 0;
  for (const file of files) {
    const text = readFileSync(file, "utf8");
    if (!text.startsWith("---")) {
      console.error(`error: ${file}: missing YAML frontmatter`);
      errors++;
      continue;
    }
    const end = text.indexOf("\n---", 3);
    if (end < 0) {
      console.error(`error: ${file}: unterminated frontmatter`);
      errors++;
      continue;
    }
    const yaml = text.slice(3, end);
    if (!/^\s*type:\s*\S+/m.test(yaml)) {
      console.error(`error: ${file}: missing required type:`);
      errors++;
    }
  }
  console.log(
    `fallback lint: ${files.length} concept(s), ${errors} error(s)`
  );
  return errors;
}

const bin = findKbsync();
if (bin) {
  const res = spawnSync(bin, ["lint", "--root", root], {
    encoding: "utf8",
    stdio: "inherit",
  });
  process.exit(res.status ?? 1);
} else {
  console.warn("kbsync binary not built — running fallback OKF lint");
  process.exit(fallbackLint() === 0 ? 0 : 1);
}
