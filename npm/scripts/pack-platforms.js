"use strict";

/**
 * Assemble per-platform npm packages from release binaries.
 *
 *   node npm/scripts/pack-platforms.js --binaries ./dist --out ./npm/dist/platforms
 *
 * --binaries must contain one file per PLATFORMS[].asset
 * (okfsync-darwin-arm64, okfsync-linux-x64, okfsync-windows-x64.exe, …).
 */

const fs = require("node:fs");
const path = require("node:path");
const { PLATFORMS } = require("../lib/platforms");
const { packageVersion } = require("../lib/resolve");

function printHelp() {
  console.log(`Assemble okfsync-<platform> npm packages from prebuilt binaries.

Usage:
  node npm/scripts/pack-platforms.js --binaries <dir> --out <dir>

Examples:
  node npm/scripts/pack-platforms.js --binaries ./dist --out ./npm/dist/platforms
  node npm/scripts/pack-platforms.js --help

--binaries  Directory of GitHub Release assets (okfsync-darwin-arm64, …)
--out       Directory that will hold one folder per platform package
`);
}

function parseArgs(argv) {
  const out = { binaries: null, out: null };
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    }
    if (arg === "--binaries") {
      out.binaries = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--out") {
      out.out = argv[i + 1];
      i += 1;
      continue;
    }
    console.error(`unknown argument: ${arg}\n`);
    printHelp();
    process.exit(1);
  }
  if (!out.binaries || !out.out) {
    printHelp();
    process.exit(1);
  }
  return out;
}

function copyLicense(destDir) {
  const license = path.join(__dirname, "..", "..", "LICENSE");
  if (fs.existsSync(license)) {
    fs.copyFileSync(license, path.join(destDir, "LICENSE"));
  }
}

function packOne(spec, version, binariesDir, outRoot) {
  const src = path.join(binariesDir, spec.asset);
  if (!fs.existsSync(src) || !fs.statSync(src).isFile()) {
    throw new Error(`missing binary ${spec.asset} in ${binariesDir}`);
  }
  if (fs.statSync(src).size === 0) {
    throw new Error(`empty binary ${spec.asset}`);
  }

  const destDir = path.join(outRoot, spec.npmName);
  const binDir = path.join(destDir, "bin");
  fs.rmSync(destDir, { recursive: true, force: true });
  fs.mkdirSync(binDir, { recursive: true });

  const destBin = path.join(binDir, spec.binName);
  fs.copyFileSync(src, destBin);
  if (process.platform !== "win32") {
    fs.chmodSync(destBin, 0o755);
  }

  const pkg = {
    name: spec.npmName,
    version,
    description: `kbsync binary for ${spec.label} (${spec.key}). Install the okfsync meta-package instead.`,
    license: "MIT",
    os: spec.os,
    cpu: spec.cpu,
    engines: { node: ">=18" },
    files: ["bin/", "README.md", "LICENSE"],
    repository: {
      type: "git",
      url: "git+https://github.com/daniellinuk/okfsync.git",
      directory: "npm",
    },
    homepage: "https://github.com/daniellinuk/okfsync",
    publishConfig: { access: "public" },
  };

  fs.writeFileSync(path.join(destDir, "package.json"), `${JSON.stringify(pkg, null, 2)}\n`);
  fs.writeFileSync(
    path.join(destDir, "README.md"),
    `# ${spec.npmName}\n\n${spec.label} \`kbsync\` binary. Install the meta-package:\n\n\`\`\`bash\nbun add -g okfsync\n# or: npm i -g okfsync\n\`\`\`\n`
  );
  copyLicense(destDir);
  return destDir;
}

function main(argv = process.argv) {
  const args = parseArgs(argv);
  const binariesDir = path.resolve(args.binaries);
  const outRoot = path.resolve(args.out);
  const version = packageVersion();

  if (!fs.existsSync(binariesDir) || !fs.statSync(binariesDir).isDirectory()) {
    throw new Error(`--binaries is not a directory: ${binariesDir}`);
  }

  fs.mkdirSync(outRoot, { recursive: true });
  const packed = [];
  for (const spec of Object.values(PLATFORMS)) {
    packed.push(packOne(spec, version, binariesDir, outRoot));
  }
  console.log(`packed ${packed.length} platform packages @${version} → ${outRoot}`);
  return packed;
}

if (require.main === module) {
  try {
    main();
  } catch (err) {
    console.error(err.message || err);
    process.exit(1);
  }
}

module.exports = { main, parseArgs, packOne };
