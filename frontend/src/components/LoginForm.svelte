<script lang="ts">
  import { onMount } from "svelte";
  import { api, ApiError } from "../lib/api";
  import { getCurrentUser, saveToken } from "../lib/auth";

  let email = "";
  let password = "";
  let error = "";
  let loading = false;

  onMount(() => {
    const user = getCurrentUser();
    if (user) {
      redirectFor(user.role);
    }
  });

  function redirectFor(role: "admin" | "exploitation") {
    window.location.href = role === "admin" ? "/admin/exploitations" : "/entries";
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    loading = true;
    try {
      const { token } = await api.login(email, password);
      saveToken(token);
      const user = getCurrentUser();
      if (user) redirectFor(user.role);
    } catch (err) {
      error =
        err instanceof ApiError
          ? err.message
          : "Impossible de se connecter. Vérifiez votre connexion.";
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit={submit}>
  <label>
    Email
    <input type="email" bind:value={email} required autocomplete="username" />
  </label>
  <label>
    Mot de passe
    <input type="password" bind:value={password} required autocomplete="current-password" />
  </label>
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <button type="submit" disabled={loading}>{loading ? "Connexion..." : "Se connecter"}</button>
</form>
