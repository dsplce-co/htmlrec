#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");

const KEY = `${process.platform}-${process.arch}`;
const PKG = {
    "linux-x64": "htmlrec-linux-x64",
    "darwin-x64": "htmlrec-darwin-x64",
    "darwin-arm64": "htmlrec-darwin-arm64",
}[KEY];

if (!PKG) {
    console.error(`htmlrec: unsupported platform "${KEY}".`);
    console.error("Supported: linux-x64, darwin-x64, darwin-arm64.");
    console.error("On other platforms, install via `cargo install htmlrec`.");
    process.exit(1);
}

let bin;
try {
    bin = require.resolve(`${PKG}/bin/hrec`);
} catch {
    console.error(`htmlrec: native package "${PKG}" is not installed.`);
    console.error("Reinstall without --ignore-scripts / --no-optional, or run `npm i -g htmlrec`.");
    process.exit(1);
}

const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) {
    console.error(`htmlrec: failed to launch hrec: ${r.error.message}`);
    process.exit(1);
}
if (r.signal) {
    process.kill(process.pid, r.signal);
    process.exit(1);
}
process.exit(r.status ?? 1);
