<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { CURRENCY_SYMBOL, formatAmount, formatQuantity } from "../lib/format";
  import type { Entry, Product } from "../lib/types";

  let entries: Entry[] = [];
  let products: Record<string, Product> = {};
  let loading = true;
  let error = "";

  onMount(async () => {
    const user = requireAuth("exploitation");
    if (!user || !user.exploitation_id) return;

    try {
      const [entryList, productList] = await Promise.all([
        api.listEntries(user.exploitation_id),
        api.listProducts(),
      ]);
      entries = entryList.sort((a, b) => b.mois.localeCompare(a.mois));
      products = Object.fromEntries(productList.map((p) => [p.id, p]));
    } catch {
      error = "Impossible de charger l'historique.";
    } finally {
      loading = false;
    }
  });
</script>

<h2>Historique</h2>

{#if loading}
  <p>Chargement...</p>
{:else if error}
  <p class="error">{error}</p>
{:else if entries.length === 0}
  <p>Aucune saisie pour le moment.</p>
{:else}
  <table>
    <thead>
      <tr>
        <th>Mois</th>
        <th>Produit</th>
        <th>Qté</th>
        <th>Coût</th>
      </tr>
    </thead>
    <tbody>
      {#each entries as entry (entry.id)}
        <tr>
          <td>{entry.mois}</td>
          <td>{products[entry.product_id]?.nom ?? "?"}</td>
          <td>{formatQuantity(entry.quantite)} {products[entry.product_id]?.unite ?? ""}</td>
          <td>{formatAmount(entry.cout)} {CURRENCY_SYMBOL}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
