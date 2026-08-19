import { expect, test } from "@playwright/test";
import { login, logout } from "../support/actions";
import { ADMIN_EMAIL, ADMIN_PASSWORD, unique } from "../support/env";

/**
 * The golden path: everything a cooperative admin and one of its member
 * exploitations do together, in the order they'd actually do it. This is
 * the main "film" of e2e/README.md's living documentation - one continuous
 * video showing catalogue setup, a farmer's monthly declarations, and both
 * dashboards reacting to that data.
 */

function currentMonth(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

test("cooperative admin onboards a member, who declares costs and production", async ({
  page,
}) => {
  const productName = unique("Mais");
  const exploitationName = unique("Ferme du Vallon");
  const exploitationEmail = `${unique("ferme")}@elevia.test`;
  const exploitationPassword = "TestPass123!";
  const month = currentMonth();

  await test.step("admin logs in", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
    await expect(page).toHaveURL(/\/admin\/exploitations$/);
    await expect(page.getByRole("heading", { name: "Exploitations" })).toBeVisible();
  });

  await test.step("admin adds a product to the catalogue", async () => {
    await page.getByRole("link", { name: "Produits" }).click();
    await expect(page).toHaveURL(/\/admin\/products$/);

    await page.getByLabel("Nom").fill(productName);
    await page.getByLabel("Unité").fill("kg");
    await page.getByLabel("Catégorie").fill("intrant");
    await page.getByRole("button", { name: "Créer" }).click();

    await expect(page.getByText(`${productName} — kg — intrant`)).toBeVisible();
  });

  await test.step("admin corrects the product's category", async () => {
    // data-testid instead of hasText: once "Modifier" is clicked, the
    // product's name only lives in an input's value, not in text content,
    // so a text-based filter would stop matching its own card mid-edit.
    const productCard = page.getByTestId(`product-card-${productName}`);
    await productCard.getByRole("button", { name: "Modifier" }).click();
    await productCard.getByLabel("Catégorie").fill("engrais");
    await productCard.getByRole("button", { name: "Sauvegarder" }).click();

    await expect(page.getByText(`${productName} — kg — engrais`)).toBeVisible();
  });

  await test.step("admin registers a new member exploitation", async () => {
    await page.getByRole("link", { name: "Exploitations" }).click();
    await expect(page).toHaveURL(/\/admin\/exploitations$/);

    await page.getByLabel("Nom").fill(exploitationName);
    await page.getByLabel("Contact").fill("Jean Dupont");
    await page.getByLabel("Email").fill(exploitationEmail);
    await page.getByLabel("Mot de passe").fill(exploitationPassword);
    await page.getByRole("button", { name: "Créer" }).click();

    const row = page.locator("tr", { hasText: exploitationName });
    await expect(row).toBeVisible();
    await expect(row.getByRole("cell", { name: "—" }).first()).toBeVisible();
  });

  await test.step("admin signs out", async () => {
    await logout(page);
  });

  await test.step("the new member logs in and lands on their cost entry screen", async () => {
    await login(page, exploitationEmail, exploitationPassword);
    await expect(page).toHaveURL(/\/entries$/);
    await expect(page.getByRole("heading", { name: "Saisir un coût" })).toBeVisible();
  });

  await test.step("member declares a monthly input cost", async () => {
    await page.getByLabel("Produit / intrant").selectOption({ label: `${productName} (kg)` });
    await page.getByLabel("Quantité").fill("20");
    await page.getByLabel("Coût total (FCFA)").fill("150");
    await page.getByLabel("Mois").fill(month);
    await page.getByRole("button", { name: "Enregistrer" }).click();

    await expect(page.getByText("Coût enregistré.")).toBeVisible();
    const historyRow = page.locator("tr", { hasText: productName });
    await expect(historyRow).toBeVisible();
    // Coût column (Mois, Produit, Qté, Coût, then the delete button cell) -
    // scoped to that column, not a loose text search, since the product's
    // random suffix can itself contain "150". The backend renders the
    // decimal with its own precision, so match loosely.
    await expect(historyRow.getByRole("cell").nth(3)).toHaveText(/^150(\.\d+)? FCFA$/);
  });

  await test.step("member declares monthly production", async () => {
    await page.getByRole("link", { name: "Production" }).click();
    await expect(page).toHaveURL(/\/production$/);

    await page.getByLabel("Qu'est-ce qui a été produit ?").fill("Blé");
    await page.getByLabel("Quantité produite").fill("10");
    await page.getByLabel("Unité").fill("tonnes");
    await page.getByLabel("Quantité vendue").fill("10");
    await page.getByLabel("Prix de vente unitaire").fill("50");
    await page.getByLabel("Mois").fill(month);
    await page.getByRole("button", { name: "Enregistrer" }).click();

    await expect(page.getByText("Production enregistrée.")).toBeVisible();
  });

  await test.step("member sees their own dashboard reflect both declarations", async () => {
    await page.getByRole("link", { name: "Mon dashboard" }).click();
    await expect(page).toHaveURL(/\/dashboard$/);

    const monthCard = page.locator(".card", { hasText: month });
    const costStat = monthCard.locator(".stat", { hasText: "Coût total" });
    await expect(costStat.locator(".value")).toHaveText(/^150(\.\d+)? FCFA$/);
    const producedStat = monthCard.locator(".stat", { hasText: "Blé" });
    await expect(producedStat.locator(".value")).toHaveText(/^10(\.\d+)? tonnes$/);
  });

  await test.step("member signs out", async () => {
    await logout(page);
  });

  await test.step("admin sees the member's submission status flip to done", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);
    await expect(page).toHaveURL(/\/admin\/exploitations$/);

    const row = page.locator("tr", { hasText: exploitationName });
    await expect(row.getByRole("cell", { name: "✓" }).first()).toBeVisible();
  });

  await test.step("admin reviews the cooperative-wide dashboard", async () => {
    await page.getByRole("link", { name: "Coopérative" }).click();
    await expect(page).toHaveURL(/\/coop$/);

    await expect(
      page.getByRole("heading", { name: /Coopérative · \d{4}-\d{2}/ }),
    ).toBeVisible();
    const needRow = page.locator("tr", { hasText: productName });
    await expect(needRow).toBeVisible();
    await expect(page.getByText("Marge moyenne")).toBeVisible();
    await expect(page.getByText("Q1")).toBeVisible();
  });
});
