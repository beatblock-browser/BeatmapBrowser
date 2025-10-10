import React, { useCallback, useEffect, useState } from "react";
import { getAuth, GoogleAuthProvider, signInWithPopup, signOut } from "firebase/auth";
import { app } from "@/lib/firebase";
import { apiFetchAuth } from "@/lib/api";

type MeResponse = {
  sub: string;
  iat: number;
  exp: number;
};

export default function AccountPage() {
  const [jwt, setJwt] = useState<string | null>(() => localStorage.getItem("jwt"));
  const [me, setMe] = useState<MeResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [signingIn, setSigningIn] = useState(false);

  useEffect(() => {
    const load = async () => {
      if (!jwt) { setMe(null); return; }
      setLoading(true);
      setError(null);
      try {
        const attempt = async (): Promise<boolean> => {
          const res = await apiFetchAuth("/api/me");
          if (res.ok) {
            const data = (await res.json()) as MeResponse;
            setMe(data);
            return true;
          }
          // If token expired, try to refresh from Firebase
          if (res.status === 401) {
            const text = await res.text();
            if (text.includes("ExpiredSignature") || text.toLowerCase().includes("expired")) {
              try {
                const auth = getAuth(app);
                const user = auth.currentUser;
                if (!user) throw new Error("Not signed in");
                const idToken = await user.getIdToken(true);
                const resp = await fetch("/api/signin", {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify({ firebaseToken: idToken }),
                });
                if (!resp.ok) throw new Error(await resp.text());
                const data = await resp.json();
                localStorage.setItem("jwt", data.token);
                setJwt(data.token);
                return false; // signal to retry
              } catch (e: any) {
                // Refresh failed; clear session
                localStorage.removeItem("jwt");
                setJwt(null);
                setMe(null);
                setError("Session expired. Please sign in again.");
                return true;
              }
            }
          }
          throw new Error(await res.text());
        };
        // First attempt; if it returns false, retry once after refresh
        const done = await attempt();
        if (!done) {
          const final = await apiFetchAuth("/api/me");
          if (!final.ok) throw new Error(await final.text());
          const data2 = (await final.json()) as MeResponse;
          setMe(data2);
        }
      } catch (e: any) {
        setError(e?.message || "Failed to load account");
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [jwt]);

  const handleSignIn = useCallback(async () => {
    setSigningIn(true);
    try {
      const auth = getAuth(app);
      const provider = new GoogleAuthProvider();
      const result = await signInWithPopup(auth, provider);
      const idToken = await result.user.getIdToken(true);
      const resp = await fetch("/api/signin", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ firebaseToken: idToken }),
      });
      if (!resp.ok) throw new Error(await resp.text());
      const data = await resp.json();
      localStorage.setItem("jwt", data.token);
      setJwt(data.token);
    } catch (e: any) {
      setError(e?.message || "Failed to sign in");
    } finally {
      setSigningIn(false);
    }
  }, []);

  const handleSignOut = useCallback(async () => {
    try { await signOut(getAuth(app)); } catch {}
    localStorage.removeItem("jwt");
    setJwt(null);
    setMe(null);
  }, []);

  return (
    <main className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <h1 className="font-['Press_Start_2P'] text-xl mb-4 pixel-title">Account</h1>

      {error && (
        <div className="mb-4 border border-red-500 bg-red-100 text-red-800 px-3 py-2 font-['Press_Start_2P'] text-sm">{error}</div>
      )}

      {!jwt ? (
        <div className="pixel-panel p-4 bg-white">
          <p className="text-sm mb-3">You are not signed in.</p>
          <button
            type="button"
            className="pixel-btn bg-black text-white px-3 py-2 text-xs font-['Press_Start_2P']"
            onClick={handleSignIn}
            disabled={signingIn}
          >
            {signingIn ? "Signing in..." : "Sign in with Google"}
          </button>
        </div>
      ) : (
        <div className="pixel-panel p-4 bg-white">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-['Press_Start_2P'] text-sm">User</div>
              {loading ? (
                <div className="text-xs text-gray-600">Loading…</div>
              ) : me ? (
                <div className="text-xs">
                  <div><span className="text-gray-600">ID:</span> {me.sub}</div>
                  <div><span className="text-gray-600">JWT exp:</span> {new Date(me.exp * 1000).toLocaleString()}</div>
                </div>
              ) : (
                <div className="text-xs text-gray-600">No profile loaded</div>
              )}
            </div>
            <button
              type="button"
              className="pixel-btn bg-white px-3 py-2 text-xs font-['Press_Start_2P']"
              onClick={handleSignOut}
            >
              Sign out
            </button>
          </div>
        </div>
      )}
    </main>
  );
}
