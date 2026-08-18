import { expect, test } from "@playwright/test";
import { login } from "../support/actions";
import { ADMIN_EMAIL, ADMIN_PASSWORD, unique } from "../support/env";

/**
 * Elevia is an offline-first PWA for farmers who may not have signal in the
 * field: a submission made offline must queue in IndexedDB instead of
 * failing, then flush automatically once the connection is back.
 */

function currentMonth(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

test("a cost entry submitted offline is queued locally, then synced on reconnect", async ({
  page,
  context,
}) => {
  const productName = unique("Gasoil");
  const exploitationName = unique("Ferme Hors Ligne");
  const exploitationEmail = `${unique("horsligne")}@elevia.test`;
  const exploitationPassword = "TestPass123!";

  await test.step("admin prepares a product and a member exploitation", async () => {
    await login(page, ADMIN_EMAIL, ADMIN_PASSWORD);

    await page.getByRole("link", { name: "Produits" }).click();
    await page.getByLabel("Nom").fill(productName);
    await page.getByLabel("Unité").fill("L");
    await page.getByLabel("Catégorie").fill("intrant");
    await page.getByRole("button", { name: "Créer" }).click();
    await expect(page.getByText(`${productName} — L — intrant`)).toBeVisible();

    await page.getByRole("link", { name: "Exploitations" }).click();
    await page.getByLabel("Nom").fill(exploitationName);
    await page.getByLabel("Contact").fill("Test");
    await page.getByLabel("Email").fill(exploitationEmail);
    await page.getByLabel("Mot de passe").fill(exploitationPassword);
    await page.getByRole("button", { name: "Créer" }).click();
    await expect(page.locator("tr", { hasText: exploitationName })).toBeVisible();

    await page.getByRole("button", { name: "Déconnexion" }).click();
  });

  await test.step("member logs in while still online", async () => {
    await login(page, exploitationEmail, exploitationPassword);
    await expect(page).toHaveURL(/\/entries$/);
  });

  await test.step("member submits a cost entry with no network", async () => {
    await context.setOffline(true);

    await page.getByLabel("Produit / intrant").selectOption({ label: `${productName} (L)` });
    await page.getByLabel("Quantité").fill("40");
    await page.getByLabel("Coût (€)").fill("80");
    await page.getByLabel("Mois").fill(currentMonth());
    await page.getByRole("button", { name: "Enregistrer" }).click();

    await expect(
      page.getByText("Hors ligne : enregistré localement, sera envoyé à la reconnexion."),
    ).toBeVisible();
  });

  await test.step("the entry sits in the local queue while offline", async () => {
    const pending = await page.evaluate(async () => {
      const { pendingCount } = await import("/src/lib/offlineQueue.ts");
      return pendingCount();
    });
    expect(pending).toBeGreaterThanOrEqual(1);
  });

  await test.step("reconnecting flushes the queue to the backend", async () => {
    const syncRequest = page.waitForRequest(
      (req) => req.method() === "POST" && req.url().includes("/entries"),
    );
    await context.setOffline(false);
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    await syncRequest;

    await expect
      .poll(async () =>
        page.evaluate(async () => {
          const { pendingCount } = await import("/src/lib/offlineQueue.ts");
          return pendingCount();
        }),
      )
      .toBe(0);
  });

  await test.step("the synced entry now shows up in the member's history", async () => {
    await page.reload();
    const historyRow = page.locator("tr", { hasText: productName });
    await expect(historyRow).toBeVisible();
    // Last cell = Coût, scoped to that column: the product's random suffix
    // can itself contain "80", so a loose text search would be ambiguous.
    await expect(historyRow.getByRole("cell").last()).toHaveText(/^80(\.\d+)? €$/);
  });
});
