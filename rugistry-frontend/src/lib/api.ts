const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export interface Space {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface RegistryEntry {
  id: string;
  space_id: string;
  key: string;
  value: string;
  value_type: 'string' | 'number' | 'boolean' | 'json';
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
  value_type: 'string' | 'number' | 'boolean' | 'json';
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
  const response = await fetch(`${API_URL}/api/spaces`);
  if (!response.ok) throw new Error('Failed to fetch spaces');
  return response.json();
}

export async function getSpace(id: string): Promise<Space> {
  const response = await fetch(`${API_URL}/api/spaces/${id}`);
  if (!response.ok) throw new Error('Failed to fetch space');
  return response.json();
}

export async function createSpace(data: CreateSpaceRequest): Promise<Space> {
  const response = await fetch(`${API_URL}/api/spaces`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to create space');
  return response.json();
}

export async function updateSpace(id: string, data: Partial<CreateSpaceRequest>): Promise<Space> {
  const response = await fetch(`${API_URL}/api/spaces/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update space');
  return response.json();
}

export async function deleteSpace(id: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/spaces/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete space');
}

// Registry Entry API
export async function getEntries(spaceId: string): Promise<RegistryEntry[]> {
  const response = await fetch(`${API_URL}/api/spaces/${spaceId}/entries`);
  if (!response.ok) throw new Error('Failed to fetch entries');
  return response.json();
}

export async function createEntry(spaceId: string, data: CreateEntryRequest): Promise<RegistryEntry> {
  const response = await fetch(`${API_URL}/api/entries`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      space_id: spaceId,
      ...data
    }),
  });
  if (!response.ok) throw new Error('Failed to create entry');
  return response.json();
}

export async function updateEntry(spaceId: string, entryId: string, data: Partial<CreateEntryRequest>): Promise<RegistryEntry> {
  const response = await fetch(`${API_URL}/api/entries/${entryId}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update entry');
  return response.json();
}

export async function deleteEntry(spaceId: string, entryId: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/entries/${entryId}`, {
    method: 'DELETE',
  });
  if (!response.ok) throw new Error('Failed to delete entry');
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
  
  const ws = new WebSocket(`${wsUrl}/api/ws/${spaceId}${queryString}`);
  
  ws.onmessage = (event) => {
    const notification: ChangeNotification = JSON.parse(event.data);
    onMessage(notification);
  };
  
  return ws;
}
