// Read env from multiple sources without assuming Node's process exists in the browser
const readEnv = (key: string): string => {
  try {
    const g: any = (typeof globalThis !== 'undefined') ? (globalThis as any) : {};
    const fromGlobal = g?.__ENV__?.[key] || g?.[key];
    if (typeof fromGlobal === 'string') return fromGlobal;
  } catch {}
  try {
    const ie: any = (typeof import.meta !== 'undefined') ? (import.meta as any).env : undefined;
    const v = ie?.[key] || ie?.[`VITE_${key}`] || ie?.[`RSBUILD_${key}`];
    if (typeof v === 'string') return v;
  } catch {}
  try {
    const p: any = (typeof process !== 'undefined') ? (process as any) : undefined;
    const v = p?.env?.[key];
    if (typeof v === 'string') return v;
  } catch {}
  return '';
};

export const API_BASE = readEnv('API_BASE');
export const R2_PUBLIC_URL = readEnv('R2_PUBLIC_URL');

export function apiFetch(input: string, init?: RequestInit) {
  const url = `${API_BASE}${input}`;
  return fetch(url, { credentials: 'include', ...(init || {}) });
}

import { getAuth } from 'firebase/auth';
import { app } from '@/lib/firebase';

export async function apiFetchAuth(input: string, init?: RequestInit): Promise<Response> {
  const url = `${API_BASE}${input}`;
  // If no backend JWT yet, try to bootstrap it from Firebase
  try {
    const hasJwt = (typeof window !== 'undefined') ? !!localStorage.getItem('jwt') : false;
    if (!hasJwt) {
      const auth = getAuth(app);
      const user = auth.currentUser;
      if (user) {
        const idToken = await user.getIdToken(true);
        const signin = await fetch(`${API_BASE}/api/signin`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ firebaseToken: idToken }),
          signal: init?.signal,
          credentials: 'include',
        });
        if (signin.ok) {
          const data = await signin.json() as { token: string };
          localStorage.setItem('jwt', data.token);
        }
      }
    }
  } catch {}
  const doFetch = async (): Promise<Response> => {
    const token = (typeof window !== 'undefined') ? localStorage.getItem('jwt') : null;
    const headers: Record<string, string> = {
      ...(init && init.headers ? (init.headers as Record<string, string>) : {}),
    };
    if (token) headers['Authorization'] = `Bearer ${token}`;
    return fetch(url, { credentials: 'include', ...(init || {}), headers });
  };

  let res = await doFetch();
  // Handle expired token across 400/401/403 with ExpiredSignature markers
  if (res.status === 400 || res.status === 401 || res.status === 403) {
    let bodyText = '';
    try { bodyText = await res.clone().text(); } catch {}
    const lower = bodyText.toLowerCase();
    const expired = bodyText.includes('ExpiredSignature') || lower.includes('expired');
    if (expired) {
      try {
        const auth = getAuth(app);
        const user = auth.currentUser;
        if (!user) throw new Error('not signed in');
        const idToken = await user.getIdToken(true);
        const signin = await fetch(`${API_BASE}/api/signin`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ firebaseToken: idToken }),
          signal: init?.signal,
        });
        if (!signin.ok) throw new Error(await signin.text());
        const data = await signin.json() as { token: string };
        localStorage.setItem('jwt', data.token);
        // retry once with refreshed JWT
        res = await doFetch();
      } catch {
        try { localStorage.removeItem('jwt'); } catch {}
        // bubble up as 401 to force re-auth in UI
        return new Response('Unauthorized: expired and refresh failed', { status: 401 });
      }
    }
  }
  return res;
}
