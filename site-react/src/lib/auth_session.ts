import { onIdTokenChanged } from "firebase/auth";
import { auth } from "@/lib/firebase";
import { API_BASE } from "@/lib/api";

let started = false;
let refreshTimer: number | null = null;

export function initAuthSession() {
  if (started) return;
  started = true;

  const schedule = () => {
    if (refreshTimer) window.clearTimeout(refreshTimer);
    // Refresh every 10 minutes; Firebase will only mint a new ID token when needed
    refreshTimer = window.setTimeout(async () => {
      try {
        const user = auth.currentUser;
        if (user) {
          const idToken = await user.getIdToken(false);
          await exchange(idToken);
        }
      } finally {
        schedule();
      }
    }, 10 * 60 * 1000);
  };

  const exchange = async (idToken: string) => {
    try {
      const resp = await fetch(`${API_BASE}/api/signin`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ firebaseToken: idToken }),
      });
      if (resp.ok) {
        const data = (await resp.json()) as { token: string };
        localStorage.setItem("jwt", data.token);
      }
    } catch {
      // ignore; apiFetchAuth will handle refresh on demand
    }
  };

  onIdTokenChanged(auth, async (user) => {
    if (!user) {
      try { localStorage.removeItem("jwt"); } catch {}
      return;
    }
    try {
      const idToken = await user.getIdToken(false);
      await exchange(idToken);
    } catch {
      // ignore; apiFetchAuth will handle refresh on demand
    }
  });

  // Kick off periodic background refresh
  schedule();
}
