import React, { useCallback, useEffect, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { getAuth, GoogleAuthProvider, signInWithPopup, signOut } from "firebase/auth";
import { app } from "@/lib/firebase";

export default function Topbar() {
  const location = useLocation();
  const navigate = useNavigate();
  const [jwt, setJwt] = useState<string | null>(() => localStorage.getItem("jwt"));
  const [signingIn, setSigningIn] = useState(false);

  const isActive = (path: string) => location.pathname === path;

  useEffect(() => {
    const handler = () => setJwt(localStorage.getItem("jwt"));
    window.addEventListener("storage", handler);
    return () => window.removeEventListener("storage", handler);
  }, []);

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
        body: JSON.stringify({ firebaseToken: idToken })
      });
      if (!resp.ok) throw new Error(await resp.text());
      const data = await resp.json();
      localStorage.setItem("jwt", data.token);
      setJwt(data.token);
      // stay on page; optionally navigate to account
    } catch (e: any) {
      // Ignore benign popup closures
      if (e?.code === 'auth/popup-closed-by-user') return;
      console.error(e);
    } finally {
      setSigningIn(false);
    }
  }, []);

  const handleSignOut = useCallback(async () => {
    try {
      const auth = getAuth(app);
      await signOut(auth);
    } catch {}
    localStorage.removeItem("jwt");
    setJwt(null);
    // If on protected nav like /upload, bounce to home
    if (location.pathname === "/upload") navigate("/");
  }, [location.pathname, navigate]);

  return (
    <header className="sticky top-0 z-30 bg-white/80 backdrop-blur border-b border-black">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-14 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Link to="/" className="inline-flex items-center gap-2 select-none">
            <span className="block w-3 h-3 bg-black" />
            <span className="font-['Press_Start_2P'] text-sm pixel-title">Beatblock Browser</span>
          </Link>
        </div>
        <nav className="flex items-center gap-2">
          {!!jwt && (
            <Link
              to="/upload"
              className={`pixel-btn px-3 py-2 text-xs font-['Press_Start_2P'] bg-white ${
                isActive("/upload") ? "bg-purple-600 text-black" : "hover:bg-gray-50"
              }`}
              aria-current={isActive("/upload") ? "page" : undefined}
            >
              Upload
            </Link>
          )}
          {!!jwt && (
            <Link
              to="/account"
              className={`pixel-btn px-3 py-2 text-xs font-['Press_Start_2P'] bg-white ${
                isActive("/account") ? "bg-purple-600 text-black" : "hover:bg-gray-50"
              }`}
              aria-current={isActive("/account") ? "page" : undefined}
            >
              Account
            </Link>
          )}
          {!jwt ? (
            <button
              type="button"
              className="pixel-btn px-3 py-2 text-xs font-['Press_Start_2P'] bg-black text-white"
              onClick={handleSignIn}
              disabled={signingIn}
            >
              {signingIn ? "Signing in..." : "Sign in"}
            </button>
          ) : (
            <button
              type="button"
              className="pixel-btn px-3 py-2 text-xs font-['Press_Start_2P'] bg-white hover:bg-gray-50"
              onClick={handleSignOut}
            >
              Sign out
            </button>
          )}
        </nav>
      </div>
    </header>
  );
}
