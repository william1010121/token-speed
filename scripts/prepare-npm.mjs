// Maintainer-only: assemble npm from checksum-verified GitHub release archives.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { chmod, copyFile, mkdir, readFile, rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const packageDir = path.join(root, "npm");
const manifest = JSON.parse(await readFile(path.join(packageDir, "package.json"), "utf8"));
const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
if (!cargo.includes(`version = "${manifest.version}"`)) throw new Error("Cargo and npm versions must match");
if (!process.argv[2]) throw new Error("Usage: node scripts/prepare-npm.mjs <release-archive-directory>");
const archives = path.resolve(process.argv[2]);
const checksums = new Map((await readFile(path.join(archives, "SHA256SUMS"), "utf8")).trim().split(/\r?\n/).map(line => {
  const match = /^([a-f0-9]{64})\s+\*?(.+)$/.exec(line);
  if (!match) throw new Error("Invalid SHA256SUMS entry");
  return [match[2], match[1]];
}));
const targets = ["aarch64-apple-darwin", "x86_64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"];
for (const target of targets) {
  const name = `token-speed-v${manifest.version}-${target}.tar.gz`;
  const archive = path.join(archives, name);
  const hash = createHash("sha256").update(await readFile(archive)).digest("hex");
  if (hash !== checksums.get(name)) throw new Error(`Checksum mismatch: ${name}`);
}
await rm(path.join(packageDir, "vendor"), { recursive: true, force: true });
for (const target of targets) {
  const archive = path.join(archives, `token-speed-v${manifest.version}-${target}.tar.gz`);
  const destination = path.join(packageDir, "vendor", target);
  await mkdir(destination, { recursive: true });
  execFileSync("tar", ["-xzf", archive, "-C", destination, "token-speed"]);
  await chmod(path.join(destination, "token-speed"), 0o755);
  console.log(`Verified and staged ${target}`);
}
await chmod(path.join(packageDir, "bin", "token-speed.cjs"), 0o755);
for (const name of ["README.md", "LICENSE"]) await copyFile(path.join(root, name), path.join(packageDir, name));
