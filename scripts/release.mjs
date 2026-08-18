#!/usr/bin/env node
// One-shot release: bump version -> tauri build (incl. signed updater artifacts) -> commit+tag -> gh release.
// Usage:
//   $env:GH_TOKEN="..."
//   npm run release -- 2.0.1
// (.tauri-keys/bejaclient_updater.key has no passphrase, so TAURI_SIGNING_PRIVATE_KEY_PASSWORD is not needed.)

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { generateLatestYml } from "./generate-latest-yml.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const newVersion = process.argv[2];

if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error("Usage: npm run release -- <version>  (e.g. npm run release -- 2.0.1)");
  process.exit(1);
}
if (!process.env.GH_TOKEN && !process.env.GITHUB_TOKEN) {
  console.error("GH_TOKEN ist nicht gesetzt. $env:GH_TOKEN=\"...\"");
  process.exit(1);
}
if (!process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
  process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "";
}

const keyPath = path.join(root, ".tauri-keys/bejaclient_updater.key");
if (!existsSync(keyPath)) {
  console.error(`Signing key nicht gefunden: ${keyPath}`);
  process.exit(1);
}

function run(cmd) {
  console.log(`\n$ ${cmd}`);
  execSync(cmd, { stdio: "inherit", cwd: root, env: process.env });
}

function bumpJsonVersion(relPath) {
  const p = path.join(root, relPath);
  const json = JSON.parse(readFileSync(p, "utf8"));
  json.version = newVersion;
  writeFileSync(p, JSON.stringify(json, null, 2) + "\n");
}

console.log(`=== Release v${newVersion} ===`);

bumpJsonVersion("package.json");
bumpJsonVersion("src-tauri/tauri.conf.json");

const cargoPath = path.join(root, "src-tauri/Cargo.toml");
const cargoToml = readFileSync(cargoPath, "utf8").replace(/^version = ".*"$/m, `version = "${newVersion}"`);
writeFileSync(cargoPath, cargoToml);

process.env.TAURI_SIGNING_PRIVATE_KEY = readFileSync(keyPath, "utf8");

run("npm run tauri:build");

const installer = path.join(root, "src-tauri/target/release/bundle/nsis", `BejaClient_${newVersion}_x64-setup.exe`);
if (!existsSync(installer)) {
  console.error(`Installer nicht gefunden: ${installer}`);
  process.exit(1);
}

// Also emit an electron-builder-compatible latest.yml pointing at this installer, so
// existing Electron clients (electron-updater, GitHub provider) pick this release up as
// a normal update and silently install it — see generate-latest-yml.mjs and
// BejaClient-ML/src/main/services/updaterService.ts. Harmless/unused for Tauri clients,
// which use their own signed updater manifest via the api.bejaclient.xyz endpoint instead.
const latestYml = generateLatestYml(installer, newVersion, path.dirname(installer));
console.log(`latest.yml geschrieben: ${latestYml}`);

run(`git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock`);
run(`git commit -m "chore: release v${newVersion}"`);
run(`git tag v${newVersion}`);
run(`git pull --rebase origin main`);
run(`git push origin main`);
run(`git push origin v${newVersion}`);
run(`gh release create v${newVersion} "${installer}" "${latestYml}" --title "v${newVersion}" --generate-notes`);

console.log(`\nRelease v${newVersion} veroeffentlicht.`);
