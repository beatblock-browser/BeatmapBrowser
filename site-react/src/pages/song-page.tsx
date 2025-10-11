import React, { useState, useEffect } from "react";
import { createPortal } from "react-dom";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { BeatMap } from "@/schema";
import { useStore } from "@/lib/store";
import { apiFetch, apiFetchAuth, R2_PUBLIC_URL } from "@/lib/api";
import { sanitizeText } from "@/lib/sanitize";
// @ts-ignore
import default_image from './../public/beatblocks.jpg';

interface SongPageProps {
    skipEntranceAnimation?: boolean;
    onClose?: () => void;
    shouldFadeOut?: boolean;
}

export default function SongPage({ skipEntranceAnimation = false, onClose, shouldFadeOut = false }: SongPageProps) {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const location = useLocation();
    const { selectedMap, setSelectedMap, upvoteMap } = useStore();
    const [fetchedMap, setFetchedMap] = useState<BeatMap | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [hasAnimatedCard, setHasAnimatedCard] = useState(false);
    const [isBackVisible, setIsBackVisible] = useState(false);
    const [shouldRenderBack, setShouldRenderBack] = useState(false);
    const [jwt, setJwt] = useState<string | null>(null);
    const [showUpdatedBanner, setShowUpdatedBanner] = useState<boolean>(() => Boolean((location.state as any)?.updated));
    const [isDeleted, setIsDeleted] = useState<boolean>(false);
    const [isModerator, setIsModerator] = useState<boolean>(false);
    const [isAdmin, setIsAdmin] = useState<boolean>(false);

    // Determine which map to use and whether we're in overlay mode
    const isOverlayMode = !!selectedMap;
    const currentMap = selectedMap || fetchedMap;
    const safeSong = currentMap ? sanitizeText(currentMap.song, 200) : "";
    const safeArtist = currentMap ? sanitizeText(currentMap.artist, 200) : "";
    const safeCharter = currentMap ? sanitizeText(currentMap.charter, 200) : "";

    useEffect(() => {
        // Fade in the song page after a shorter delay
        const fadeInTimer = setTimeout(() => {
            setIsVisible(true);
        }, skipEntranceAnimation ? 100 : 50); // Much shorter delay
        
        return () => clearTimeout(fadeInTimer);
    }, [skipEntranceAnimation]);

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

    // Delay back button appearance explicitly via state to ensure consistent timing
    useEffect(() => {
        if (isVisible) {
            const t = setTimeout(() => {
                setShouldRenderBack(true);
                // ensure first paint at opacity-0, then animate to 1
                requestAnimationFrame(() => setIsBackVisible(true));
            }, 500);
            return () => clearTimeout(t);
        }
        setIsBackVisible(false);
        setShouldRenderBack(false);
    }, [isVisible]);

    // Monitor for animated cards
    useEffect(() => {
        const checkAnimatedCard = () => {
            const animatedCard = document.getElementById('animated-banner-card');
            setHasAnimatedCard(!!animatedCard);
        };
        
        // Check immediately
        checkAnimatedCard();
        
        // Set up an interval to check periodically
        const interval = setInterval(checkAnimatedCard, 100);
        
        return () => clearInterval(interval);
    }, []);

    // Handle fade out when requested
    useEffect(() => {
        if (shouldFadeOut && isVisible) {
            setIsVisible(false);
            // Notify parent after fade out completes
            const fadeOutTimer = setTimeout(() => {
                if (onClose) {
                    onClose();
                }
            }, 300); // Match the fade out duration
            
            return () => clearTimeout(fadeOutTimer);
        }
    }, [shouldFadeOut, isVisible, onClose]);

    useEffect(() => {
        // If we have selectedMap from store, don't fetch
        if (selectedMap) return;

        // If we have URL param but no selectedMap, fetch the data
        if (id && !selectedMap) {
            const fetchSong = async () => {
                setIsLoading(true);
                setError(null);
                
                try {
                    const response = await apiFetch(`/api/map/${id}`);

                    if (!response.ok) {
                        if (response.status === 404) {
                            // If opened as a full page (no overlay), route to /404
                            if (!selectedMap) {
                                navigate('/404', { replace: true });
                                return;
                            }
                            setError("Song not found");
                        } else {
                            throw new Error(`HTTP ${response.status}`);
                        }
                        return;
                    }
                    const data: BeatMap = await response.json();
                    setFetchedMap(data);
                } catch (e: any) {
                    setError(e.message || "Failed to load song. Please try again later.");
                } finally {
                    setIsLoading(false);
                }
            };

            fetchSong();
        }
    }, [id, selectedMap]);

    // Fetch moderation status for this map when signed in
    useEffect(() => {
        const doFetch = async () => {
            if (!jwt) { setIsDeleted(false); setIsModerator(false); setIsAdmin(false); return; }
            const map = selectedMap || fetchedMap;
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
    }, [jwt, selectedMap, fetchedMap]);

    const handleBack = () => {
        // Always fade out first
        setIsVisible(false);
        
        // Then handle the actual back action after fade out completes
        setTimeout(() => {
            // Prefer router history back when we were opened with a background location
            const hasBackground = (location.state as any)?.backgroundLocation;
            if (hasBackground) {
                // Let Home know to run reverse animation
                window.dispatchEvent(new Event('song:closing'));
                navigate(-1);
                return;
            }

            // Fallbacks
            if (isOverlayMode) {
                // Notify reverse animation as we close overlay
                window.dispatchEvent(new Event('song:closing'));
                if (onClose) {
                    onClose();
                } else {
                    setSelectedMap(null);
                }
            } else {
                navigate('/');
            }
        }, 300); // Wait for fade out to complete
    };

    const handleUpvote = async () => {
        if (!currentMap) return;
        await upvoteMap(currentMap.id);
        
        // Update the appropriate state
        if (isOverlayMode) {
            // Store will update selectedMap automatically
        } else {
            // Update local fetchedMap state
            setFetchedMap(prev => prev ? { ...prev, upvotes: prev.upvotes + 1 } : null);
        }
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

    if (isLoading) {
        return (
            <div className="fixed inset-0 z-10 bg-white flex items-center justify-center">
                <div className="flex flex-col items-center">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-black mb-4"></div>
                    <div className="text-center font-['Press_Start_2P']">Loading song...</div>
                </div>
            </div>
        );
    }

    if (error || !currentMap) {
        return (
            <div className="fixed inset-0 z-10 bg-white flex items-center justify-center">
                <div className="text-center">
                    <h1 className="text-2xl font-['Press_Start_2P'] mb-4">
                        {error || "Song not found"}
                    </h1>
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
        <div className={`fixed inset-0 z-[40] transition-opacity duration-300 ease-in-out ${isVisible ? 'opacity-100' : 'opacity-0'}`}>
            {(((location.state as any)?.backgroundLocation) || isOverlayMode) && shouldRenderBack && createPortal(
                (
                    <button
                        onClick={handleBack}
                        className={`fixed top-4 left-4 z-[70] text-white p-2 bg-black/30 rounded-full backdrop-blur-sm hover:scale-110 active:scale-90 transition-all duration-700 ease-out ${isBackVisible ? 'opacity-100 pointer-events-auto scale-100' : 'opacity-0 pointer-events-none scale-90'}`}
                        style={{ willChange: 'opacity, transform' }}
                        aria-label="Back"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>
                ),
                document.body
            )}

            <div className={`absolute inset-0 bg-white transition-all duration-300 ease-in-out ${isVisible ? 'opacity-100' : 'opacity-0'}`} />

            {showUpdatedBanner && (
                <div className="absolute top-4 left-1/2 -translate-x-1/2 z-[80] px-4 py-2 bg-green-600 text-white rounded shadow font-['Press_Start_2P'] text-xs">
                    Updated your existing upload
                </div>
            )}

            <div
                className={`absolute top-0 left-0 right-0 overflow-hidden transition-all duration-300 ease-in-out ${isVisible ? 'opacity-100' : 'opacity-0'} ${hasAnimatedCard ? 'hidden' : ''}`}
                style={{ height: 'calc(100vw * 9 / 32)', maxHeight: '320px' }}
            >
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

            <div
                className={`absolute top-0 left-0 right-0 bottom-0 overflow-y-auto transition-all duration-300 ease-in-out ${isVisible ? 'opacity-100' : 'opacity-0'}`}
                style={{ paddingTop: `min(calc(100vw * 9 / 32), 320px)` }}
            >
                <div className="max-w-6xl mx-auto p-4 pb-2">
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