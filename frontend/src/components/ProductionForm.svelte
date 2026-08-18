<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError, OfflineError } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { queueProduction } from "../lib/offlineQueue";

  function currentMonth(): string {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  }

  let exploitationId = "";
  let mois = currentMonth();
  let quantiteProduite = "";
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
      quantite_produite: quantiteProduite,
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
    <input type="text" bind:value={unite} required placeholder="ex: tonnes" />
  </label>
  <label>
    Prix de vente unitaire (€, optionnel)
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
