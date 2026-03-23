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
    getShares,
    addShare,
    removeShare,
    searchUsers,
    listWebhooks,
    createWebhook,
    deleteWebhook,
    type Space, 
    type RegistryEntry,
    type CreateEntryRequest,
    type SpaceShare,
    type UserSearchResult,
    type Webhook,
  } from '../lib/api';
  import MonacoEditor from '../components/MonacoEditor.svelte';

  type ValueType = 'string' | 'number' | 'boolean' | 'json' | 'list' | 'hocon' | 'toml' | 'yaml';

  let { id }: { id: string } = $props();

  let space: Space | null = $state(null);
  let entries: RegistryEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let formError = $state('');
  let ws: WebSocket | null = null;

  // New entry form
  let newEntryKey = $state('');
  let newEntryValue = $state('');
  let newEntryListItemInput = $state('');
  let newEntryValueType = $state<ValueType>('string');
  let newEntryDescription = $state('');
  let newEntryListItems = $state<string[]>([]);

  // Edit entry form
  let editingEntry: RegistryEntry | null = $state(null);
  let editEntryKey = $state('');
  let editEntryValue = $state('');
  let editEntryListItemInput = $state('');
  let editEntryValueType = $state<ValueType>('string');
  let editEntryDescription = $state('');
  let editEntryListItems = $state<string[]>([]);

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
    elements: { trigger: shareTrigger, overlay: shareOverlay, content: shareContent, title: shareTitle, close: shareClose, portalled: sharePortalled },
    states: { open: shareOpen }
  } = createDialog({ forceVisible: true });

  // ── Share state ───────────────────────────────────────────────────────────
  let shares: SpaceShare[] = $state([]);
  let sharesLoading = $state(false);
  let shareUsername = $state('');
  let sharePermission = $state('readonly');
  let shareError = $state('');
  let userSuggestions: UserSearchResult[] = $state([]);
  let suggestionsOpen = $state(false);
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

  function onShareUsernameInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    shareUsername = val;
    if (searchDebounce) clearTimeout(searchDebounce);
    if (!val.trim()) { userSuggestions = []; suggestionsOpen = false; return; }
    searchDebounce = setTimeout(async () => {
      userSuggestions = await searchUsers(val);
      suggestionsOpen = userSuggestions.length > 0;
    }, 200);
  }

  function selectSuggestion(u: UserSearchResult) {
    shareUsername = u.username;
    userSuggestions = [];
    suggestionsOpen = false;
  }

  async function openShareModal() {
    if (!space) return;
    shareError = ''; shareUsername = ''; sharePermission = 'readonly';
    sharesLoading = true;
    shareOpen.set(true);
    try {
      shares = await getShares(space.id);
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Failed to load shares';
    } finally {
      sharesLoading = false;
    }
  }

  async function handleAddShare() {
    if (!space || !shareUsername.trim()) return;
    try {
      await addShare(space.id, shareUsername.trim(), sharePermission);
      shareUsername = ''; userSuggestions = []; suggestionsOpen = false;
      shares = await getShares(space.id);
      shareError = '';
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Failed to add share';
    }
  }

  async function handleRemoveShare(userId: string) {
    if (!space) return;
    try {
      await removeShare(space.id, userId);
      shares = await getShares(space.id);
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Failed to remove share';
    }
  }

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

  const viewDialog = createDialog({ forceVisible: true });
  const {
    elements: { portalled: viewPortalled, overlay: viewOverlay, content: viewContent, title: viewTitle, close: viewClose },
    states: { open: viewOpen }
  } = viewDialog;

  let viewingEntry: RegistryEntry | null = $state(null);

  function handleViewEntry(entry: RegistryEntry) {
    viewingEntry = entry;
    viewDialog.states.open.set(true);
  }

  // ── Webhook dialog ────────────────────────────────────────────────────────
  const webhookDialog = createDialog({ forceVisible: true });
  const {
    elements: { portalled: webhookPortalled, overlay: webhookOverlay, content: webhookContent, title: webhookTitle, close: webhookClose },
    states: { open: webhookOpen }
  } = webhookDialog;

  let webhooks: Webhook[] = $state([]);
  let webhooksLoading = $state(false);
  let newWebhookUrl = $state('');
  let newWebhookSecret = $state('');
  let webhookError = $state('');

  async function openWebhookModal() {
    if (!space) return;
    webhookError = ''; newWebhookUrl = ''; newWebhookSecret = '';
    webhooksLoading = true;
    webhookOpen.set(true);
    try {
      webhooks = await listWebhooks(space.id);
    } catch (e) {
      webhookError = e instanceof Error ? e.message : 'Failed to load webhooks';
    } finally {
      webhooksLoading = false;
    }
  }

  async function handleCreateWebhook() {
    if (!space || !newWebhookUrl.trim()) return;
    webhookError = '';
    try {
      const created = await createWebhook(space.id, newWebhookUrl.trim(), newWebhookSecret.trim() || undefined);
      webhooks = [...webhooks, created];
      newWebhookUrl = ''; newWebhookSecret = '';
    } catch (e) {
      webhookError = e instanceof Error ? e.message : 'Failed to add webhook';
    }
  }

  async function handleDeleteWebhook(webhookId: string) {
    if (!space) return;
    try {
      await deleteWebhook(space.id, webhookId);
      webhooks = webhooks.filter(w => w.id !== webhookId);
    } catch (e) {
      webhookError = e instanceof Error ? e.message : 'Failed to delete webhook';
    }
  }

  // ── Permission helpers ────────────────────────────────────────────────────
  function canCreate(): boolean {
    if (!space) return true;
    return space.permission === null || space.permission === 'write' || space.permission === 'appendonly';
  }
  function canModify(): boolean {
    if (!space) return true;
    return space.permission === null || space.permission === 'write';
  }
  function permissionLabel(): string {
    if (!space || space.permission === null) return 'owner';
    return space.permission;
  }
  function permissionBadgeClass(): string {
    if (!space || space.permission === null) return 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300';
    if (space.permission === 'readonly') return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300';
    if (space.permission === 'write') return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
    if (space.permission === 'appendonly') return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300';
    return 'bg-gray-100 text-gray-600';
  }

  // ── Monaco language mapping ───────────────────────────────────────────────
  function monacoLang(vt: string): string {
    switch (vt) {
      case 'json': return 'json';
      case 'yaml': return 'yaml';
      case 'toml': return 'ini';
      case 'hocon': return 'hcl';
      default: return 'plaintext';
    }
  }
  function usesMonaco(vt: string): boolean {
    return ['json', 'yaml', 'toml', 'hocon'].includes(vt);
  }

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
    const wsUrl = `${protocol}//${window.location.hostname}:3000/api/v1/ws/${id}`;
    ws = new WebSocket(wsUrl);
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.space_id === id) {
          if (message.event_type === 'created' && message.entry) entries = [...entries, message.entry];
          else if (message.event_type === 'updated' && message.entry) entries = entries.map(e => e.id === message.entry.id ? message.entry : e);
          else if (message.event_type === 'deleted' && message.entry_id) entries = entries.filter(e => e.id !== message.entry_id);
        }
      } catch {}
    };
    ws.onclose = () => {
      setTimeout(() => { if (!ws || ws.readyState === WebSocket.CLOSED) connectWebSocket(); }, 3000);
    };
  }

  async function handleCreateEntry() {
    if (!newEntryKey.trim()) return;
    formError = '';
    let finalValue = newEntryValue;
    if (newEntryValueType === 'list') {
      finalValue = JSON.stringify(newEntryListItems);
    } else if (!finalValue.trim()) {
      formError = 'Value is required';
      return;
    }
    try {
      const created = await createEntry(id, { key: newEntryKey, value: finalValue, value_type: newEntryValueType, description: newEntryDescription || undefined });
      entries = [...entries, created];
      open.set(false);
      newEntryKey = ''; newEntryValue = ''; newEntryValueType = 'string'; newEntryDescription = ''; newEntryListItems = []; newEntryListItemInput = '';
    } catch (e) {
      formError = e instanceof Error ? e.message : 'Failed to create entry';
    }
  }

  function handleEditEntry(entry: RegistryEntry) {
    editingEntry = entry;
    editEntryKey = entry.key;
    editEntryValue = entry.value;
    editEntryValueType = entry.value_type as ValueType;
    editEntryDescription = entry.description || '';
    editEntryListItems = entry.value_type === 'list' ? (() => { try { return JSON.parse(entry.value); } catch { return []; } })() : [];
    editEntryListItemInput = '';
    formError = '';
    editDialog.states.open.set(true);
  }

  async function handleUpdateEntry() {
    if (!editingEntry || !editEntryKey.trim()) return;
    formError = '';
    let finalValue = editEntryValue;
    if (editEntryValueType === 'list') {
      finalValue = JSON.stringify(editEntryListItems);
    } else if (!finalValue.trim()) {
      formError = 'Value is required';
      return;
    }
    try {
      const updated = await updateEntry(id, editingEntry.id, { key: editEntryKey, value: finalValue, value_type: editEntryValueType, description: editEntryDescription || undefined });
      entries = entries.map(e => e.id === updated.id ? updated : e);
      editDialog.states.open.set(false);
      editingEntry = null;
      editEntryListItems = [];
    } catch (e) {
      formError = e instanceof Error ? e.message : 'Failed to update entry';
    }
  }

  // List management helpers for new entry
  function addNewListItem() {
    if (newEntryListItemInput.trim()) {
      newEntryListItems = [...newEntryListItems, newEntryListItemInput.trim()];
      newEntryListItemInput = '';
    }
  }
  function removeNewListItem(index: number) {
    newEntryListItems = newEntryListItems.filter((_, i) => i !== index);
  }

  // List management helpers for edit entry
  function addEditListItem() {
    if (editEntryListItemInput.trim()) {
      editEntryListItems = [...editEntryListItems, editEntryListItemInput.trim()];
      editEntryListItemInput = '';
    }
  }
  function removeEditListItem(index: number) {
    editEntryListItems = editEntryListItems.filter((_, i) => i !== index);
  }

  function parseListValue(value: string): string[] {
    try { return JSON.parse(value); } catch { return []; }
  }

  async function handleDeleteEntry(entryId: string) {
    if (!confirm('Are you sure you want to delete this entry?')) return;
    
    try {
      await deleteEntry(id, entryId);
      entries = entries.filter(e => e.id !== entryId);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete entry';
    }
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
      onclick={() => navigate('/')}
      aria-label="Go back"
      class="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
    >
      <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
      </svg>
    </button>
    <div class="flex-1">
      <div class="flex items-center gap-2 flex-wrap">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          {space?.name ?? 'Loading...'}
        </h1>
        {#if space}
          <span class="px-2 py-0.5 text-xs rounded-full font-medium {permissionBadgeClass()}">{permissionLabel()}</span>
        {/if}
      </div>
      {#if space?.description}
        <p class="text-gray-600 dark:text-gray-400 mt-1">{space.description}</p>
      {/if}
    </div>
    {#if space?.permission === null}
      <button
        onclick={openShareModal}
        class="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors font-medium border border-gray-300 dark:border-gray-600"
      >
        Share
      </button>
      <button
        onclick={openWebhookModal}
        class="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors font-medium border border-gray-300 dark:border-gray-600"
      >
        Webhooks
      </button>
    {/if}
    {#if canCreate()}
      <button
        use:melt={$trigger}
        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
      >
        + Add Entry
      </button>
    {/if}
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
                {#if entry.value_type === 'list'}
                  {@const items = parseListValue(entry.value)}
                  <div class="flex flex-wrap gap-1">
                    {#each items.slice(0, 3) as item}
                      <span class="text-xs bg-gray-100 dark:bg-gray-600 px-2 py-0.5 rounded font-mono">{item}</span>
                    {/each}
                    {#if items.length > 3}
                      <span class="text-xs text-gray-500">+{items.length - 3} more</span>
                    {/if}
                  </div>
                {:else}
                  <div class="max-w-xs truncate font-mono text-xs" title={entry.value_type === 'boolean' ? entry.value : undefined}>{entry.value}</div>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                <span class="px-2 py-1 bg-gray-100 dark:bg-gray-600 rounded text-xs">{entry.value_type}</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{entry.version}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                {new Date(entry.updated_at).toLocaleString()}
              </td>
              {#if canModify()}
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm space-x-3">
                  <button
                    onclick={() => handleViewEntry(entry)}
                    class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 font-medium"
                  >
                    View
                  </button>
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
              {:else}
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                  <button
                    onclick={() => handleViewEntry(entry)}
                    class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 font-medium"
                  >
                    View
                  </button>
                </td>
              {/if}
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
      class="fixed left-1/2 top-1/2 z-50 max-h-[90vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl overflow-y-auto"
    >
      <h2 use:melt={$title} class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        Add New Entry
      </h2>

      {#if formError}
        <div class="mb-3 p-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded text-sm">{formError}</div>
      {/if}

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
              <option value="list">List</option>
              <option value="yaml">YAML</option>
              <option value="toml">TOML</option>
              <option value="hocon">HOCON</option>
            </select>
          </div>
          
          {#if newEntryValueType === 'boolean'}
            <div>
              <label for="new-bool-value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Value *</label>
              <select id="new-bool-value" bind:value={newEntryValue} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm dark:bg-gray-700 dark:text-white">
                <option value="true">true</option>
                <option value="false">false</option>
              </select>
            </div>
          {:else if newEntryValueType === 'list'}
            <div>
              <label for="new-list-item" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                List Items *
              </label>
              <div class="space-y-2">
                <div class="flex gap-2">
                  <input
                    id="new-list-item"
                    type="text"
                    bind:value={newEntryListItemInput}
                    placeholder="Enter list item"
                    onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addNewListItem())}
                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  />
                  <button
                    type="button"
                    onclick={addNewListItem}
                    class="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                  >
                    Add
                  </button>
                </div>
                {#if newEntryListItems.length > 0}
                  <div class="flex flex-wrap gap-1">
                    {#each newEntryListItems as item, index}
                      <span class="flex items-center gap-1 px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs">
                        {item}
                        <button type="button" onclick={() => removeNewListItem(index)} class="text-red-500 hover:text-red-700 ml-1">✕</button>
                      </span>
                    {/each}
                  </div>
                {:else}
                  <p class="text-sm text-gray-500 dark:text-gray-400">No items added yet</p>
                {/if}
              </div>
            </div>
          {:else if usesMonaco(newEntryValueType)}
            <div>
              <p class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Value *</p>
              <MonacoEditor bind:value={newEntryValue} language={monacoLang(newEntryValueType)} height="200px" />
            </div>
          {:else}
            <div>
              <label for="value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Value *
              </label>
              <input
                id="value"
                type={newEntryValueType === 'number' ? 'number' : 'text'}
                bind:value={newEntryValue}
                placeholder="Enter value"
                required
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
            </div>
          {/if}
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
      class="fixed left-1/2 top-1/2 z-50 max-h-[90vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl overflow-y-auto"
    >
      <h2 use:melt={$editTitle} class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        Edit Entry
      </h2>

      {#if formError}
        <div class="mb-3 p-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded text-sm">{formError}</div>
      {/if}

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
              <option value="list">List</option>
              <option value="yaml">YAML</option>
              <option value="toml">TOML</option>
              <option value="hocon">HOCON</option>
            </select>
          </div>

          {#if editEntryValueType === 'boolean'}
            <div>
              <label for="edit-bool-value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Value *</label>
              <select id="edit-bool-value" bind:value={editEntryValue} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm dark:bg-gray-700 dark:text-white">
                <option value="true">true</option>
                <option value="false">false</option>
              </select>
            </div>
          {:else if editEntryValueType === 'list'}
            <div>
              <label for="edit-list-item" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                List Items *
              </label>
              <div class="space-y-2">
                <div class="flex gap-2">
                  <input
                    id="edit-list-item"
                    type="text"
                    bind:value={editEntryListItemInput}
                    placeholder="Enter list item"
                    onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addEditListItem())}
                    class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  />
                  <button
                    type="button"
                    onclick={addEditListItem}
                    class="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                  >
                    Add
                  </button>
                </div>
                {#if editEntryListItems.length > 0}
                  <div class="flex flex-wrap gap-1">
                    {#each editEntryListItems as item, index}
                      <span class="flex items-center gap-1 px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs">
                        {item}
                        <button type="button" onclick={() => removeEditListItem(index)} class="text-red-500 hover:text-red-700 ml-1">✕</button>
                      </span>
                    {/each}
                  </div>
                {:else}
                  <p class="text-sm text-gray-500 dark:text-gray-400">No items added yet</p>
                {/if}
              </div>
            </div>
          {:else if usesMonaco(editEntryValueType)}
            <div>
              <p class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Value *</p>
              <MonacoEditor bind:value={editEntryValue} language={monacoLang(editEntryValueType)} height="200px" />
            </div>
          {:else}
            <div>
              <label for="edit-value" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Value *
              </label>
              <input
                id="edit-value"
                type={editEntryValueType === 'number' ? 'number' : 'text'}
                bind:value={editEntryValue}
                placeholder="Enter value"
                required
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              />
            </div>
          {/if}
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

<!-- Share Space Dialog -->
<div use:melt={$sharePortalled}>
  {#if $shareOpen}
    <div use:melt={$shareOverlay} class="fixed inset-0 z-40 bg-black/50"></div>
    <div
      use:melt={$shareContent}
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl overflow-y-auto"
    >
      <h2 use:melt={$shareTitle} class="text-xl font-semibold text-gray-900 dark:text-white mb-1">
        Share "{space?.name}"
      </h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">Manage who has access to this space.</p>

      {#if shareError}
        <div class="mb-3 p-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded text-sm">{shareError}</div>
      {/if}

      <!-- Add share form -->
      <div class="flex gap-2 mb-4">
        <div class="relative flex-1">
          <input
            type="text"
            value={shareUsername}
            oninput={onShareUsernameInput}
            onblur={() => setTimeout(() => { suggestionsOpen = false; }, 150)}
            placeholder="Username"
            autocomplete="off"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
          />
          {#if suggestionsOpen}
            <ul class="absolute z-50 left-0 right-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg overflow-hidden">
              {#each userSuggestions as suggestion}
                <li>
                  <button
                    type="button"
                    onmousedown={() => selectSuggestion(suggestion)}
                    class="w-full text-left px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
                  >
                    <span class="text-sm font-medium text-gray-900 dark:text-white">{suggestion.username}</span>
                    {#if suggestion.email}
                      <span class="text-xs text-gray-400 dark:text-gray-500">{suggestion.email}</span>
                    {/if}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
        <select
          bind:value={sharePermission}
          class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm dark:bg-gray-700 dark:text-white"
        >
          <option value="readonly">Read only</option>
          <option value="write">Write</option>
          <option value="appendonly">Append only</option>
        </select>
        <button
          onclick={handleAddShare}
          class="px-3 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700 transition-colors whitespace-nowrap"
        >
          Add
        </button>
      </div>

      <!-- Current shares list -->
      {#if sharesLoading}
        <div class="text-center py-4 text-gray-500 text-sm">Loading shares…</div>
      {:else if shares.length === 0}
        <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">No shares yet.</p>
      {:else}
        <ul class="space-y-2">
          {#each shares as share (share.user_id)}
            <li class="flex items-center justify-between bg-gray-50 dark:bg-gray-700 px-3 py-2 rounded-lg">
              <span class="text-sm font-medium text-gray-900 dark:text-white">{share.username}</span>
              <div class="flex items-center gap-2">
                <span class="px-2 py-0.5 text-xs rounded-full font-medium {share.permission === 'write' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : share.permission === 'appendonly' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300'}">{share.permission}</span>
                <button
                  onclick={() => handleRemoveShare(share.user_id)}
                  class="text-red-500 hover:text-red-700 text-xs"
                  aria-label="Remove"
                >✕</button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="flex justify-end mt-6">
        <button use:melt={$shareClose}
          class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors">
          Close
        </button>
      </div>

      <button use:melt={$shareClose} class="absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" aria-label="Close">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>

<!-- View Entry Dialog -->
<div use:melt={$viewPortalled}>
  {#if $viewOpen && viewingEntry}
    <div use:melt={$viewOverlay} class="fixed inset-0 z-40 bg-black/50"></div>
    <div
      use:melt={$viewContent}
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white dark:bg-gray-800 p-6 shadow-xl overflow-y-auto"
    >
      <h2 use:melt={$viewTitle} class="text-xl font-semibold text-gray-900 dark:text-white mb-1 font-mono">
        {viewingEntry.key}
      </h2>
      <div class="flex items-center gap-2 mb-4">
        <span class="px-2 py-0.5 text-xs rounded bg-gray-100 dark:bg-gray-600 text-gray-600 dark:text-gray-300 font-mono">{viewingEntry.value_type}</span>
        <span class="text-xs text-gray-400">v{viewingEntry.version}</span>
        {#if viewingEntry.description}
          <span class="text-xs text-gray-500 dark:text-gray-400">— {viewingEntry.description}</span>
        {/if}
      </div>

      <div class="rounded-lg overflow-hidden border border-gray-200 dark:border-gray-600">
        {#if viewingEntry.value_type === 'list'}
          <ul class="divide-y divide-gray-100 dark:divide-gray-700">
            {#each parseListValue(viewingEntry.value) as item, i}
              <li class="px-4 py-2 text-sm font-mono text-gray-800 dark:text-gray-200 flex items-center gap-3">
                <span class="text-gray-400 text-xs w-6 shrink-0">{i + 1}</span>
                {item}
              </li>
            {/each}
          </ul>
        {:else if usesMonaco(viewingEntry.value_type)}
          <MonacoEditor value={viewingEntry.value} language={monacoLang(viewingEntry.value_type)} readonly height="400px" />
        {:else}
          <pre class="p-4 text-sm font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap break-all bg-gray-50 dark:bg-gray-900">{viewingEntry.value}</pre>
        {/if}
      </div>

      <div class="flex justify-end mt-6">
        <button use:melt={$viewClose}
          class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors">
          Close
        </button>
      </div>

      <button use:melt={$viewClose} class="absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" aria-label="Close">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>

<!-- Webhook Management Modal -->
<div use:melt={$webhookPortalled}>
  {#if $webhookOpen}
    <div use:melt={$webhookOverlay} class="fixed inset-0 bg-black/50 z-40"></div>
    <div use:melt={$webhookContent}
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white dark:bg-gray-800 shadow-2xl p-6 focus:outline-none"
    >
      <h2 use:melt={$webhookTitle} class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Webhooks</h2>

      {#if webhookError}
        <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded-lg text-sm text-red-700 dark:text-red-300">
          {webhookError}
        </div>
      {/if}

      <!-- Add new webhook -->
      <div class="mb-6 space-y-2">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">URL</label>
        <input
          type="url"
          bind:value={newWebhookUrl}
          placeholder="https://example.com/hook"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          onkeydown={(e) => { if (e.key === 'Enter') handleCreateWebhook(); }}
        />
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">Secret (optional)</label>
        <input
          type="text"
          bind:value={newWebhookSecret}
          placeholder="Sent as X-Webhook-Secret header"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          onclick={handleCreateWebhook}
          disabled={!newWebhookUrl.trim()}
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm font-medium"
        >
          Add Webhook
        </button>
      </div>

      <!-- Existing webhooks -->
      {#if webhooksLoading}
        <p class="text-sm text-gray-500 dark:text-gray-400">Loading...</p>
      {:else if webhooks.length === 0}
        <p class="text-sm text-gray-500 dark:text-gray-400 italic">No webhooks configured.</p>
      {:else}
        <ul class="divide-y divide-gray-100 dark:divide-gray-700 rounded-lg border border-gray-200 dark:border-gray-600 overflow-hidden">
          {#each webhooks as wh}
            <li class="flex items-start gap-3 px-4 py-3 bg-white dark:bg-gray-750">
              <div class="flex-1 min-w-0">
                <p class="text-sm font-mono text-gray-800 dark:text-gray-200 truncate">{wh.url}</p>
                <p class="text-xs text-gray-400 mt-0.5">
                  {wh.has_secret ? '🔒 secret set' : 'no secret'} &bull; added {new Date(wh.created_at).toLocaleDateString()}
                </p>
              </div>
              <button
                onclick={() => handleDeleteWebhook(wh.id)}
                class="text-red-500 hover:text-red-700 dark:hover:text-red-400 text-xs px-2 py-1 rounded hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors shrink-0"
              >
                Remove
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="flex justify-end mt-6">
        <button use:melt={$webhookClose}
          class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-sm">
          Close
        </button>
      </div>

      <button use:melt={$webhookClose} class="absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" aria-label="Close">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>
