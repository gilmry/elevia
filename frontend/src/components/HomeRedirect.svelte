<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentUser } from "../lib/auth";

  onMount(() => {
    const user = getCurrentUser();
    if (!user) {
      window.location.href = "/login";
    } else if (user.role === "admin") {
      window.location.href = "/admin/exploitations";
    } else {
      window.location.href = "/entries";
    }
  });
</script>

<p>Redirection...</p>
