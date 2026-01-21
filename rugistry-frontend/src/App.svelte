<script lang="ts">
  import { Router, Route } from 'svelte-routing';
  import Navbar from './components/Navbar.svelte';
  import Spaces from './routes/Spaces.svelte';
  import SpaceDetail from './routes/SpaceDetail.svelte';
</script>

<Router>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <Navbar />
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <Route path="/"><Spaces /></Route>
      <Route path="/spaces/:id" let:params><SpaceDetail id={params.id} /></Route>
    </main>
  </div>
</Router>
