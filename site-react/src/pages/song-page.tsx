import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { BeatMap } from "@/schema";
import { useStore } from "@/lib/store";
// @ts-ignore
import default_image from './../public/beatblocks.jpg';

interface SongPageProps {
    skipEntranceAnimation?: boolean;
}

export default function SongPage({ skipEntranceAnimation = false }: SongPageProps) {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const { selectedMap, setSelectedMap, upvoteMap, unvoteMap } = useStore();
    const [fetchedMap, setFetchedMap] = useState<BeatMap | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isVisible, setIsVisible] = useState(skipEntranceAnimation);
    const [showBackButton, setShowBackButton] = useState(!skipEntranceAnimation);

    // Determine which map to use and whether we're in overlay mode
    const isOverlayMode = !!selectedMap;
    const currentMap = selectedMap || fetchedMap;

    useEffect(() => {
        // Trigger animation after component mounts, unless skipping entrance animation
        if (!skipEntranceAnimation) {
            setIsVisible(true);
        } else {
            // When animating from card, hide back button initially
            setShowBackButton(false);
            setIsVisible(true);
            
            // Show back button after a brief delay for the card animation to complete
            const showTimer = setTimeout(() => {
                setShowBackButton(true);
            }, 600);
            
            return () => clearTimeout(showTimer);
        }
    }, [skipEntranceAnimation]);

    useEffect(() => {
        // If we have selectedMap from store, don't fetch
        if (selectedMap) return;

        // If we have URL param but no selectedMap, fetch the data
        if (id && !selectedMap) {
            const fetchSong = async () => {
                setIsLoading(true);
                setError(null);
                
                try {
                    const response = await fetch(`/api/map/${id}`);
                    if (!response.ok) {
                        if (response.status === 404) {
                            setError("Song not found");
                        } else {
                            throw new Error(`HTTP ${response.status}`);
                        }
                        return;
                    }
                    const data: BeatMap = await response.json();
                    setFetchedMap(data);
                } catch (e: any) {
                    console.error("Failed to fetch song:", e);
                    setError(e.message || "Failed to load song. Please try again later.");
                } finally {
                    setIsLoading(false);
                }
            };

            fetchSong();
        }
    }, [id, selectedMap]);

    const handleBack = () => {
        setIsVisible(false);
        setShowBackButton(false);
        setTimeout(() => {
            if (isOverlayMode) {
                // In overlay mode, close the overlay
                setSelectedMap(null);
            } else {
                // In URL mode, navigate back to home
                navigate('/');
            }
        }, skipEntranceAnimation ? 0 : 400);
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
        <div
            className={`fixed inset-0 z-40 transition-opacity duration-200 ${isVisible ? 'opacity-100' : 'opacity-0'}`}
        >
            {/* Background Panel */}
            <div
                className={`absolute inset-0 bg-white transform transition-transform ${
                    skipEntranceAnimation ? 'duration-0' : 'duration-400'
                } ease-out ${
                    isVisible ? 'translate-y-0' : '-translate-y-full'
                }`}
            />

            {/* Banner - Fixed at top */}
            <div
                className={`absolute top-0 left-0 right-0 overflow-hidden transition-all duration-150 ease-out ${
                    isVisible ? 'opacity-100' : 'opacity-0'
                }`}
                style={{
                    height: 'calc(100vw * 9 / 32)',
                    maxHeight: '320px'
                }}
            >
                <div className="absolute inset-0">
                    <img
                        src={currentMap.image ? `https://beatmap-browser.s3.amazonaws.com/${currentMap.id}.png` : default_image}
                        alt={currentMap.song}
                        className="w-full h-full object-cover"
                    />
                </div>

                <div className="absolute inset-0 bg-black/60" />

                <div className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-6">
                    <div className="flex-1 min-w-0">
                        <h1 className="text-xl mb-1 line-clamp-1">{currentMap.song}</h1>
                        <p className="text-sm">by {currentMap.artist}</p>
                        <p className="text-sm">Charter: {currentMap.charter}</p>
                    </div>
                </div>
            </div>

            {/* Back Button - Positioned after banner to appear on top */}
            <button
                className={`absolute top-4 left-4 z-50 text-white p-2 bg-black/20 rounded-full backdrop-blur-sm 
                           transform transition-all duration-300 hover:scale-110 active:scale-90 
                           ${showBackButton ? 'opacity-100 scale-100' : 'opacity-0 scale-75'}`}
                onClick={handleBack}
            >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                </svg>
            </button>

            {/* Main Content - Scrollable below banner */}
            <div
                className={`absolute top-0 left-0 right-0 bottom-0 overflow-y-auto pt-[320px] 
                           transform transition-all duration-300 ease-out ${
                               isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-12'
                           }`}
                style={{ transitionDelay: isVisible && !skipEntranceAnimation ? '300ms' : '0ms' }}
            >
                <div className="max-w-6xl mx-auto p-4 pb-2">
                    <div className="flex gap-6">
                        {/* Left Sidebar */}
                        <div className="w-64 flex-shrink-0">
                            <div className="border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                                <h2 className="text-xl font-['Press_Start_2P'] mb-4">Details</h2>
                                <div className="space-y-2">
                                    <button 
                                        onClick={handleUpvote}
                                        className="w-full text-left font-['Press_Start_2P'] text-sm hover:bg-gray-100 p-2 rounded transition-colors"
                                    >
                                        <span className="text-gray-600">Upvotes:</span> {currentMap.upvotes}
                                    </button>
                                    {currentMap.difficulties && (
                                        <p className="font-['Press_Start_2P'] text-sm">
                                            <span className="text-gray-600">Difficulties:</span> {currentMap.difficulties.join(", ")}
                                        </p>
                                    )}
                                </div>
                            </div>

                            <div className="mt-4 border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                                <h2 className="text-xl font-['Press_Start_2P'] mb-4">Download</h2>
                                <div className="space-y-4">
                                    <a
                                        href={`https://beatmap-browser.s3.amazonaws.com/${currentMap.id}.zip`}
                                        className="block w-full text-center py-2 px-4 bg-blue-500 text-white font-['Press_Start_2P'] text-sm hover:bg-blue-600 transition-colors"
                                    >
                                        Download Map
                                    </a>
                                    <button
                                        className="w-full py-2 px-4 bg-green-500 text-white font-['Press_Start_2P'] text-sm hover:bg-green-600 transition-colors"
                                    >
                                        One-Click Install
                                    </button>
                                </div>
                            </div>
                        </div>

                        {/* Main Content - Comments */}
                        <div className="flex-1">
                            <div className="border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
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
    );
}