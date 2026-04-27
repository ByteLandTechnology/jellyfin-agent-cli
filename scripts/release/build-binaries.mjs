#!/usr/bin/env node
// Bumps Cargo.toml [package].version, builds all configured targets, and
// creates dist/ archives with sha256 checksums. Called by @semantic-release/exec
// prepareCmd with the next release version, before sync-platform-packages.mjs.

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { readReleaseConfig } from "./release-config.mjs";

// cargo-zigbuild 0.22.x ships a multicall `ar` backed by a bundled llvm-ar
// 21.x that crashes on `cq <archive> <objs>` with "unable to open <archive>".
// `cc-rs` build scripts (ring, aws-lc-sys, …) hit this path and fail before
// any `.a` is created, surfacing later as a link-time "No such file" error.
// Override AR with Homebrew's llvm-ar (22.x), which handles the same args
// fine. Also covers the case where the cargo-zigbuild wrapper would otherwise
// fall through to Apple's /usr/bin/ar (can't archive ELF objects).
function resolveLinuxAr() {
  const r = spawnSync("brew", ["--prefix", "llvm"], { encoding: "utf8" });
  if (r.status !== 0) {
    throw new Error(
      "Homebrew llvm not installed — required for Linux musl cross-compile " +
        "(cargo-zigbuild's bundled llvm-ar is broken for cc-rs `cq` calls). " +
        "Install via `brew install llvm`.",
    );
  }
  const candidate = path.join(r.stdout.trim(), "bin", "llvm-ar");
  if (!existsSync(candidate)) {
    throw new Error(`Expected llvm-ar at ${candidate}, but it does not exist.`);
  }
  return candidate;
}

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const version = process.argv[2];
if (!version) {
  throw new Error("Usage: build-binaries.mjs <version>");
}

const config = readReleaseConfig(rootDir);
const cliName = config.cliName;

// --- Bump Cargo.toml [package].version so the binary embeds the correct version ---
const cargoTomlPath = path.join(rootDir, "Cargo.toml");
const cargoLines = readFileSync(cargoTomlPath, "utf8").split("\n");
let inPackage = false;
let bumped = false;
for (let i = 0; i < cargoLines.length; i++) {
  const trimmed = cargoLines[i].trim();
  if (trimmed === "[package]") {
    inPackage = true;
    continue;
  }
  if (inPackage && trimmed.startsWith("[")) {
    break;
  }
  if (inPackage && /^version\s*=/.test(trimmed)) {
    cargoLines[i] = `version = "${version}"`;
    bumped = true;
    break;
  }
}
if (!bumped) {
  throw new Error("Could not find [package].version in Cargo.toml.");
}
writeFileSync(cargoTomlPath, cargoLines.join("\n"), "utf8");
console.log(`Bumped Cargo.toml version to ${version}`);

// Regenerate Cargo.lock to reflect the new version.
const lockResult = spawnSync("cargo", ["generate-lockfile"], {
  stdio: "inherit",
  shell: true,
});
if (lockResult.error) {
  throw new Error(
    `cargo generate-lockfile failed: ${lockResult.error.message}`,
  );
}
if (lockResult.status !== 0) {
  throw new Error(
    `cargo generate-lockfile failed (exit ${lockResult.status}).`,
  );
}

// --- Build all targets and create archives ---
const distDir = path.join(rootDir, "dist");
mkdirSync(distDir, { recursive: true });

const linuxAr = config.targets.some((t) => t.rustTarget.includes("linux"))
  ? resolveLinuxAr()
  : null;

for (const target of config.targets) {
  const rt = target.rustTarget;
  const isWindows = rt.includes("windows");
  const isLinux = rt.includes("linux");
  const binaryName = `${cliName}${isWindows ? ".exe" : ""}`;
  const targetDir = path.join(rootDir, "target", "release-build", rt);

  console.log(`Building ${rt}...`);
  const buildArgs = isLinux
    ? ["zigbuild", "--release", "--target", rt]
    : ["build", "--release", "--target", rt];

  // Limit parallelism for Linux targets to avoid OpenSSL build race conditions
  const buildEnv = { ...process.env, CARGO_TARGET_DIR: targetDir };
  if (isLinux) {
    buildEnv.MAKEFLAGS = "-j1";
    buildEnv.CARGO_BUILD_JOBS = "1";
    const t = rt.replace(/-/g, "_");
    buildEnv[`AR_${t}`] = linuxAr;
    buildEnv[`CARGO_TARGET_${t.toUpperCase()}_AR`] = linuxAr;
  }

  const result = spawnSync("cargo", buildArgs, {
    stdio: "inherit",
    shell: true,
    env: buildEnv,
  });
  if (result.error) {
    throw new Error(`cargo build failed for ${rt}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`cargo build failed for ${rt} (exit ${result.status}).`);
  }

  const src = path.join(targetDir, rt, "release", binaryName);
  if (!existsSync(src)) {
    throw new Error(`Built binary not found at ${src}.`);
  }

  const outDir = path.join(distDir, rt);
  mkdirSync(outDir, { recursive: true });
  copyFileSync(src, path.join(outDir, binaryName));

  const archiveName = `${cliName}-${rt}.tar.gz`;
  const archivePath = path.join(distDir, archiveName);
  const tar = spawnSync(
    "tar",
    ["-czf", archivePath, "-C", outDir, binaryName],
    { stdio: "pipe" },
  );
  if (tar.error || tar.status !== 0) {
    throw new Error(
      `tar failed for ${archiveName}: ${tar.error?.message ?? `exit ${tar.status}`}`,
    );
  }
  const archiveBuf = readFileSync(archivePath);
  const hash = createHash("sha256").update(archiveBuf).digest("hex");
  writeFileSync(`${archivePath}.sha256`, `${hash}  ${archiveName}\n`, "utf8");

  console.log(`  -> ${archivePath}`);
}

console.log(`Built ${config.targets.length} targets for v${version}.`);

// Write build provenance so recovery can verify artifact origin.
const commitSha =
  spawnSync("git", ["rev-parse", "HEAD"], {
    encoding: "utf8",
  }).stdout?.trim() ?? "unknown";
const gitTag =
  spawnSync("git", ["describe", "--tags", "--exact-match", "HEAD"], {
    encoding: "utf8",
  }).stdout?.trim() ?? null;
const repository = config.sourceRepository;
writeFileSync(
  path.join(distDir, "provenance.json"),
  JSON.stringify(
    {
      version,
      commitSha,
      gitTag,
      repository,
      timestamp: new Date().toISOString(),
    },
    null,
    2,
  ) + "\n",
  "utf8",
);
