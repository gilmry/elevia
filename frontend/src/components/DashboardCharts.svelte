<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  // chart.js/auto registers every controller/scale/plugin Chart.js ships
  // (radar, bubble, scatter, filler, decimation...) - none of which this
  // dashboard uses. Registering only line/bar/pie + what they need cuts
  // the bundle roughly in half.
  import {
    Chart,
    LineController,
    BarController,
    PieController,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    BarElement,
    ArcElement,
    Title,
    Legend,
    Tooltip,
  } from "chart.js";
  import { CURRENCY_SYMBOL } from "../lib/format";
  import type { ExploitationDashboard } from "../lib/types";

  Chart.register(
    LineController,
    BarController,
    PieController,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    BarElement,
    ArcElement,
    Title,
    Legend,
    Tooltip,
  );

  export let dashboard: ExploitationDashboard;

  let costCanvas: HTMLCanvasElement;
  let marginCanvas: HTMLCanvasElement;
  let costPerUnitCanvas: HTMLCanvasElement;
  let pieCanvas: HTMLCanvasElement;
  let charts: Chart[] = [];

  const palette = ["#2e7d32", "#c62828", "#1565c0", "#ef6c00", "#6a1b9a", "#00838f"];

  // Same shape as the "same month can have several productions" model: one
  // line per distinct produit name across the months where it appears,
  // rather than assuming a single series.
  function seriesByProduction(
    field: "estimated_margin" | "cost_per_unit",
  ): { label: string; data: (number | null)[] }[] {
    const noms = [...new Set(dashboard.monthly.flatMap((m) => m.productions.map((p) => p.nom)))];
    return noms.map((nom) => ({
      label: nom,
      data: dashboard.monthly.map((m) => {
        const production = m.productions.find((p) => p.nom === nom);
        const value = production?.[field];
        return value ? Number(value) : null;
      }),
    }));
  }

  onMount(() => {
    const labels = dashboard.monthly.map((m) => m.mois);

    charts.push(
      new Chart(costCanvas, {
        type: "line",
        data: {
          labels,
          datasets: [
            {
              label: `Coût total (${CURRENCY_SYMBOL})`,
              data: dashboard.monthly.map((m) => Number(m.total_cost)),
              borderColor: palette[0],
              backgroundColor: palette[0],
            },
          ],
        },
        options: { plugins: { title: { display: true, text: "Coût total par mois" } } },
      }),
    );

    charts.push(
      new Chart(marginCanvas, {
        type: "bar",
        data: {
          labels,
          datasets: seriesByProduction("estimated_margin").map((s, i) => ({
            ...s,
            backgroundColor: palette[i % palette.length],
          })),
        },
        options: { plugins: { title: { display: true, text: "Marge estimée par mois" } } },
      }),
    );

    charts.push(
      new Chart(costPerUnitCanvas, {
        type: "line",
        data: {
          labels,
          datasets: seriesByProduction("cost_per_unit").map((s, i) => ({
            ...s,
            borderColor: palette[i % palette.length],
            backgroundColor: palette[i % palette.length],
          })),
        },
        options: {
          plugins: { title: { display: true, text: "Coût par unité produite (efficacité)" } },
        },
      }),
    );

    if (dashboard.totals_by_product.length > 0) {
      charts.push(
        new Chart(pieCanvas, {
          type: "pie",
          data: {
            labels: dashboard.totals_by_product.map(([nom]) => nom),
            datasets: [
              {
                data: dashboard.totals_by_product.map(([, total]) => Number(total)),
                backgroundColor: dashboard.totals_by_product.map(
                  (_, i) => palette[i % palette.length],
                ),
              },
            ],
          },
          options: {
            plugins: {
              title: { display: true, text: "Répartition des coûts par intrant" },
            },
          },
        }),
      );
    }
  });

  onDestroy(() => {
    charts.forEach((c) => c.destroy());
  });
</script>

<div class="card">
  <div class="chart-wrap"><canvas bind:this={costCanvas}></canvas></div>
  <div class="chart-wrap"><canvas bind:this={marginCanvas}></canvas></div>
  <div class="chart-wrap"><canvas bind:this={costPerUnitCanvas}></canvas></div>
  {#if dashboard.totals_by_product.length > 0}
    <div class="chart-wrap"><canvas bind:this={pieCanvas}></canvas></div>
  {/if}
</div>

<style>
  .chart-wrap {
    position: relative;
    height: 260px;
    margin-bottom: 1.5rem;
  }

  .chart-wrap:last-child {
    margin-bottom: 0;
  }
</style>
