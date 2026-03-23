<script lang="ts">
  import { navigate } from 'svelte-routing';
  import { authState, logout } from '../lib/auth';
  
  function handleHomeClick(e: MouseEvent) {
    e.preventDefault();
    navigate('/');
  }

  function handleLogout() {
    logout();
    navigate('/login');
  }
</script>

<nav class="bg-white dark:bg-gray-800 shadow-sm">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <div class="flex items-center">
        <a href="/" onclick={handleHomeClick} class="flex items-center space-x-2">
          <svg class="h-8 w-8 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
          </svg>
          <span class="text-xl font-bold text-gray-900 dark:text-white">Rugistry</span>
        </a>
      </div>
      <div class="flex items-center space-x-4">
        <a 
          href="/" 
          onclick={handleHomeClick}
          class="px-3 py-2 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        >
          Spaces
        </a>
        
        {#if $authState.authenticated}
          <div class="flex items-center space-x-3">
            <span class="text-sm text-gray-700 dark:text-gray-300">
              {$authState.username}
            </span>
            <button
              onclick={handleLogout}
              class="px-3 py-2 rounded-md text-sm font-medium text-white bg-red-600 hover:bg-red-700 transition-colors"
            >
              Logout
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
</nav>
