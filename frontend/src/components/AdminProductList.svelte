<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import type { Product } from "../lib/types";

  let products: Product[] = [];
  let loading = true;
  let error = "";

  let nom = "";
  let unite = "";
  let categorie = "";
  let creating = false;
  let createError = "";

  let editing: Record<string, { nom: string; unite: string; categorie: string }> = {};
  let savingId = "";

  async function load() {
    loading = true;
    try {
      products = await api.listProducts();
    } catch {
      error = "Impossible de charger les produits.";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    const user = requireAuth("admin");
    if (!user) return;
    load();
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    createError = "";
    creating = true;
    try {
      await api.createProduct({ nom, unite, categorie });
      nom = "";
      unite = "";
      categorie = "";
      await load();
    } catch (err) {
      createError = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      creating = false;
    }
  }

  function startEdit(product: Product) {
    editing[product.id] = { nom: product.nom, unite: product.unite, categorie: product.categorie };
    editing = editing;
  }

  function cancelEdit(id: string) {
    delete editing[id];
    editing = editing;
  }

  async function saveEdit(id: string) {
    const changes = editing[id];
    if (!changes) return;
    savingId = id;
    try {
      await api.updateProduct(id, changes);
      cancelEdit(id);
      await load();
    } catch {
      error = "Impossible de sauvegarder ce produit.";
    } finally {
      savingId = "";
    }
  }
</script>

<h1>Produits & intrants</h1>

<div class="card">
  <h2>Ajouter un produit</h2>
  <form on:submit={submit}>
    <label>Nom<input type="text" bind:value={nom} required placeholder="ex: Maïs" /></label>
    <label>Unité<input type="text" bind:value={unite} required placeholder="ex: kg" /></label>
    <label>
      Catégorie
      <input type="text" bind:value={categorie} required placeholder="ex: intrant" />
    </label>
    {#if createError}
      <p class="error">{createError}</p>
    {/if}
    <button type="submit" disabled={creating}>{creating ? "Création..." : "Créer"}</button>
  </form>
</div>

<div class="card">
  <h2>Produits existants</h2>
  {#if loading}
    <p>Chargement...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if products.length === 0}
    <p>Aucun produit pour le moment.</p>
  {:else}
    {#each products as product (product.id)}
      <div class="card" data-testid="product-card-{product.nom}">
        {#if editing[product.id]}
          <label>Nom<input type="text" bind:value={editing[product.id].nom} /></label>
          <label>Unité<input type="text" bind:value={editing[product.id].unite} /></label>
          <label>Catégorie<input type="text" bind:value={editing[product.id].categorie} /></label>
          <button on:click={() => saveEdit(product.id)} disabled={savingId === product.id}>
            {savingId === product.id ? "Sauvegarde..." : "Sauvegarder"}
          </button>
          <button class="secondary" on:click={() => cancelEdit(product.id)}>Annuler</button>
        {:else}
          <p><strong>{product.nom}</strong> — {product.unite} — {product.categorie}</p>
          <button class="secondary" on:click={() => startEdit(product)}>Modifier</button>
        {/if}
      </div>
    {/each}
  {/if}
</div>
