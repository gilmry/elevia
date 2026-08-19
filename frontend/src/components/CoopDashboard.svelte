<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { requireAuth } from "../lib/auth";
  import { CURRENCY_SYMBOL, formatAmount, formatQuantity } from "../lib/format";
  import type { CoopDashboard } from "../lib/types";

  let dashboard: CoopDashboard | null = null;
  let loading = true;
  let error = "";

  onMount(async () => {
    const user = requireAuth();
    if (!user) return;

    try {
      dashboard = await api.getCoopDashboard();
    } catch {
      error = "Impossible de charger le dashboard coopérative.";
    } finally {
      loading = false;
    }
  });
</script>

<h1>Coopérative {dashboard ? `· ${dashboard.mois}` : ""}</h1>

{#if loading}
  <p>Chargement...</p>
{:else if error}
  <p class="error">{error}</p>
{:else if dashboard}
  <div class="card">
    <h2>Besoins en intrants ce mois-ci</h2>
    {#if dashboard.intrant_needs.length === 0}
      <p>Aucune donnée.</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Produit</th>
            <th>Quantité totale</th>
          </tr>
        </thead>
        <tbody>
          {#each dashboard.intrant_needs as need (need.product_id)}
            <tr>
              <td>{need.nom}</td>
              <td>{formatQuantity(need.total_quantite)} {need.unite}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <div class="card">
    <h2>Marge moyenne</h2>
    {#if dashboard.average_margin !== null}
      <div class="stat">
        <span class="value">{formatAmount(dashboard.average_margin)} {CURRENCY_SYMBOL}</span>
      </div>
    {:else}
      <p>Pas encore de données.</p>
    {/if}
  </div>

  <div class="card">
    <h2>Écarts de coût / unité (anonymisé, par quartile)</h2>
    {#if dashboard.cost_per_unit_quartiles}
      <div class="stat">
        <span class="label">Q1</span><span class="value"
          >{formatAmount(dashboard.cost_per_unit_quartiles.q1)} {CURRENCY_SYMBOL}</span
        >
      </div>
      <div class="stat">
        <span class="label">Médiane</span><span class="value"
          >{formatAmount(dashboard.cost_per_unit_quartiles.median)} {CURRENCY_SYMBOL}</span
        >
      </div>
      <div class="stat">
        <span class="label">Q3</span><span class="value"
          >{formatAmount(dashboard.cost_per_unit_quartiles.q3)} {CURRENCY_SYMBOL}</span
        >
      </div>
    {:else}
      <p>Pas encore de données.</p>
    {/if}
  </div>
{/if}
