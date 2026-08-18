<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError, OfflineError } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { queueEntry } from "../lib/offlineQueue";
  import type { Product } from "../lib/types";

  function currentMonth(): string {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  }

  let exploitationId = "";
  let products: Product[] = [];
  let productId = "";
  let mois = currentMonth();
  let quantite = "";
  let cout = "";
  let message = "";
  let error = "";
  let loading = false;
  let loadingProducts = true;

  onMount(async () => {
    const user = requireAuth("exploitation");
    if (!user || !user.exploitation_id) return;
    exploitationId = user.exploitation_id;

    try {
      products = await api.listProducts();
    } catch {
      error = "Impossible de charger la liste des produits.";
    } finally {
      loadingProducts = false;
    }
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    message = "";
    loading = true;

    const input = { product_id: productId, mois, quantite, cout };

    try {
      await api.submitEntry(exploitationId, input);
      message = "Coût enregistré.";
      quantite = "";
      cout = "";
    } catch (err) {
      if (err instanceof OfflineError) {
        await queueEntry(exploitationId, input);
        message = "Hors ligne : enregistré localement, sera envoyé à la reconnexion.";
        quantite = "";
        cout = "";
      } else if (err instanceof ApiError) {
        error = err.message;
      } else {
        error = "Une erreur est survenue.";
      }
    } finally {
      loading = false;
    }
  }
</script>

<h1>Saisir un coût</h1>

{#if loadingProducts}
  <p>Chargement...</p>
{:else}
  <form on:submit={submit}>
    <label>
      Produit / intrant
      <select bind:value={productId} required>
        <option value="" disabled selected>Choisir...</option>
        {#each products as product (product.id)}
          <option value={product.id}>{product.nom} ({product.unite})</option>
        {/each}
      </select>
    </label>
    <label>
      Quantité
      <input
        type="number"
        step="0.001"
        min="0"
        bind:value={quantite}
        required
        inputmode="decimal"
      />
    </label>
    <label>
      Coût (€)
      <input type="number" step="0.01" min="0" bind:value={cout} required inputmode="decimal" />
    </label>
    <label>
      Mois
      <input type="month" bind:value={mois} required />
    </label>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    {#if message}
      <p class="notice">{message}</p>
    {/if}
    <button type="submit" disabled={loading}>{loading ? "Envoi..." : "Enregistrer"}</button>
  </form>
{/if}
