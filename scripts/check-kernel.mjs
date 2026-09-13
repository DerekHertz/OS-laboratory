import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = new URL("../", import.meta.url);
const directory = new URL(".verification/", root);
const output = new URL("kernel-events.json", directory);
mkdirSync(directory, { recursive: true });
rmSync(output, { force: true });
function run(command, args, env = process.env) {
  const result = spawnSync(command, args, {
    cwd: fileURLToPath(root),
    env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run("cargo", ["test", "--workspace", "--locked"], {
  ...process.env,
  OS_LAB_KERNEL_EVENTS: fileURLToPath(output),
});
run(process.execPath, [
  fileURLToPath(new URL("check-kernel-events.mjs", import.meta.url)),
  fileURLToPath(output),
]);
