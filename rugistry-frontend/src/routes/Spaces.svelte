<script lang="ts">
  import { onMount } from 'svelte';
  import { createDialog, melt } from '@melt-ui/svelte';
  import { navigate } from 'svelte-routing';
  import {
    getSpaces, createSpace, deleteSpace, getShares, addShare, removeShare, searchUsers,
    type Space, type SpaceShare, type UserSearchResult,
  } from '../lib/api';

  let spaces: Space[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let newSpaceName = $state('');
  let newSpaceDescription = $state('');
  let createError = $state('');

  // Share modal state
  let sharingSpace: Space | null = $state(null);
  let shares: SpaceShare[] = $state([]);
  let sharesLoading = $state(false);
  let shareUsername = $state('');
  let sharePermission = $state('readonly');
  let shareError = $state('');

  // Username autocomplete state
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

  const {
    elements: { trigger, overlay, content, title, close, portalled },
    states: { open }
  } = createDialog({ forceVisible: true });

  const {
    elements: { trigger: shareTrigger, overlay: shareOverlay, content: shareContent, title: shareTitle, close: shareClose, portalled: sharePortalled },
    states: { open: shareOpen }
  } = createDialog({ forceVisible: true });

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
    createError = '';
    try {
      await createSpace({ name: newSpaceName, description: newSpaceDescription || undefined });
      open.set(false);
      newSpaceName = '';
      newSpaceDescription = '';
      createError = '';
      await loadSpaces();
    } catch (e) {
      createError = e instanceof Error ? e.message : 'Failed to create space';
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

  async function openShareModal(space: Space) {
    sharingSpace = space;
    shareError = '';
    shareUsername = '';
    sharePermission = 'readonly';
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
    if (!sharingSpace || !shareUsername.trim()) return;
    try {
      await addShare(sharingSpace.id, shareUsername.trim(), sharePermission);
      shareUsername = ''; userSuggestions = []; suggestionsOpen = false;
      shares = await getShares(sharingSpace.id);
      shareError = '';
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Failed to add share';
    }
  }

  async function handleRemoveShare(userId: string) {
    if (!sharingSpace) return;
    try {
      await removeShare(sharingSpace.id, userId);
      shares = await getShares(sharingSpace.id);
    } catch (e) {
      shareError = e instanceof Error ? e.message : 'Failed to remove share';
    }
  }

  function viewSpace(id: string) { navigate(`/spaces/${id}`); }

  function permissionBadgeClass(perm: string | null): string {
    if (perm === null) return 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300';
    if (perm === 'readonly') return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300';
    if (perm === 'write') return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
    if (perm === 'appendonly') return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300';
    return '';
  }

  function permissionLabel(perm: string | null): string {
    if (perm === null) return 'owner';
    return perm;
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
            <div class="flex items-start justify-between mb-2">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{space.name}</h3>
              <span class="ml-2 px-2 py-0.5 text-xs rounded-full font-medium {permissionBadgeClass(space.permission)}">
                {permissionLabel(space.permission)}
              </span>
            </div>
            {#if space.description}
              <p class="text-gray-600 dark:text-gray-400 text-sm mb-4">{space.description}</p>
            {:else}
              <p class="text-gray-400 dark:text-gray-500 text-sm mb-4 italic">No description</p>
            {/if}
            <p class="text-xs text-gray-500 dark:text-gray-500">
              Created: {new Date(space.created_at).toLocaleDateString()}
            </p>
          </div>
          <div class="px-5 py-3 bg-gray-50 dark:bg-gray-700 flex justify-end gap-2 flex-wrap">
            <button
              onclick={() => viewSpace(space.id)}
              class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              View
            </button>
            {#if space.permission === null}
              <button
                onclick={() => openShareModal(space)}
                class="px-3 py-1.5 text-sm bg-indigo-600 text-white rounded hover:bg-indigo-700 transition-colors"
              >
                Share
              </button>
              <button
                onclick={() => handleDeleteSpace(space.id)}
                class="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
              >
                Delete
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Create Space Dialog -->
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
          {#if createError}
            <div class="p-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded text-sm">{createError}</div>
          {/if}
          <div>
            <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name *</label>
            <input id="name" type="text" bind:value={newSpaceName} placeholder="Enter space name" required
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
            <textarea id="description" bind:value={newSpaceDescription} placeholder="Enter description (optional)" rows="3"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"></textarea>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button type="button" use:melt={$close}
            class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors">
            Cancel
          </button>
          <button type="submit" class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
            Create
          </button>
        </div>
      </form>
      <button use:melt={$close} class="absolute right-4 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" aria-label="Close">
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
        Share "{sharingSpace?.name}"
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
                <span class="px-2 py-0.5 text-xs rounded-full font-medium {permissionBadgeClass(share.permission)}">{share.permission}</span>
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

