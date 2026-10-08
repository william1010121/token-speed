import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { realpathSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

const target = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
}[`${process.platform}-${process.arch}`];

async function fixture(t, body) {
  const root = await mkdtemp(path.join(os.tmpdir(), "token-speed npm "));
  t.after(() => rm(root, { recursive: true, force: true }));
  const bin = path.join(root, "bin");
  const vendor = path.join(root, "vendor", target);
  await mkdir(bin, { recursive: true });
  await mkdir(vendor, { recursive: true });
  const launcher = path.join(bin, "token-speed.cjs");
  await copyFile(new URL("../npm/bin/token-speed.cjs", import.meta.url), launcher);
  if (body !== null) await writeFile(path.join(vendor, "token-speed"), `#!/usr/bin/env node\n${body}`, { mode: 0o755 });
  return { root, launcher };
}

test("launcher preserves arguments, cwd, stdout, stderr and exit status", async t => {
  const { root, launcher } = await fixture(t, `console.log(JSON.stringify({args:process.argv.slice(2),cwd:process.cwd()})); console.error("synthetic stderr"); process.exitCode=7;`);
  const args = ["--model", "space and $shell;chars", "--json"];
  const result = spawnSync(process.execPath, [launcher, ...args], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 7);
  assert.deepEqual(JSON.parse(result.stdout), { args, cwd: realpathSync(root) });
  assert.equal(result.stderr.trim(), "synthetic stderr");
});

test("missing binary reports an actionable error", async t => {
  const { launcher } = await fixture(t, null);
  const result = spawnSync(process.execPath, [launcher], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /cannot run native binary/);
  assert.match(result.stderr, /reinstalling or building from source/);
});

test("SIGTERM reaches the child so it can clean up", { timeout: 5000 }, async t => {
  const { launcher } = await fixture(t, `process.on("SIGTERM",()=>{console.log("cleaned up");process.exit(42)});console.log("ready");setInterval(()=>{},1000);`);
  const child = spawn(process.execPath, [launcher], { stdio: ["ignore", "pipe", "pipe"] });
  t.after(() => { if (child.exitCode === null) child.kill("SIGTERM"); });
  let stdout = "";
  const exit = new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("exit", (code, signal) => resolve({ code, signal }));
  });
  child.stdout.on("data", chunk => {
    stdout += chunk;
    if (stdout.trim() === "ready") child.kill("SIGTERM");
  });
  assert.deepEqual(await exit, { code: 42, signal: null });
  assert.match(stdout, /cleaned up/);
});
