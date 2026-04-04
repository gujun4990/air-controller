import fs from "node:fs/promises";
import path from "node:path";

const nextVersion = process.argv[2]?.trim();

if (!nextVersion) {
  console.error("Usage: npm run release:sync-version -- <version>");
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(nextVersion)) {
  console.error(`Invalid version: ${nextVersion}`);
  process.exit(1);
}

const root = "/opt/air-controller";
const packageJsonPath = path.join(root, "package.json");
const tauriConfigPath = path.join(root, "src-tauri", "tauri.conf.json");

const packageJson = JSON.parse(await fs.readFile(packageJsonPath, "utf8"));
const tauriConfig = JSON.parse(await fs.readFile(tauriConfigPath, "utf8"));

packageJson.version = nextVersion;
tauriConfig.version = nextVersion;

await fs.writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);
await fs.writeFile(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`);

console.log(`Synced version to ${nextVersion}`);
