import { expect, test } from "@playwright/test";

test.describe("preview shell", () => {
  test("loads the SPA and exposes the app root", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("app-root")).toBeVisible();
    await expect(page.getByRole("heading", { name: /furnigen/i })).toBeVisible();
  });
});
