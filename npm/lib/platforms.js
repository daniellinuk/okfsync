"use strict";

/**
 * Prebuilt kbsync targets. `darwin-*` is macOS (Apple Silicon and Intel).
 * npm optionalDependencies and GitHub Release assets use these names.
 */

const PLATFORMS = {
  "darwin-arm64": {
    key: "darwin-arm64",
    npmName: "okfsync-darwin-arm64",
    os: ["darwin"],
    cpu: ["arm64"],
    rustTarget: "aarch64-apple-darwin",
    asset: "okfsync-darwin-arm64",
    binName: "kbsync",
    label: "macOS Apple Silicon",
  },
  "darwin-x64": {
    key: "darwin-x64",
    npmName: "okfsync-darwin-x64",
    os: ["darwin"],
    cpu: ["x64"],
    rustTarget: "x86_64-apple-darwin",
    asset: "okfsync-darwin-x64",
    binName: "kbsync",
    label: "macOS Intel",
  },
  "linux-x64": {
    key: "linux-x64",
    npmName: "okfsync-linux-x64",
    os: ["linux"],
    cpu: ["x64"],
    rustTarget: "x86_64-unknown-linux-gnu",
    asset: "okfsync-linux-x64",
    binName: "kbsync",
    label: "Linux x64",
  },
  "linux-arm64": {
    key: "linux-arm64",
    npmName: "okfsync-linux-arm64",
    os: ["linux"],
    cpu: ["arm64"],
    rustTarget: "aarch64-unknown-linux-gnu",
    asset: "okfsync-linux-arm64",
    binName: "kbsync",
    label: "Linux ARM64",
  },
  "win32-x64": {
    key: "win32-x64",
    npmName: "okfsync-windows-x64",
    os: ["win32"],
    cpu: ["x64"],
    rustTarget: "x86_64-pc-windows-msvc",
    asset: "okfsync-windows-x64.exe",
    binName: "kbsync.exe",
    label: "Windows x64",
  },
};

const PLATFORM_MAP = Object.fromEntries(
  Object.entries(PLATFORMS).map(([key, spec]) => [key, spec.npmName])
);

function optionalDependencyMap(version) {
  const out = {};
  for (const spec of Object.values(PLATFORMS)) {
    out[spec.npmName] = version;
  }
  return out;
}

module.exports = {
  PLATFORMS,
  PLATFORM_MAP,
  optionalDependencyMap,
};
