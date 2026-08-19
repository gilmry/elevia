<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError, OfflineError } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { queueProduction } from "../lib/offlineQueue";
  import { CURRENCY_SYMBOL } from "../lib/format";

  function currentMonth(): string {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  }

  let exploitationId = "";
  let mois = currentMonth();
  let nom = "";
  let quantiteProduite = "";
  let quantiteVendue = "";
  let unite = "";
  let prixUnitaireVente = "";
  let message = "";
  let error = "";
  let loading = false;

  onMount(() => {
    const user = requireAuth("exploitation");
    if (!user || !user.exploitation_id) return;
    exploitationId = user.exploitation_id;
  });

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    message = "";
    loading = true;

    const input = {
      mois,
      nom,
      quantite_produite: quantiteProduite,
      quantite_vendue: quantiteVendue || null,
      unite,
      prix_unitaire_vente: prixUnitaireVente || null,
    };

    try {
      await api.submitProduction(exploitationId, input);
      message = "Production enregistrée.";
    } catch (err) {
      if (err instanceof OfflineError) {
        await queueProduction(exploitationId, input);
        message = "Hors ligne : enregistré localement, sera envoyé à la reconnexion.";
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

<h1>Déclarer la production</h1>

<form on:submit={submit}>
  <label>
    Qu'est-ce qui a été produit ?
    <input type="text" bind:value={nom} required placeholder="ex: Œufs, Viande de poulet" />
  </label>
  <p class="notice">
    Une ferme peut produire plusieurs choses le même mois : un nom déjà
    utilisé ce mois-ci met à jour cette ligne, un nom différent en ajoute
    une nouvelle.
  </p>
  <label>
    Quantité produite
    <input
      type="number"
      step="0.001"
      min="0"
      bind:value={quantiteProduite}
      required
      inputmode="decimal"
    />
  </label>
  <label>
    Unité
    <input type="text" bind:value={unite} required placeholder="ex: kg, litres, douzaines" />
  </label>
  <label>
    Quantité vendue (optionnel)
    <input
      type="number"
      step="0.001"
      min="0"
      bind:value={quantiteVendue}
      inputmode="decimal"
      placeholder="peut différer de la quantité produite"
    />
  </label>
  <label>
    Prix de vente unitaire ({CURRENCY_SYMBOL}, optionnel)
    <input type="number" step="0.01" min="0" bind:value={prixUnitaireVente} inputmode="decimal" />
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
