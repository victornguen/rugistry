<script lang="ts">
  import { onMount } from 'svelte';
  import { Router, Route } from 'svelte-routing';
  import Navbar from './components/Navbar.svelte';
  import Spaces from './routes/Spaces.svelte';
  import SpaceDetail from './routes/SpaceDetail.svelte';
  import Login from './routes/Login.svelte';
  import Register from './routes/Register.svelte';
  import { initAuth, authState } from './lib/auth';

  let isReady = $state(false);

  onMount(() => {
    initAuth();
    isReady = true;
  });
</script>

{#if !isReady}
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center">
    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
  </div>
{:else}
  <Router>
    <Route path="/register"><Register /></Route>
    <Route path="/login"><Login /></Route>

    {#if $authState.authenticated}
      <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
        <Navbar />
        <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <Route path="/"><Spaces /></Route>
          <Route path="/spaces/:id" let:params><SpaceDetail id={params.id} /></Route>
        </main>
      </div>
    {:else}
      <Route path="/"><Login /></Route>
      <Route path="/spaces/:id"><Login /></Route>
    {/if}
  </Router>
{/if}
