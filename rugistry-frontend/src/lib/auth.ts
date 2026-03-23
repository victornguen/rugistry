import { writable } from 'svelte/store';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export interface AuthState {
  authenticated: boolean;
  token: string | null;
  username: string | null;
}

export const authState = writable<AuthState>({
  authenticated: false,
  token: null,
  username: null,
});

export async function register(username: string, password: string, email: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/auth/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password, email }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Registration failed');
  }

  const data = await response.json();
  setAuthToken(data.token, data.username);
}

export async function login(username: string, password: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Login failed');
  }

  const data = await response.json();
  setAuthToken(data.token, data.username);
}

export function logout(): void {
  localStorage.removeItem('auth_token');
  localStorage.removeItem('auth_username');
  authState.set({ authenticated: false, token: null, username: null });
}

export function initAuth(): void {
  const token = localStorage.getItem('auth_token');
  const username = localStorage.getItem('auth_username');

  if (token && username) {
    authState.set({ authenticated: true, token, username });
  }
}

function setAuthToken(token: string, username: string): void {
  localStorage.setItem('auth_token', token);
  localStorage.setItem('auth_username', username);
  authState.set({ authenticated: true, token, username });
}

export function getToken(): string | null {
  return localStorage.getItem('auth_token');
}
