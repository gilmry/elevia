<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentUser, logout } from "../lib/auth";
  import type { AuthClaims } from "../lib/types";

  let user: AuthClaims | null = null;
  let path = "";

  onMount(() => {
    user = getCurrentUser();
    path = window.location.pathname;
  });
</script>

{#if user}
  <nav class="app-nav">
    {#if user.role === "exploitation"}
      <a href="/entries" class:active={path === "/entries"}>Coûts</a>
      <a href="/production" class:active={path === "/production"}>Production</a>
      <a href="/dashboard" class:active={path === "/dashboard"}>Mon dashboard</a>
    {/if}
    {#if user.role === "admin"}
      <a href="/admin/exploitations" class:active={path === "/admin/exploitations"}>
        Exploitations
      </a>
      <a href="/admin/products" class:active={path === "/admin/products"}>Produits</a>
    {/if}
    <a href="/coop" class:active={path === "/coop"}>Coopérative</a>
    <a href="/account" class:active={path === "/account"}>Mon compte</a>
    <button class="secondary" on:click={logout}>Déconnexion</button>
  </nav>
{/if}

<style>
  button.secondary {
    margin-left: auto;
    padding: 0.4rem 0.7rem;
    min-height: unset;
    font-size: 0.9rem;
  }
</style>
