import React, { useState, useEffect } from "react";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { BeatMap } from "@/schema";
import { useStore } from "@/lib/store";
import { apiFetch, apiFetchAuth, R2_PUBLIC_URL } from "@/lib/api";
import { sanitizeText } from "@/lib/sanitize";
import { getCachedMap, setCachedMap } from "@/lib/mapCache";
// @ts-ignore
import default_image from './../public/beatblocks.jpg';

export default function SongPage() {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const location = useLocation();
    const { upvoteMap } = useStore();
    const [fetchedMap, setFetchedMap] = useState<BeatMap | null>(() => id ? getCachedMap(id) : null);
    const [jwt, setJwt] = useState<string | null>(null);
    const [showUpdatedBanner, setShowUpdatedBanner] = useState<boolean>(() => Boolean((location.state as any)?.updated));
    const [isDeleted, setIsDeleted] = useState<boolean>(false);
    const [isModerator, setIsModerator] = useState<boolean>(false);
    const [isAdmin, setIsAdmin] = useState<boolean>(false);

    const currentMap = fetchedMap;
    const safeSong = currentMap ? sanitizeText(currentMap.song, 200) : "";
    const safeArtist = currentMap ? sanitizeText(currentMap.artist, 200) : "";
    const safeCharter = currentMap ? sanitizeText(currentMap.charter, 200) : "";

    useEffect(() => {
        if ((location.state as any)?.updated) {
            setShowUpdatedBanner(true);
            const t = setTimeout(() => setShowUpdatedBanner(false), 4000);
            return () => clearTimeout(t);
        }
    }, [location.state]);

    useEffect(() => {
        setJwt(localStorage.getItem('jwt'));
        const onStorage = () => setJwt(localStorage.getItem('jwt'));
        window.addEventListener('storage', onStorage);
        return () => window.removeEventListener('storage', onStorage);
    }, []);


    useEffect(() => {
        if (id) {
            const fetchSong = async () => {
                try {
                    const response = await apiFetch(`/api/map/${id}`);
                    if (!response.ok) {
                        if (response.status === 404) {
                            navigate('/404', { replace: true });
                            return;
                        }
                        return;
                    }
                    const data: BeatMap = await response.json();
                    setCachedMap(id, data);
                    setFetchedMap(data);
                } catch {}
            };
            fetchSong();
        }
    }, [id, navigate]);

    useEffect(() => {
        const doFetch = async () => {
            if (!jwt) { setIsDeleted(false); setIsModerator(false); setIsAdmin(false); return; }
            const map = fetchedMap;
            if (!map) return;
            try {
                const res = await apiFetchAuth(`/api/moderation/status`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ map_id: map.id }),
                });
                if (res.ok) {
                    const data = await res.json() as { is_deleted: boolean; is_moderator: boolean; is_admin: boolean };
                    setIsDeleted(Boolean(data.is_deleted));
                    setIsModerator(Boolean(data.is_moderator));
                    setIsAdmin(Boolean(data.is_admin));
                }
            } catch {}
        };
        doFetch();
    }, [jwt, fetchedMap]);

    const handleBack = () => {
        navigate('/');
    };

    const handleUpvote = async () => {
        if (!currentMap) return;
        await upvoteMap(currentMap.id);
        setFetchedMap(prev => prev ? { ...prev, upvotes: prev.upvotes + 1 } : null);
    };

    const handleDelete = async () => {
        if (!currentMap) return;
        const ok = window.confirm("Delete this map? This hides it from users until an admin restores it.");
        if (!ok) return;
        try {
            const resp = await apiFetchAuth(`/api/delete`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ map_id: currentMap.id }),
            });
            if (!resp.ok) {
                const msg = await resp.text();
                alert(msg || `Failed to delete (HTTP ${resp.status})`);
                return;
            }
            // Reflect deleted state in UI
            setIsDeleted(true);
            try {
                // Notify home page to refresh search and optimistically hide this map
                const deletedId = currentMap.id;
                window.dispatchEvent(new CustomEvent('map:deleted', { detail: { id: deletedId } }));
            } catch {}
            // Navigate back to the song list (handles overlay/full page with fade-out)
            handleBack();
        } catch {
            alert('Failed to delete.');
        }
    };

    const handleRestore = async () => {
        if (!currentMap) return;
        const ok = window.confirm("Restore this map for all users?");
        if (!ok) return;
        try {
            const resp = await apiFetchAuth(`/api/moderation/restore`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ map_id: currentMap.id }),
            });
            if (!resp.ok) {
                const msg = await resp.text();
                alert(msg || `Failed to restore (HTTP ${resp.status})`);
                return;
            }
            setIsDeleted(false);
        } catch {
            alert('Failed to restore.');
        }
    };

    if (!currentMap) {
        return (
            <div className="min-h-[calc(100vh-56px)] bg-white flex items-center justify-center">
                <div className="text-center">
                    <h1 className="text-2xl font-['Press_Start_2P'] mb-4">Song not found</h1>
                    <button
                        onClick={handleBack}
                        className="px-4 py-2 bg-black text-white font-['Press_Start_2P'] text-sm hover:bg-gray-800 transition-colors"
                    >
                        Back to Home
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-[calc(100vh-56px)] bg-white">
            {showUpdatedBanner && (
                <div className="fixed top-20 left-1/2 -translate-x-1/2 z-50 px-4 py-2 bg-green-600 text-white rounded shadow font-['Press_Start_2P'] text-xs">
                    Updated your existing upload
                </div>
            )}

            <div className="max-w-6xl mx-auto p-4">
                <div className="relative overflow-hidden rounded-lg border border-black mb-4" style={{ height: '320px' }}>
                    <button
                        onClick={handleBack}
                        className="absolute top-4 left-4 z-50 text-white p-2 bg-black/30 rounded-full backdrop-blur-sm hover:scale-110 active:scale-90 transition-all"
                        aria-label="Back to search"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>
                    <div className="absolute inset-0">
                        <img src={currentMap.image ? `${R2_PUBLIC_URL}/thumbs/${currentMap.id}.png` : default_image} alt={safeSong} className={`w-full h-full object-cover ${isDeleted ? 'grayscale-[80%] opacity-80' : ''}`} />
                    </div>
                    <div className="absolute inset-0 card-overlay-gradient" />
                    <div className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-6">
                        <div className="flex-1 min-w-0">
                            <h1 className="text-xl mb-1">{safeSong}</h1>
                            <p className="text-sm">by {safeArtist}</p>
                            <p className="text-sm">Charter: {safeCharter}</p>
                            {isDeleted && (isModerator || isAdmin) && (
                                <p className="mt-2 inline-block px-2 py-1 text-[10px] bg-yellow-600 text-white">Deleted</p>
                            )}
                        </div>
                    </div>
                </div>

                <div className="pb-2">
                    <div className="pixel-panel rounded-md bg-white p-4">
                        <div className="flex gap-6">
                            <div className="w-64 flex-shrink-0">
                                <div className="pixel-panel p-4 bg-white">
                                    <h2 className="text-xl font-['Press_Start_2P'] mb-4">Details</h2>
                                    <div className="space-y-2">
                                        {!jwt ? (
                                            <div className="w-full border border-black px-3 py-2 font-['Press_Start_2P'] text-sm bg-white">
                                                <span className="text-gray-700">↑ Upvotes:</span> {currentMap.upvotes}
                                            </div>
                                        ) : (
                                            <button onClick={handleUpvote} className="w-full pixel-btn bg-white hover:bg-gray-100 px-3 py-2 text-left font-['Press_Start_2P'] text-sm rounded-sm">
                                                <span className="text-gray-700">↑ Upvotes:</span> {currentMap.upvotes}
                                            </button>
                                        )}
                                        {Array.isArray((currentMap as any).difficulties) && (
                                            <p className="font-['Press_Start_2P'] text-sm">
                                                <span className="text-gray-600">Difficulties:</span> {((currentMap as any).difficulties || []).map((d: any) => sanitizeText(d.display, 64)).join(', ')}
                                            </p>
                                        )}
                                    </div>
                                </div>

                                <div className="mt-4 pixel-panel p-4 bg-white">
                                    <h2 className="text-xl font-['Press_Start_2P'] mb-4">Download</h2>
                                    <div className="space-y-4">
                                        <a href={`${R2_PUBLIC_URL}/maps/${currentMap.id}.zip`} className="block w-full text-center py-2 px-4 pixel-btn bg-blue-500 text-white font-['Press_Start_2P'] text-sm rounded-sm hover:bg-blue-600">
                                            Download Map
                                        </a>
                                        {jwt && (isModerator || isAdmin) && !isDeleted && (
                                            <button className="w-full py-2 px-4 pixel-btn bg-green-500 text-white font-['Press_Start_2P'] text-sm rounded-sm hover:bg-green-600">
                                                One-Click Install
                                            </button>
                                        )}
                                        {jwt && (isModerator || isAdmin) && !isDeleted && (
                                            <button
                                                onClick={handleDelete}
                                                className="w-full py-2 px-4 pixel-btn bg-red-600 text-white font-['Press_Start_2P'] text-sm rounded-sm hover:bg-red-700"
                                            >
                                                Delete Map
                                            </button>
                                        )}
                                        {jwt && isAdmin && isDeleted && (
                                            <button
                                                onClick={handleRestore}
                                                className="w-full py-2 px-4 pixel-btn bg-yellow-600 text-white font-['Press_Start_2P'] text-sm rounded-sm hover:bg-yellow-700"
                                            >
                                                Restore Map
                                            </button>
                                        )}
                                    </div>
                                </div>
                            </div>

                            <div className="flex-1">
                                <div className="pixel-panel p-4 bg-white">
                                    <h2 className="text-xl font-['Press_Start_2P'] mb-4">Comments</h2>
                                    <div className="space-y-4">
                                        <div className="border border-gray-200 p-4 rounded">
                                            <p className="text-gray-500 font-['Press_Start_2P'] text-sm">No comments yet. Be the first to comment!</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}