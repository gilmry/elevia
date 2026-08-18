<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError } from "../lib/api";
  import { requireAuth } from "../lib/auth";

  onMount(() => {
    requireAuth();
  });

  let currentPassword = "";
  let newPassword = "";
  let confirmPassword = "";
  let error = "";
  let success = false;
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    success = false;

    if (newPassword !== confirmPassword) {
      error = "Les deux mots de passe ne correspondent pas.";
      return;
    }

    loading = true;
    try {
      await api.changePassword(currentPassword, newPassword);
      success = true;
      currentPassword = "";
      newPassword = "";
      confirmPassword = "";
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit={submit}>
  <label>
    Mot de passe actuel
    <input
      type="password"
      bind:value={currentPassword}
      required
      autocomplete="current-password"
    />
  </label>
  <label>
    Nouveau mot de passe
    <input
      type="password"
      bind:value={newPassword}
      required
      minlength="8"
      autocomplete="new-password"
    />
  </label>
  <label>
    Confirmer le nouveau mot de passe
    <input
      type="password"
      bind:value={confirmPassword}
      required
      minlength="8"
      autocomplete="new-password"
    />
  </label>
  {#if error}
    <p class="error">{error}</p>
  {/if}
  {#if success}
    <p class="success">Mot de passe mis à jour.</p>
  {/if}
  <button type="submit" disabled={loading}>
    {loading ? "Mise à jour..." : "Changer le mot de passe"}
  </button>
</form>
