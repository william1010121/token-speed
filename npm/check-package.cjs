"use strict";
const { accessSync, constants, statSync } = require("node:fs");
const path = require("node:path");
for (const target of ["aarch64-apple-darwin", "x86_64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"]) {
  const binary = path.join(__dirname, "vendor", target, "token-speed");
  accessSync(binary, constants.X_OK);
  if (!statSync(binary).isFile()) throw new Error(`Missing binary: ${target}`);
}
for (const name of ["README.md", "LICENSE"]) accessSync(path.join(__dirname, name));
console.log(`All four native binaries ready for token-speed@${require("./package.json").version}`);
