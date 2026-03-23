import { getToken, authState } from './auth';
import { get } from 'svelte/store';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

function getHeaders(): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  
  // Get token from localStorage
  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  return headers;
}

export interface Space {
  id: string;
  name: string;
  description: string | null;
  owner_id: string | null;
  /** null = current user is owner; "readonly"/"write"/"appendonly" = shared access */
  permission: string | null;
  created_at: string;
  updated_at: string;
}

export interface SpaceShare {
  user_id: string;
  username: string;
  permission: string;
  created_at: string;
}

export interface RegistryEntry {
  id: string;
  space_id: string;
  key: string;
  value: string;
  value_type: 'string' | 'number' | 'boolean' | 'json' | 'list' | 'hocon' | 'toml' | 'yaml';
  description: string | null;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface CreateSpaceRequest {
  name: string;
  description?: string;
}

export interface CreateEntryRequest {
  key: string;
  value: string;
  value_type: 'string' | 'number' | 'boolean' | 'json' | 'list' | 'hocon' | 'toml' | 'yaml';
  description?: string;
}

export interface ChangeNotification {
  event_type: 'created' | 'updated' | 'deleted';
  space_id: string;
  entry_id?: string;
  key?: string;
  timestamp: string;
}

// Space API
export async function getSpaces(): Promise<Space[]> {
  const response = await fetch(`${API_URL}/api/v1/spaces`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch spaces');
  return response.json();
}

export async function getSpace(id: string): Promise<Space> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${id}`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch space');
  return response.json();
}

export async function createSpace(data: CreateSpaceRequest): Promise<Space> {
  const response = await fetch(`${API_URL}/api/v1/spaces`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create space');
  return response.json();
}

export async function updateSpace(id: string, data: Partial<CreateSpaceRequest>): Promise<Space> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${id}`, {
    method: 'PUT',
    headers: getHeaders(),
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update space');
  return response.json();
}

export async function deleteSpace(id: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${id}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) throw new Error('Failed to delete space');
}

// Space sharing API
export async function getShares(spaceId: string): Promise<SpaceShare[]> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${spaceId}/shares`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch shares');
  return response.json();
}

export async function addShare(spaceId: string, username: string, permission: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${spaceId}/shares`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ username, permission }),
  });
  if (!response.ok) {
    const err = await response.json().catch(() => ({}));
    throw new Error(err.error || 'Failed to share space');
  }
}

export async function removeShare(spaceId: string, userId: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${spaceId}/shares/${userId}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) throw new Error('Failed to remove share');
}

export interface UserSearchResult {
  username: string;
  email: string | null;
}

export async function searchUsers(query: string): Promise<UserSearchResult[]> {
  if (!query.trim()) return [];
  const response = await fetch(`${API_URL}/api/v1/users/search?q=${encodeURIComponent(query)}`, {
    headers: getHeaders(),
  });
  if (!response.ok) return [];
  return response.json();
}

// Registry Entry API
export async function getEntries(spaceId: string): Promise<RegistryEntry[]> {
  const response = await fetch(`${API_URL}/api/v1/spaces/${spaceId}/entries`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch entries');
  return response.json();
}

export async function createEntry(spaceId: string, data: CreateEntryRequest): Promise<RegistryEntry> {
  const response = await fetch(`${API_URL}/api/v1/entries`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({
      space_id: spaceId,
      ...data
    }),
  });
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.error ?? 'Failed to create entry');
  }
  return response.json();
}

export async function updateEntry(spaceId: string, entryId: string, data: Partial<CreateEntryRequest>): Promise<RegistryEntry> {
  const response = await fetch(`${API_URL}/api/v1/entries/${entryId}`, {
    method: 'PUT',
    headers: getHeaders(),
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.error ?? 'Failed to update entry');
  }
  return response.json();
}

export async function deleteEntry(spaceId: string, entryId: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/entries/${entryId}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.error ?? 'Failed to delete entry');
  }
}

// WebSocket connection
export interface WebSocketOptions {
  /** Filter by exact key match */
  key?: string;
  /** Filter by regex pattern */
  pattern?: string;
}

export function connectWebSocket(
  spaceId: string, 
  onMessage: (notification: ChangeNotification) => void,
  options?: WebSocketOptions
): WebSocket {
  const wsUrl = API_URL.replace('http', 'ws');
  
  // Build query parameters
  const params = new URLSearchParams();
  if (options?.key) {
    params.append('key', options.key);
  }
  if (options?.pattern) {
    params.append('pattern', options.pattern);
  }
  const queryString = params.toString() ? `?${params.toString()}` : '';
  
  const ws = new WebSocket(`${wsUrl}/api/v1/ws/${spaceId}${queryString}`);
  
  ws.onmessage = (event) => {
    const notification: ChangeNotification = JSON.parse(event.data);
    onMessage(notification);
  };
  
  return ws;
}
