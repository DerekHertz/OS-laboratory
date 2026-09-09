import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
const result = spawnSync(
  "cargo",
  [
    "build",
    "-p",
    "sim-wasm",
    "--target",
    "wasm32-unknown-unknown",
    "--release",
    "--locked",
    "--target-dir",
    "target",
  ],
  { stdio: "inherit" },
);
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
mkdirSync("apps/web/src/generated", { recursive: true });
copyFileSync(
  "target/wasm32-unknown-unknown/release/sim_wasm.wasm",
  "apps/web/src/generated/sim_wasm.wasm",
);
