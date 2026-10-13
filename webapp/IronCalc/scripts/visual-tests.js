#!/usr/bin/env node
/**
 * Runs the WorksheetCanvas visual regression tests (tests/worksheetCanvas)
 * on the platform the goldens were generated on.
 *
 * The goldens are rendered on linux/amd64 (what the CI runs). Text
 * anti-aliasing differs on other platforms, so there the tests run inside a
 * linux/amd64 Docker container; on Linux x86_64 they run natively.
 *
 *   node scripts/visual-tests.js            compare with the goldens
 *   node scripts/visual-tests.js --update   regenerate the goldens
 *
 * Both are exposed as `npm run test:visual` and `npm run test:visual:update`.
 */

import { spawnSync } from "node:child_process";
import { arch, platform } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const update = process.argv.includes("--update");
const vitestArgs = ["vitest", "run", "tests/worksheetCanvas"];

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.error) {
    console.error(`Could not run ${command}: ${result.error.message}`);
    process.exit(1);
  }
  process.exit(result.status ?? 1);
}

const action = update ? "regenerating the goldens" : "running the visual tests";

if (platform() === "linux" && arch() === "x64") {
  console.log(`[visual-tests] linux/x64: ${action} natively`);
  run("npx", vitestArgs, {
    env: { ...process.env, ...(update ? { UPDATE_SCREENSHOTS: "1" } : {}) },
  });
}

// Any other platform: same command, inside a linux/amd64 container.
console.log(
  `[visual-tests] ${platform()}/${arch()} is not linux/x64, which the goldens ` +
    `are rendered on: ${action} in a linux/amd64 Docker container`,
);
const docker = spawnSync("docker", ["info"], { stdio: "ignore" });
if (docker.error) {
  console.error(
    "[visual-tests] Docker is not installed (or not in PATH). The visual " +
      "tests need Docker on this platform: install it from " +
      "https://docs.docker.com/get-docker/ and try again.",
  );
  process.exit(1);
}
if (docker.status !== 0) {
  console.error(
    "[visual-tests] Docker is installed but the daemon is not running. " +
      "Start Docker and try again.",
  );
  process.exit(1);
}

// The container mounts the whole repo: node_modules/@ironcalc/wasm is a
// symlink into bindings/wasm/pkg. The package's own node_modules is hidden
// behind a named volume so the Linux native modules (@napi-rs/canvas) live
// there and the host install is untouched.
const repoRoot = resolve(fileURLToPath(import.meta.url), "../../../..");
const workdir = "/work/webapp/IronCalc";
run("docker", [
  "run",
  "--rm",
  "--platform",
  "linux/amd64",
  "-v",
  `${repoRoot}:/work`,
  "-v",
  `ironcalc-visual-tests-node_modules:${workdir}/node_modules`,
  "-w",
  workdir,
  ...(update ? ["-e", "UPDATE_SCREENSHOTS=1"] : []),
  ...(process.env.CI ? ["-e", "CI=1"] : []),
  "node:24",
  "sh",
  "-c",
  `npm install --no-audit --no-fund --loglevel=error && npx ${vitestArgs.join(" ")}`,
]);
