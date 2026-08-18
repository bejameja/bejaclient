#!/usr/bin/env node
// Generates an electron-builder-compatible latest.yml for a Tauri NSIS installer, so the
// existing Electron client's auto-updater (electron-updater, GitHub provider) can discover
// this Tauri release and silently install it as if it were a normal Electron update.
// See release.mjs and BejaClient-ML/src/main/services/updaterService.ts.
//
// Usage: node scripts/generate-latest-yml.mjs <installer.exe> <version> [outDir]

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

export function generateLatestYml(installerPath, version, outDir) {
  const buf = readFileSync(installerPath);
  const sha512 = createHash("sha512").update(buf).digest("base64");
  const size = buf.length;
  const fileName = path.basename(installerPath);
  const targetDir = outDir || path.dirname(installerPath);
  const outPath = path.join(targetDir, "latest.yml");

  const yml = `version: ${version}
files:
  - url: ${fileName}
    sha512: ${sha512}
    size: ${size}
path: ${fileName}
sha512: ${sha512}
releaseDate: '${new Date().toISOString()}'
`;

  writeFileSync(outPath, yml);
  return outPath;
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
if (isMain) {
  const [, , installerPath, version, outDirArg] = process.argv;
  if (!installerPath || !version) {
    console.error("Usage: node scripts/generate-latest-yml.mjs <installer.exe> <version> [outDir]");
    process.exit(1);
  }
  const outPath = generateLatestYml(installerPath, version, outDirArg);
  console.log(`latest.yml geschrieben: ${outPath}`);
}
