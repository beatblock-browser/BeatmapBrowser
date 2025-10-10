import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { getAuth, GoogleAuthProvider, signInWithPopup } from "firebase/auth";
import { app } from "@/lib/firebase";
import { apiFetchAuth, API_BASE } from "@/lib/api";
import { unzipSync, zipSync } from "fflate";
import SongCard from "@/components/SongCard";
import { BeatMap } from "@/schema";
import { sanitizeText } from "@/lib/sanitize";

type UploadResponse = {
  id: string;
  song: string;
  artist: string;
  charter: string;
  charter_uid: string;
  download_url: string;
  thumbnail_url: string;
  updated?: boolean;
};

export default function UploadPage() {
  const navigate = useNavigate();
  const [jwt, setJwt] = useState<string | null>(() => localStorage.getItem("jwt"));
  const [isSigningIn, setIsSigningIn] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [zipFile, setZipFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [previewMap, setPreviewMap] = useState<BeatMap | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [progress, setProgress] = useState<number>(0);
  const [phase, setPhase] = useState<string>("");
  const abortRef = useRef<AbortController | null>(null);
  const canceledRef = useRef<boolean>(false);
  const [willUpdate, setWillUpdate] = useState<boolean>(false);


  const canSubmit = useMemo(() => {
    return (
      !!zipFile &&
      !!jwt &&
      !submitting
    );
  }, [zipFile, jwt, submitting]);

  // Cleanup preview URL on unmount or when replaced
  useEffect(() => {
    return () => {
      if (previewUrl) {
        try { URL.revokeObjectURL(previewUrl); } catch {}
      }
    };
  }, [previewUrl]);

  const handleGoogleSignIn = useCallback(async () => {
    setError(null);
    setIsSigningIn(true);
    try {
      const auth = getAuth(app);
      const provider = new GoogleAuthProvider();
      const result = await signInWithPopup(auth, provider);
      const idToken = await result.user.getIdToken(true);
      // Exchange Firebase token for backend JWT
      const resp = await fetch("/api/signin", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ firebaseToken: idToken }),
      });
      if (!resp.ok) {
        const msg = await resp.text();
        throw new Error(msg || `Signin failed (${resp.status})`);
      }
      const data = (await resp.json()) as { token: string };
      localStorage.setItem("jwt", data.token);
      setJwt(data.token);
    } catch (e: any) {
      setError(e?.message || "Failed to sign in");
    } finally {
      setIsSigningIn(false);
    }
  }, []);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setSubmitting(true);
    setProgress(0);
    setPhase("Validating");
    setError(null);
    try {
      // Proactively refresh backend JWT from Firebase to avoid ExpiredSignature during upload
      try {
        const auth = getAuth(app);
        let user = auth.currentUser;
        if (!user) {
          // No Firebase session present; prompt Google sign-in
          const provider = new GoogleAuthProvider();
          const result = await signInWithPopup(auth, provider);
          user = result.user;
        }
        if (user) {
          const idToken = await user.getIdToken(true);
          const resp = await fetch(`${API_BASE}/api/signin`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ firebaseToken: idToken }),
          });
          if (resp.ok) {
            const data = (await resp.json()) as { token: string };
            localStorage.setItem("jwt", data.token);
            setJwt(data.token);
          }
        }
      } catch {}

      if (!zipFile) throw new Error("Missing file");
      canceledRef.current = false;
      // Prepare AbortController for the network request
      abortRef.current = new AbortController();

      // Client-side sanitize: enforce same policy as backend verify
      const MAX_COMPRESSED = 200 * 1024 * 1024; // 200 MB
      const MAX_FILES = 2000;
      const MAX_FILE_SIZE = 25 * 1024 * 1024; // 25 MB
      const ALLOWED = new Set(["png","jpg","jpeg","webp","mp3","bmp","ogg","oog","wav","json","md","txt"]);

      const buf = new Uint8Array(await zipFile.arrayBuffer());
      if (buf.byteLength > MAX_COMPRESSED) {
        throw new Error("File is too large (max 200 MB).");
      }
      let entries: Record<string, Uint8Array>;
      try {
        entries = unzipSync(buf);
      } catch {
        throw new Error("Invalid ZIP file.");
      }

      const names = Object.keys(entries);
      if (names.length > MAX_FILES) throw new Error("Too many files in archive.");

      const clean: Record<string, Uint8Array> = {};
      let total = 0;
      for (let i = 0; i < names.length; i++) {
        if (canceledRef.current) throw new Error("Upload canceled");
        const name = names[i];
        // Disallow traversal or absolute paths
        if (name.includes("..") || name.startsWith("/") || name.startsWith("\\")) {
          continue;
        }
        // Allow directories
        if (name.endsWith("/") || name.endsWith("\\")) continue;
        const lower = name.toLowerCase();
        const ext = lower.split(".").pop() || "";
        if (!ALLOWED.has(ext)) continue;
        const data = entries[name];
        if (!data) continue;
        if (data.byteLength > MAX_FILE_SIZE) throw new Error(`File ${name} exceeds per-file size limit.`);
        // Track uncompressed total (same as server verify)
        const next = total + data.byteLength;
        if (Math.max(next, data.byteLength) > (200 * 1024 * 1024) * 2) {
          throw new Error("Uncompressed content too large.");
        }
        total = next;
        clean[name] = data;
        // update progress through entries
        if ((i & 7) === 0) setProgress(Math.floor((i / names.length) * 70));
      }

      if (Object.keys(clean).length === 0) throw new Error("Archive contained no valid files.");

      // Rebuild sanitized ZIP with default deflate
      setPhase("Packing");
      const zipped = zipSync(clean, { level: 6 });
      setProgress(80);
      const sanitized = new File([zipped], zipFile.name.endsWith('.zip') ? zipFile.name : (zipFile.name + '.zip'), { type: "application/zip" });

      const form = new FormData();
      form.set("beatmap", sanitized);

      setPhase("Uploading");
      setProgress(90);
      const doUpload = async () => {
        return apiFetchAuth("/api/upload", {
          method: "POST",
          body: form,
          signal: abortRef.current!.signal,
        });
      };

      let res = await doUpload();
      if (!res.ok) {
        let msg = '';
        try { msg = await res.clone().text(); } catch {}
        const lower = msg.toLowerCase();
        if (res.status === 401 || res.status === 403 || msg.includes('ExpiredSignature') || lower.includes('expired')) {
          // Force refresh and retry once
          try {
            const auth = getAuth(app);
            const user = auth.currentUser;
            if (!user) throw new Error('not signed in');
            const idToken = await user.getIdToken(true);
            const signin = await fetch(`${API_BASE}/api/signin`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ firebaseToken: idToken }),
              signal: abortRef.current!.signal,
            });
            if (signin.ok) {
              const data = await signin.json() as { token: string };
              localStorage.setItem('jwt', data.token);
              setJwt(data.token);
              res = await doUpload();
            }
          } catch {}
        }
      }
      if (!res.ok) {
        const msg = await res.text();
        // Provide clearer message for expired auth
        if (msg.includes('ExpiredSignature') || msg.toLowerCase().includes('expired')) {
          throw new Error('Your session expired. Please sign in again and retry.');
        }
        throw new Error(msg || `Upload failed (${res.status})`);
      }
      setProgress(100);
      setPhase("Done");
      const data = (await res.json()) as UploadResponse;
      navigate(`/song/${data.id}` , { state: data.updated ? { updated: true } : undefined });
    } catch (e: any) {
      setError(e?.message || "Upload failed");
    } finally {
      setSubmitting(false);
      abortRef.current = null;
      setTimeout(() => { setPhase(""); setProgress(0); }, 1500);
    }
  }, [zipFile, canSubmit, navigate]);

  const handleCancel = useCallback(() => {
    canceledRef.current = true;
    if (abortRef.current) {
      try { abortRef.current.abort(); } catch {}
    }
    setPhase("Canceled");
    setProgress(0);
    setSubmitting(false);
  }, []);

  return (
    <main className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <h1 className="font-['Press_Start_2P'] text-xl mb-4 pixel-title">Upload Beatmap</h1>

      {error && (
        <div className="mb-4 border border-red-500 bg-red-100 text-red-800 px-3 py-2 font-['Press_Start_2P'] text-sm">
          {error}
        </div>
      )}
      {willUpdate && (
        <div className="mb-4 border border-green-600 bg-green-100 text-green-800 px-3 py-2 font-['Press_Start_2P'] text-sm">
          This ZIP matches a map you already uploaded. Submitting will update your existing upload.
        </div>
      )}

      {!jwt ? (
        <div className="pixel-panel p-4 bg-white">
          <p className="text-sm mb-3">Sign in to upload a map.</p>
          <button
            type="button"
            className="pixel-btn bg-black text-white px-3 py-2 text-xs font-['Press_Start_2P']"
            onClick={handleGoogleSignIn}
            disabled={isSigningIn}
          >
            {isSigningIn ? "Signing in..." : "Sign in with Google"}
          </button>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="pixel-panel p-4 bg-white">
          <div className="grid grid-cols-1 gap-4">
            <label className="block">
              <span className="block text-xs font-['Press_Start_2P'] mb-1">Beatmap ZIP</span>
              <input
                type="file"
                accept=".zip,application/zip,application/x-zip-compressed"
                className="w-full h-10 px-3 text-[12px] font-['Press_Start_2P'] pixel-control pixel-focus file:pixel-btn file:h-8 file:mt-1"
                onChange={(e) => {
                  const file = e.target.files?.[0] || null;
                  if (!file) { setZipFile(null); return; }
                  // Client-side guard: size and extension check
                  const MAX_COMPRESSED = 200 * 1024 * 1024; // 200MB
                  const name = file.name.toLowerCase();
                  if (!name.endsWith('.zip')) {
                    setError('Please upload a .zip file.');
                    e.currentTarget.value = '';
                    setZipFile(null);
                    // Clear preview
                    if (previewUrl) URL.revokeObjectURL(previewUrl);
                    setPreviewUrl(null);
                    setPreviewMap(null);
                    return;
                  }
                  if (file.size > MAX_COMPRESSED) {
                    setError('File is too large (max 200 MB).');
                    e.currentTarget.value = '';
                    setZipFile(null);
                    if (previewUrl) URL.revokeObjectURL(previewUrl);
                    setPreviewUrl(null);
                    setPreviewMap(null);
                    return;
                  }
                  setError(null);
                  setZipFile(file);
                  // Build a lightweight preview
                  (async () => {
                    try {
                      const buf = new Uint8Array(await file.arrayBuffer());
                      let entries: Record<string, Uint8Array> = {};
                      try {
                        entries = unzipSync(buf);
                      } catch {
                        // ignore preview errors; still allow upload
                        entries = {} as any;
                      }
                      // Find first image for thumbnail
                      const names = Object.keys(entries);
                      let imageName: string | null = null;
                      for (const n of names) {
                        const lower = n.toLowerCase();
                        if (lower.endsWith('.png') || lower.endsWith('.jpg') || lower.endsWith('.jpeg') || lower.endsWith('.webp') || lower.endsWith('.bmp')) {
                          imageName = n;
                          break;
                        }
                      }
                      let nextUrl: string | null = null;
                      if (imageName) {
                        try {
                          const bytes = entries[imageName];
                          const blob = new Blob([bytes]);
                          nextUrl = URL.createObjectURL(blob);
                        } catch {}
                      }

                      // Attempt to parse basic metadata by scanning JSON files and keys recursively
                      let nextSong = '';
                      let nextArtist = '';
                      let nextCharter = '';
                      const jsonDecoder = new TextDecoder();
                      const preferOrder = ['info.json','song.json','beatmap.json','metadata.json'];
                      const jsonNames = names
                        .filter(n => n.toLowerCase().endsWith('.json'))
                        .sort((a,b) => {
                          const al = a.toLowerCase();
                          const bl = b.toLowerCase();
                          const ai = preferOrder.findIndex(p => al.endsWith('/'+p) || al === p);
                          const bi = preferOrder.findIndex(p => bl.endsWith('/'+p) || bl === p);
                          const ar = ai === -1 ? 999 : ai;
                          const br = bi === -1 ? 999 : bi;
                          return ar - br;
                        });

                      const findStringByKeys = (obj: any, keys: string[]): string | null => {
                        if (!obj || typeof obj !== 'object') return null;
                        for (const k of Object.keys(obj)) {
                          const v = (obj as any)[k];
                          const kl = k.toLowerCase();
                          if (keys.includes(kl)) {
                            if (typeof v === 'string') return v;
                            if (Array.isArray(v)) {
                              const joined = v.filter(x => typeof x === 'string').join(', ');
                              if (joined) return joined;
                            }
                          }
                        }
                        // Recurse
                        for (const k of Object.keys(obj)) {
                          const v = (obj as any)[k];
                          if (v && typeof v === 'object') {
                            const found = findStringByKeys(v, keys);
                            if (found) return found;
                          }
                        }
                        return null;
                      };

                      for (const jn of jsonNames) {
                        try {
                          const text = jsonDecoder.decode(entries[jn]);
                          const data = JSON.parse(text);
                          if (!nextSong) {
                            nextSong = findStringByKeys(data, ['song','title','name']) || nextSong;
                          }
                          if (!nextArtist) {
                            nextArtist = findStringByKeys(data, ['artist','artists','author','creator','mapper','composer']) || nextArtist;
                          }
                          if (!nextCharter) {
                            nextCharter = findStringByKeys(data, ['charter','mapper','author','creator']) || nextCharter;
                          }
                          if (nextSong && nextArtist && nextCharter) break;
                        } catch {}
                      }

                      // Sanitize values to match server-side normalization
                      const safeSong = sanitizeText(nextSong || '', 200);
                      const safeArtist = sanitizeText(nextArtist || '', 200);
                      const safeCharter = sanitizeText(nextCharter || '', 200);

                      // Apply preview state
                      if (previewUrl) URL.revokeObjectURL(previewUrl);
                      setPreviewUrl(nextUrl);
                      const nowIso = new Date().toISOString();
                      const map: BeatMap = {
                        song: safeSong || 'Unknown Song',
                        artist: safeArtist || 'Unknown Artist',
                        charter: safeCharter || 'Unknown',
                        charter_uid: "00000000-0000-0000-0000-000000000000" as any,
                        difficulties: [],
                        description: '',
                        artist_list: '',
                        image: Boolean(nextUrl),
                        upvotes: 0,
                        upload_date: nowIso as any,
                        update_date: nowIso as any,
                        id: "00000000-0000-0000-0000-000000000000" as any,
                      };
                      setPreviewMap(map);

                      // Ask backend if this will be an update for the current user (metadata only)
                      setWillUpdate(false);
                      try {
                        const resp = await apiFetchAuth('/api/check-duplicate', {
                          method: 'POST',
                          headers: { 'Content-Type': 'application/json' },
                          body: JSON.stringify({
                            song: safeSong,
                            artist: safeArtist,
                            charter: safeCharter,
                          }),
                        });
                        if (resp.ok) {
                          const data = await resp.json() as { exists: boolean; id?: string };
                          setWillUpdate(Boolean(data.exists));
                        }
                      } catch {}
                    } catch {
                      if (previewUrl) URL.revokeObjectURL(previewUrl);
                      setPreviewUrl(null);
                      setPreviewMap(null);
                    }
                  })();
                }}
              />
              {/* Removed duplicate filename text under the input */}
            </label>

            {zipFile && previewMap && (
              <SongCard
                map={previewMap}
                className={"block w-full aspect-[32/9] border border-black overflow-hidden relative text-left bg-white rounded-lg shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]"}
                hideActions={true}
                clickable={false}
                imageOverrideUrl={previewUrl}
              />
            )}

            {submitting && (
              <div className="w-full mb-2">
                <div className="text-[10px] font-['Press_Start_2P'] mb-1">{phase || "Processing"}</div>
                <div className="w-full h-3 bg-gray-200 border border-black">
                  <div className="h-full bg-purple-600" style={{ width: `${progress}%` }} />
                </div>
              </div>
            )}

            <div className="flex items-center gap-2">
              <button
                type="submit"
                className="pixel-btn bg-black text-white px-3 py-2 text-xs font-['Press_Start_2P']"
                disabled={!canSubmit}
              >
                {submitting ? `Uploading... ${phase}` : (willUpdate ? 'Update Existing Upload' : 'Upload Map')}
              </button>
              {!submitting ? (
                <button
                  type="button"
                  className="pixel-btn bg-white px-3 py-2 text-xs font-['Press_Start_2P']"
                  onClick={() => navigate("/")}
                >
                  Cancel
                </button>
              ) : (
                <button
                  type="button"
                  className="pixel-btn bg-white px-3 py-2 text-xs font-['Press_Start_2P']"
                  onClick={handleCancel}
                >
                  Cancel Upload
                </button>
              )}
            </div>
          </div>
        </form>
      )}
    </main>
  );
}

// Cleanup any object URL on unmount or zip change via effect inside component

