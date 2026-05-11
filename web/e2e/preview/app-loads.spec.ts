import { expect, test } from "@playwright/test";

test.describe("preview shell", () => {
  test("loads the SPA and exposes the app root", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("app-root")).toBeVisible();
    await expect(page.getByRole("heading", { name: /furnigen/i })).toBeVisible();
    await expect(page.getByTestId("wasm-status")).toContainText("sample depth 600 mm validated", {
      timeout: 30_000,
    });
    await expect(
      page.getByTestId("preview-canvas").or(page.getByTestId("preview-gl-fallback"))
    ).toBeVisible({ timeout: 30_000 });
  });
});
