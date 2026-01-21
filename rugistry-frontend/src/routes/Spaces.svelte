<script lang="ts">
  import { onMount } from 'svelte';
  import { createDialog, melt } from '@melt-ui/svelte';
  import { navigate } from 'svelte-routing';
  import { getSpaces, createSpace, deleteSpace, type Space } from '../lib/api';

  let spaces: Space[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let newSpaceName = $state('');
  let newSpaceDescription = $state('');

  const {
    elements: { trigger, overlay, content, title, close, portalled },
    states: { open }
  } = createDialog({
    forceVisible: true
  });

  async function loadSpaces() {
    try {
      loading = true;
      spaces = await getSpaces();
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load spaces';
    } finally {
      loading = false;
    }
  }

  async function handleCreateSpace() {
    if (!newSpaceName.trim()) return;
    
    try {
      await createSpace({
        name: newSpaceName,
        description: newSpaceDescription || undefined,
      });
      open.set(false);
      newSpaceName = '';
      newSpaceDescription = '';
      await loadSpaces();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create space';
    }
  }

  async function handleDeleteSpace(id: string) {
    if (!confirm('Are you sure you want to delete this space?')) return;
    
    try {
      await deleteSpace(id);
      await loadSpaces();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete space';
    }
  }

  function viewSpace(id: string) {
    navigate(`/spaces/${id}`);
  }

  function closeDialog() {
    open.set(false);
    newSpaceName = '';
    newSpaceDescription = '';
  }

  onMount(loadSpaces);
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Spaces</h1>
    <button
      use:melt={$trigger}
      class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
    >
      + Create Space
    </button>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative dark:bg-red-900 dark:border-red-700 dark:text-red-200">
      <span>{error}</span>
      <button class="absolute top-0 right-0 px-4 py-3" onclick={() => error = ''}>×</button>
    </div>
  {/if}

  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
    </div>
  {:else if spaces.length === 0}
    <div class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">No spaces</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Get started by creating a new space.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each spaces as space (space.id)}
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden border border-gray-200 dark:border-gray-700">
          <div class="p-5">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">{space.name}</h3>
            {#if space.description}
              <p class="text-gray-600 dark:text-gray-400 text-sm mb-4">{space.description}</p>
            {:else}
              <p class="text-gray-400 dark:text-gray-500 text-sm mb-4 italic">No description</p>
            {/if}
            <p class="text-xs text-gray-500 dark:text-gray-500">
              Created: {new Date(space.created_at).toLocaleDateString()}
            </p>
          </div>
          <div class="px-5 py-3 bg-gray-50 dark:bg-gray-700 flex justify-end gap-2">
            <button
              onclick={() => viewSpace(space.id)}
              class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              View
            </button>
            <button
              onclick={() => handleDeleteSpace(space.id)}
              class="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Dialog Portal -->
<div use:melt={$portalled}>
  {#if $open}
    <div use:melt={$overlay} class="fixed inset-0 z-40 bg-black/50"></div>
    <div
      use:melt={$content}
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl"
    >
      <h2 use:melt={$title} class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        Create New Space
      </h2>
      
      <form onsubmit={(e) => { e.preventDefault(); handleCreateSpace(); }}>
        <div class="space-y-4">
          <div>
            <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Name *
            </label>
            <input
              id="name"
              type="text"
              bind:value={newSpaceName}
              placeholder="Enter space name"
              required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
          <div>
            <label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description
            </label>
            <textarea
              id="description"
              bind:value={newSpaceDescription}
              placeholder="Enter description (optional)"
              rows="3"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            ></textarea>
          </div>
        </div>
        
        <div class="flex justify-end gap-3 mt-6">
          <button
            type="button"
            use:melt={$close}
            class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Create
          </button>
        </div>
      </form>

      <button
        use:melt={$close}
        class="absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        aria-label="Close"
      >
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>
