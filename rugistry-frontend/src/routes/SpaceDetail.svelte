<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createDialog, melt } from '@melt-ui/svelte';
  import { navigate } from 'svelte-routing';
  import { 
    getSpace, 
    getEntries, 
    createEntry, 
    updateEntry,
    deleteEntry, 
    type Space, 
    type RegistryEntry,
    type CreateEntryRequest 
  } from '../lib/api';

  let { id }: { id: string } = $props();

  let space: Space | null = $state(null);
  let entries: RegistryEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let ws: WebSocket | null = $state(null);

  // New entry form
  let newEntryKey = $state('');
  let newEntryValue = $state('');
  let newEntryValueType = $state<'string' | 'number' | 'boolean' | 'json'>('string');
  let newEntryDescription = $state('');

  // Edit entry form
  let editingEntry: RegistryEntry | null = $state(null);
  let editEntryKey = $state('');
  let editEntryValue = $state('');
  let editEntryValueType = $state<'string' | 'number' | 'boolean' | 'json'>('string');
  let editEntryDescription = $state('');

  const {
    elements: { trigger, overlay, content, title, close, portalled },
    states: { open }
  } = createDialog({
    forceVisible: true
  });

  const editDialog = createDialog({
    forceVisible: true
  });

  const {
    elements: {
      portalled: editPortalled,
      overlay: editOverlay,
      content: editContent,
      title: editTitle,
      close: editClose
    },
    states: { open: editOpen }
  } = editDialog;

  async function loadData() {
    try {
      loading = true;
      const [spaceData, entriesData] = await Promise.all([
        getSpace(id),
        getEntries(id)
      ]);
      space = spaceData;
      entries = entriesData;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  }

  function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.hostname}:3000/api/ws/${id}`;
    
    ws = new WebSocket(wsUrl);
    
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        // Backend sends: { event_type: "created"/"updated"/"deleted", space_id, entry_id, key, entry, timestamp }
        if (message.space_id === id) {
          if (message.event_type === 'created' && message.entry) {
            // Add new entry to the list
            entries = [...entries, message.entry];
          } else if (message.event_type === 'updated' && message.entry) {
            // Update existing entry in the list
            entries = entries.map(e => e.id === message.entry.id ? message.entry : e);
          } else if (message.event_type === 'deleted' && message.entry_id) {
            // Remove entry from the list
            entries = entries.filter(e => e.id !== message.entry_id);
          }
        }
      } catch (err) {
        console.error('WebSocket message error:', err);
      }
    };

    ws.onclose = () => {
      // Reconnect after 3 seconds
      setTimeout(() => {
        if (!ws || ws.readyState === WebSocket.CLOSED) {
          connectWebSocket();
        }
      }, 3000);
    };
  }

  async function handleCreateEntry() {
    if (!newEntryKey.trim() || !newEntryValue.trim()) return;
    
    try {
      const request: CreateEntryRequest = {
        key: newEntryKey,
        value: newEntryValue,
        value_type: newEntryValueType,
        description: newEntryDescription || undefined,
      };
      await createEntry(id, request);
      open.set(false);
      newEntryKey = '';
      newEntryValue = '';
      newEntryValueType = 'string';
      newEntryDescription = '';
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create entry';
    }
  }

  function handleEditEntry(entry: RegistryEntry) {
    editingEntry = entry;
    editEntryKey = entry.key;
    editEntryValue = entry.value;
    editEntryValueType = entry.value_type;
    editEntryDescription = entry.description || '';
    editOpen.set(true);
  }

  async function handleUpdateEntry() {
    if (!editingEntry || !editEntryKey.trim() || !editEntryValue.trim()) return;
    
    try {
      const updates: Partial<CreateEntryRequest> = {
        key: editEntryKey,
        value: editEntryValue,
        value_type: editEntryValueType,
        description: editEntryDescription || undefined,
      };
      await updateEntry(id, editingEntry.id, updates);
      editOpen.set(false);
      editingEntry = null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to update entry';
    }
  }

  async function handleDeleteEntry(entryId: string) {
    if (!confirm('Are you sure you want to delete this entry?')) return;
    
    try {
      await deleteEntry(id, entryId);
      await loadData();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete entry';
    }
  }

  function goBack() {
    navigate('/');
  }

  onMount(() => {
    loadData();
    connectWebSocket();
  });

  onDestroy(() => {
    if (ws) {
      ws.close();
    }
  });
</script>

<div class="space-y-6">
  <div class="flex items-center gap-4">
    <button
      onclick={goBack}
      aria-label="Go back"
      class="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
    >
      <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
      </svg>
    </button>
    <div class="flex-1">
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
        {space?.name ?? 'Loading...'}
      </h1>
      {#if space?.description}
        <p class="text-gray-600 dark:text-gray-400 mt-1">{space.description}</p>
      {/if}
    </div>
    <button
      use:melt={$trigger}
      class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
    >
      + Add Entry
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
  {:else if entries.length === 0}
    <div class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg shadow">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">No entries</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Add your first registry entry to this space.</p>
    </div>
  {:else}
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50 dark:bg-gray-700">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Key</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Value</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Type</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Version</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Updated</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Actions</th>
          </tr>
        </thead>
        <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
          {#each entries as entry (entry.id)}
            <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">{entry.key}</td>
              <td class="px-6 py-4 text-sm text-gray-600 dark:text-gray-300">
                <div class="max-w-xs truncate" title={entry.value}>{entry.value}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                <span class="px-2 py-1 bg-gray-100 dark:bg-gray-600 rounded text-xs">{entry.value_type}</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{entry.version}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                {new Date(entry.updated_at).toLocaleString()}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm space-x-3">
                <button
                  onclick={() => handleEditEntry(entry)}
                  class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 font-medium"
                >
                  Edit
                </button>
                <button
                  onclick={() => handleDeleteEntry(entry.id)}
                  class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 font-medium"
                >
                  Delete
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
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
        Add New Entry
      </h2>
      
      <form onsubmit={(e) => { e.preventDefault(); handleCreateEntry(); }}>
        <div class="space-y-4">
          <div>
            <label for="key" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Key *
            </label>
            <input
              id="key"
              type="text"
              bind:value={newEntryKey}
              placeholder="e.g., config.database.host"
              required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
          <div>
            <label for="value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Value *
            </label>
            <textarea
              id="value"
              bind:value={newEntryValue}
              placeholder="Enter value"
              rows="4"
              required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm"
            ></textarea>
          </div>
          <div>
            <label for="valueType" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Value Type
            </label>
            <select
              id="valueType"
              bind:value={newEntryValueType}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value="string">String</option>
              <option value="number">Number</option>
              <option value="boolean">Boolean</option>
              <option value="json">JSON</option>
            </select>
          </div>
          <div>
            <label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description (optional)
            </label>
            <input
              id="description"
              type="text"
              bind:value={newEntryDescription}
              placeholder="Optional description"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
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
            Add Entry
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

<!-- Edit Entry Dialog Portal -->
<div use:melt={$editPortalled}>
  {#if $editOpen}
    <div use:melt={$editOverlay} class="fixed inset-0 z-40 bg-black/50"></div>
    <div
      use:melt={$editContent}
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl"
    >
      <h2 use:melt={$editTitle} class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        Edit Entry
      </h2>
      
      <form onsubmit={(e) => { e.preventDefault(); handleUpdateEntry(); }}>
        <div class="space-y-4">
          <div>
            <label for="edit-key" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Key *
            </label>
            <input
              id="edit-key"
              type="text"
              bind:value={editEntryKey}
              placeholder="e.g., config.database.host"
              required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
          <div>
            <label for="edit-value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Value *
            </label>
            <textarea
              id="edit-value"
              bind:value={editEntryValue}
              placeholder="Enter value"
              rows="4"
              required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm"
            ></textarea>
          </div>
          <div>
            <label for="edit-valueType" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Value Type
            </label>
            <select
              id="edit-valueType"
              bind:value={editEntryValueType}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value="string">String</option>
              <option value="number">Number</option>
              <option value="boolean">Boolean</option>
              <option value="json">JSON</option>
            </select>
          </div>
          <div>
            <label for="edit-description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description (optional)
            </label>
            <input
              id="edit-description"
              type="text"
              bind:value={editEntryDescription}
              placeholder="Optional description"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
        
        <div class="flex justify-end gap-3 mt-6">
          <button
            type="button"
            use:melt={$editClose}
            class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Update Entry
          </button>
        </div>
      </form>

      <button
        use:melt={$editClose}
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
