import { expect, test } from "@playwright/test";
import { login, logout } from "../support/actions";
import { ADMIN_EMAIL, ADMIN_PASSWORD, unique } from "../support/env";

/**
 * Two ways a password gets changed: self-service from /account (you know
 * your current password) and an admin reset from the exploitation list
 * (you don't - forgotten password). Both journeys only count as done once
 * the new password actually works at /login, not just "the API answered
 * 204".
 */

test("a member changes their own password from /account and logs back in with it", async ({
  page,
}) => {
  const exploitationName = unique("Ferme Mot De Passe");
  const exploitationEmail = `${unique("motdepasse")}@elevia.test`;
  const oldPassword = "TestPass123!";
  const newPassword = "NouveauPass456!";

  await test.step("admin provisions a throwaway member account", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
    await page.getByLabel("Nom").fill(exploitationName);
    await page.getByLabel("Contact").fill("Test");
    await page.getByLabel("Email").fill(exploitationEmail);
    await page.getByLabel("Mot de passe").fill(oldPassword);
    await page.getByRole("button", { name: "Créer" }).click();
    await expect(page.locator("tr", { hasText: exploitationName })).toBeVisible();
    await logout(page);
  });

  await test.step("the member changes their password from Mon compte", async () => {
    await login(page, exploitationEmail, oldPassword);
    await expect(page).toHaveURL(/\/entries$/);

    await page.getByRole("link", { name: "Mon compte" }).click();
    await page.getByLabel("Mot de passe actuel").fill(oldPassword);
    await page.getByLabel("Nouveau mot de passe", { exact: true }).fill(newPassword);
    await page.getByLabel("Confirmer le nouveau mot de passe").fill(newPassword);
    await page.getByRole("button", { name: "Changer le mot de passe" }).click();

    await expect(page.getByText("Mot de passe mis à jour.")).toBeVisible();
    await logout(page);
  });

  await test.step("the old password no longer works, the new one does", async () => {
    await login(page, exploitationEmail, oldPassword);
    await expect(page.getByText(/invalid email or password/i)).toBeVisible();
    await expect(page).toHaveURL(/\/login$/);

    await login(page, exploitationEmail, newPassword);
    await expect(page).toHaveURL(/\/entries$/);
  });
});

test("an admin resets a member's password so they can log back in without the old one", async ({
  page,
}) => {
  const exploitationName = unique("Ferme Oubli");
  const exploitationEmail = `${unique("oubli")}@elevia.test`;
  const forgottenPassword = "OldForgotten123!";
  const resetPassword = "ResetByAdmin789!";

  await test.step("admin provisions a throwaway member account", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
    await page.getByLabel("Nom").fill(exploitationName);
    await page.getByLabel("Contact").fill("Test");
    await page.getByLabel("Email").fill(exploitationEmail);
    await page.getByLabel("Mot de passe").fill(forgottenPassword);
    await page.getByRole("button", { name: "Créer" }).click();
    await expect(page.locator("tr", { hasText: exploitationName })).toBeVisible();
  });

  await test.step("admin resets that member's password from the exploitation list", async () => {
    const row = page.locator("tr", { hasText: exploitationName });
    await row.getByRole("button", { name: "Réinitialiser" }).click();
    await row.getByPlaceholder("Nouveau mot de passe").fill(resetPassword);
    await row.getByRole("button", { name: "Valider" }).click();

    await expect(row.getByText("Mis à jour")).toBeVisible();
  });

  await test.step("the member logs in with the reset password, not the forgotten one", async () => {
    await logout(page);

    await login(page, exploitationEmail, resetPassword);
    await expect(page).toHaveURL(/\/entries$/);
  });
});
