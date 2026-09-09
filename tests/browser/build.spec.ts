import { test, expect } from "@playwright/test";

test("real worker executes compiled Rust for independently worked cases", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("Wasm result: 42");
  await page.screenshot({ path: "test-results/t00-shell.png", fullPage: true });
  for (const [left, right, expected] of [
    ["0", "0", "0"],
    ["123", "456", "579"],
    ["4294967295", "1", "0"],
    ["4294967295", "4294967295", "4294967294"],
    ...Array.from({ length: 12 }, () => {
      const values = crypto.getRandomValues(new Uint32Array(2));
      return [
        String(values[0]),
        String(values[1]),
        String((BigInt(values[0]) + BigInt(values[1])) % 4294967296n),
      ];
    }),
  ]) {
    await page.getByLabel("Left operand").fill(left);
    await page.getByLabel("Right operand").fill(right);
    await page.getByRole("button", { name: "Run build probe" }).click();
    await expect(
      page.getByRole("status"),
      "operands " + left + ", " + right,
    ).toHaveText("Wasm result: " + expected);
  }
});

test("negative control: absent Wasm visibly fails", async ({ page }) => {
  let intercepted = 0;
  await page.route("**/*.wasm", (route) => {
    intercepted++;
    return route.abort();
  });
  await page.goto("/");
  await expect(page.getByRole("status")).toContainText("Build probe failed:");
  await expect(page.getByRole("status")).not.toContainText("Wasm result:");
  expect(intercepted).toBeGreaterThan(0);
  await page.unroute("**/*.wasm");
  await page.reload();
  await expect(page.getByRole("status")).toHaveText("Wasm result: 42");
});

test("out-of-range input is rejected", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("Wasm result: 42");
  await page.getByLabel("Left operand").fill("4294967296");
  await page.getByRole("button", { name: "Run build probe" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "Enter two unsigned 32-bit integers.",
  );
});
