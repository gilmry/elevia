<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { onEntriesChanged } from "../lib/events";
  import { CURRENCY_SYMBOL, formatAmount, formatQuantity } from "../lib/format";
  import type { Entry, Product } from "../lib/types";

  let exploitationId = "";
  let entries: Entry[] = [];
  let products: Record<string, Product> = {};
  let loading = true;
  let error = "";
  let deletingId: string | null = null;

  async function load() {
    loading = true;
    try {
      const [entryList, productList] = await Promise.all([
        api.listEntries(exploitationId),
        api.listProducts(),
      ]);
      entries = entryList.sort((a, b) => b.mois.localeCompare(a.mois));
      products = Object.fromEntries(productList.map((p) => [p.id, p]));
    } catch {
      error = "Impossible de charger l'historique.";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    const user = requireAuth("exploitation");
    if (!user || !user.exploitation_id) return;
    exploitationId = user.exploitation_id;
    load();
    return onEntriesChanged(load);
  });

  async function removeEntry(entry: Entry) {
    if (!confirm(`Supprimer la saisie "${products[entry.product_id]?.nom ?? "?"}" (${entry.mois}) ?`)) {
      return;
    }
    deletingId = entry.id;
    error = "";
    try {
      await api.deleteEntry(exploitationId, entry.id);
      entries = entries.filter((e) => e.id !== entry.id);
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Impossible de supprimer cette saisie.";
    } finally {
      deletingId = null;
    }
  }
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
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each entries as entry (entry.id)}
        <tr>
          <td>{entry.mois}</td>
          <td>{products[entry.product_id]?.nom ?? "?"}</td>
          <td>{formatQuantity(entry.quantite)} {products[entry.product_id]?.unite ?? ""}</td>
          <td>{formatAmount(entry.cout)} {CURRENCY_SYMBOL}</td>
          <td>
            <button
              class="secondary compact"
              on:click={() => removeEntry(entry)}
              disabled={deletingId === entry.id}
            >
              {deletingId === entry.id ? "..." : "Supprimer"}
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if error}
    <p class="error">{error}</p>
  {/if}
{/if}

<style>
  button.compact {
    min-height: unset;
    padding: 0.4rem 0.7rem;
    font-size: 0.85rem;
    white-space: nowrap;
  }
</style>
