export const API_BASE = process.env.API_BASE || '';

export function apiFetch(input: string, init?: RequestInit) {
  const url = `${API_BASE}${input}`;
  return fetch(url, init);
}

export function apiFetchAuth(input: string, init?: RequestInit) {
  const url = `${API_BASE}${input}`;
  const token = (typeof window !== 'undefined') ? localStorage.getItem('jwt') : null;
  const headers: Record<string, string> = {
    ...(init && init.headers ? (init.headers as Record<string, string>) : {}),
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  return fetch(url, { ...(init || {}), headers });
}
