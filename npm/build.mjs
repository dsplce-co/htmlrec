#!/usr/bin/env node
// Builds and publishes the npm distribution of htmlrec from the prebuilt
// release tarballs produced by the CI `build` job. Run from the repo root:
//
//   node npm/build.mjs <version> <artifactsDir>
//
// It creates one platform package per supported target (each shipping the
// `hrec` binary, selected at install time via the package's os/cpu fields) and
// a main `htmlrec` package whose JS launcher resolves the matching platform
// package. Platform packages are published first so the main package's
// optionalDependencies already exist on the registry.
//
// Set DRY_RUN=1 to `npm pack` into npm/.staging/_packs instead of publishing.

import { spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// triple -> npm os/cpu tokens. Extend alongside the CI build matrix.
const TARGETS = {
    "x86_64-unknown-linux-gnu": { os: "linux", cpu: "x64" },
    "x86_64-apple-darwin": { os: "darwin", cpu: "x64" },
    "aarch64-apple-darwin": { os: "darwin", cpu: "arm64" },
};

const DRY_RUN = !!process.env.DRY_RUN;
const NPM_DIR = dirname(fileURLToPath(import.meta.url));
const MAIN_SRC = join(NPM_DIR, "htmlrec");
const ROOT_README = join(NPM_DIR, "..", "README.md");
const STAGING = join(NPM_DIR, ".staging");
const PACK_OUT = join(STAGING, "_packs");
const REPO = { type: "git", url: "git+https://github.com/dsplce-co/htmlrec.git" };
const LICENSE = "MIT OR Apache-2.0";

function fail(msg) {
    console.error(`build.mjs: ${msg}`);
    process.exit(1);
}

function run(cmd, args, opts = {}) {
    return spawnSync(cmd, args, { encoding: "utf8", ...opts });
}

// True only when this exact name@version is already on the registry.
function alreadyPublished(name, version) {
    const r = run("npm", ["view", `${name}@${version}`, "version"]);
    return r.status === 0 && r.stdout.trim() === version;
}

function release(name, pkgDir) {
    if (DRY_RUN) {
        mkdirSync(PACK_OUT, { recursive: true });
        const r = run("npm", ["pack", "--pack-destination", PACK_OUT], { cwd: pkgDir, stdio: "inherit" });
        if (r.status !== 0) fail(`npm pack failed for ${name}`);
        console.log(`  packed ${name}`);
        return;
    }
    if (alreadyPublished(name, version)) {
        console.log(`  skip ${name}@${version} (already on registry)`);
        return;
    }
    const r = run("npm", ["publish", "--access", "public", "--provenance"], { cwd: pkgDir, stdio: "inherit" });
    if (r.status !== 0) fail(`npm publish failed for ${name}`);
    console.log(`  published ${name}@${version}`);
}

const [, , version, artifactsDir] = process.argv;
if (!version || !artifactsDir) fail("usage: node npm/build.mjs <version> <artifactsDir>");
if (!existsSync(artifactsDir)) fail(`artifacts dir not found: ${artifactsDir}`);

rmSync(STAGING, { recursive: true, force: true });
mkdirSync(STAGING, { recursive: true });

// Verify every expected tarball is present before publishing anything, so we
// never ship a partial platform set.
const targets = Object.entries(TARGETS).map(([triple, { os, cpu }]) => {
    const tarball = join(artifactsDir, `hrec-${triple}.tar.gz`);
    return { triple, os, cpu, tarball, pkg: `htmlrec-${os}-${cpu}` };
});

const missing = targets.filter((t) => !existsSync(t.tarball)).map((t) => t.tarball);
if (missing.length) fail(`missing release tarball(s):\n  ${missing.join("\n  ")}`);

// Platform packages first.
console.log(`Building ${targets.length} platform package(s) at version ${version}${DRY_RUN ? " (dry run)" : ""}:`);
for (const t of targets) {
    const pkgDir = join(STAGING, t.pkg);
    const binDir = join(pkgDir, "bin");
    mkdirSync(binDir, { recursive: true });

    const x = run("tar", ["-xzf", t.tarball, "-C", binDir], { stdio: "inherit" });
    if (x.status !== 0) fail(`failed to extract ${t.tarball}`);

    const bin = join(binDir, "hrec");
    if (!existsSync(bin)) fail(`extracted archive ${t.tarball} did not contain "hrec"`);
    chmodSync(bin, 0o755); // extraction/umask can drop the exec bit

    writeFileSync(
        join(pkgDir, "package.json"),
        JSON.stringify(
            {
                name: t.pkg,
                version,
                description: `htmlrec native binary (${t.os}-${t.cpu})`,
                os: [t.os],
                cpu: [t.cpu],
                files: ["bin/hrec"],
                license: LICENSE,
                homepage: "https://github.com/dsplce-co/htmlrec",
                repository: REPO,
            },
            null,
            2,
        ) + "\n",
    );
    release(t.pkg, pkgDir);
}

// Main package last; its optionalDependencies must already exist on the registry.
console.log("Building main package htmlrec:");
const mainDir = join(STAGING, "htmlrec");

mkdirSync(join(mainDir, "bin"), { recursive: true });
copyFileSync(join(MAIN_SRC, "bin", "hrec.cjs"), join(mainDir, "bin", "hrec.cjs"));
copyFileSync(ROOT_README, join(mainDir, "README.md")); // reuse the repo's canonical README

const mainPkg = JSON.parse(readFileSync(join(MAIN_SRC, "package.json"), "utf8"));
mainPkg.version = version;
mainPkg.optionalDependencies = Object.fromEntries(targets.map((t) => [t.pkg, version])); // exact pin
writeFileSync(join(mainDir, "package.json"), JSON.stringify(mainPkg, null, 2) + "\n");

release("htmlrec", mainDir);

console.log(DRY_RUN ? "Dry run complete." : "Publish complete.");
