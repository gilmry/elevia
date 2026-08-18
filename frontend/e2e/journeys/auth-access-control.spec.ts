import { expect, test } from "@playwright/test";
import { login, logout } from "../support/actions";
import { ADMIN_EMAIL, ADMIN_PASSWORD, unique } from "../support/env";

/**
 * Everything that keeps people out of screens that aren't theirs: wrong
 * credentials, no session at all, and the wrong role for a page. These are
 * short on purpose - the golden path in coop-lifecycle.spec.ts already
 * covers the happy logins for both roles.
 */

test("rejects an unknown password with a visible error, without navigating away", async ({
  page,
}) => {
  await login(page, ADMIN_EMAIL, "not-the-right-password");

  await expect(page.getByText(/invalid email or password/i)).toBeVisible();
  await expect(page).toHaveURL(/\/login$/);
});

test("bounces an anonymous visitor from a protected page to /login", async ({ page }) => {
  await page.goto("/dashboard");

  await expect(page).toHaveURL(/\/login$/);
});

test("admin heading to an exploitation-only page is sent back to their own home", async ({
  page,
}) => {
  await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
  await expect(page).toHaveURL(/\/admin\/exploitations$/);

  await page.goto("/entries");

  await expect(page).toHaveURL(/\/admin\/exploitations$/);
});

test("member exploitation heading to the admin catalogue is sent back to their own home", async ({
  page,
}) => {
  const exploitationName = unique("Ferme Test Accès");
  const exploitationEmail = `${unique("acces")}@elevia.test`;
  const exploitationPassword = "TestPass123!";

  await test.step("admin provisions a throwaway member account", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
    await page.getByLabel("Nom").fill(exploitationName);
    await page.getByLabel("Contact").fill("Test");
    await page.getByLabel("Email").fill(exploitationEmail);
    await page.getByLabel("Mot de passe").fill(exploitationPassword);
    await page.getByRole("button", { name: "Créer" }).click();
    await expect(page.locator("tr", { hasText: exploitationName })).toBeVisible();
    await logout(page);
  });

  await test.step("that member cannot reach /admin/products", async () => {
    await login(page, exploitationEmail, exploitationPassword);
    await expect(page).toHaveURL(/\/entries$/);

    await page.goto("/admin/products");

    await expect(page).toHaveURL(/\/entries$/);
  });
});
