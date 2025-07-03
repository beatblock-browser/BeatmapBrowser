import React, { useEffect, useState, useRef } from "react";
import { SearchRequest, SearchResult } from "@/schema/search";
import { BeatMap } from "@/schema";
import { useSearchCache } from "@/context/SearchCache";
import { useStore } from "@/lib/store";
import SongPage from "./song-page";
// @ts-ignore
import default_image from './../public/beatblocks.jpg';

export default function HomePage() {
    const { results, setResults, isLoading, setIsLoading } = useSearchCache();
    const { selectedMap, setSelectedMap } = useStore();
    const [error, setError] = useState<string | null>(null);
    const [titleVisible, setTitleVisible] = useState(false);
    const [transitioningCard, setTransitioningCard] = useState<string | null>(null);
    const [showSongPage, setShowSongPage] = useState(false);
    const [cardTransformed, setCardTransformed] = useState(false);
    const cardRefs = useRef<{ [key: string]: HTMLButtonElement | null }>({});
    const textRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});

    useEffect(() => {
        const fetchResults = async () => {
            setIsLoading(true);
            setError(null);
            try {
                const res = await fetch("/api/search", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ query: "" } as SearchRequest)
                });
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                const data: SearchResult = await res.json();
                setResults(data.results || []);
            } catch (e: any) {
                console.error("Failed to fetch results:", e);
                setError(e.message || "Failed to load beatmaps. Please try again later.");
                setResults([]);
            } finally {
                setIsLoading(false);
            }
        };
        fetchResults();
    }, [setResults, setIsLoading]);

    useEffect(() => {
        // Trigger title animation after component mounts
        setTitleVisible(true);
    }, []);

    // Handle body overflow when overlay is open
    useEffect(() => {
        if (selectedMap) {
            document.body.style.overflow = 'hidden';
        } else {
            document.body.style.overflow = 'auto';
        }

        // Cleanup function to restore scroll when component unmounts
        return () => {
            document.body.style.overflow = 'auto';
        };
    }, [selectedMap]);

    // Handle card transition animation
    useEffect(() => {
        if (transitioningCard && selectedMap) {
            const cardElement = cardRefs.current[transitioningCard];
            const textElement = textRefs.current[transitioningCard];
            if (cardElement && textElement) {
                // Get the card's current position
                const rect = cardElement.getBoundingClientRect();
                const viewportWidth = window.innerWidth;
                const bannerHeight = Math.min(viewportWidth * 9 / 32, 320);
                
                // Apply initial position to card
                cardElement.style.position = 'fixed';
                cardElement.style.zIndex = '50';
                cardElement.style.top = `${rect.top}px`;
                cardElement.style.left = `${rect.left}px`;
                cardElement.style.width = `${rect.width}px`;
                cardElement.style.height = `${rect.height}px`;
                cardElement.style.transformOrigin = 'top left';
                cardElement.style.transition = 'transform 400ms ease-out';
                
                // Set up text for inverse scaling with left center origin to maintain left alignment and vertical centering
                textElement.style.transformOrigin = 'left center';
                textElement.style.transition = 'transform 400ms ease-out';
                
                // Trigger the transforms on next frame
                requestAnimationFrame(() => {
                    // Calculate translation and scaling for card
                    const translateX = -rect.left;
                    const translateY = -rect.top;
                    const scaleX = viewportWidth / rect.width;
                    const scaleY = bannerHeight / rect.height;
                    
                    // Apply translation and scaling to card (this scales everything including image)
                    cardElement.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scaleX}, ${scaleY})`;
                    
                    // Apply inverse scaling to text to keep it at original size
                    // Now that card and banner both use p-6, positioning should align perfectly
                    textElement.style.transform = `scale(${1/scaleX}, ${1/scaleY})`;
                    
                    setCardTransformed(true);
                });
            }
        }
    }, [transitioningCard, selectedMap]);

    // Handle song page visibility based on selectedMap
    useEffect(() => {
        if (selectedMap && !showSongPage && cardTransformed) {
            // Card transition completes, show song page
            const timer = setTimeout(() => {
                setShowSongPage(true);
                // Reset all card and text styles
                Object.values(cardRefs.current).forEach(card => {
                    if (card) {
                        card.style.position = '';
                        card.style.zIndex = '';
                        card.style.top = '';
                        card.style.left = '';
                        card.style.width = '';
                        card.style.height = '';
                        card.style.transform = '';
                        card.style.transformOrigin = '';
                        card.style.transition = '';
                    }
                });
                Object.values(textRefs.current).forEach(text => {
                    if (text) {
                        text.style.transform = '';
                        text.style.transformOrigin = '';
                        text.style.transition = '';
                    }
                });
            }, 400);
            return () => clearTimeout(timer);
        } else if (!selectedMap && showSongPage) {
            // Hide song page and reset transition state
            setShowSongPage(false);
            setTransitioningCard(null);
            setCardTransformed(false);
            
            // Reset all card and text styles
            Object.values(cardRefs.current).forEach(card => {
                if (card) {
                    card.style.position = '';
                    card.style.zIndex = '';
                    card.style.top = '';
                    card.style.left = '';
                    card.style.width = '';
                    card.style.height = '';
                    card.style.transform = '';
                    card.style.transformOrigin = '';
                    card.style.transition = '';
                }
            });
            Object.values(textRefs.current).forEach(text => {
                if (text) {
                    text.style.transform = '';
                    text.style.transformOrigin = '';
                    text.style.transition = '';
                }
            });
        }
    }, [selectedMap, showSongPage, cardTransformed]);

    const handleCardClick = (map: BeatMap) => {
        if (transitioningCard) return; // Prevent multiple clicks during transition
        
        setTransitioningCard(map.id);
        setSelectedMap(map);
    };

    const getCardClassName = (mapId: string) => {
        const baseClass = "block w-full aspect-[32/9] border border-black cursor-pointer overflow-hidden relative text-left bg-white";
        const shadowClass = "shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]";
        
        if (transitioningCard === mapId) {
            return `${baseClass} transform-gpu`;
        }
        
        if (transitioningCard && transitioningCard !== mapId) {
            return `${baseClass} ${shadowClass} opacity-0 transition-opacity duration-200`;
        }
        
        return `${baseClass} ${shadowClass} transition-all duration-100 hover:translate-x-1 hover:translate-y-1 hover:shadow-none hover:scale-[1.02] active:scale-[0.98]`;
    };

    return (
        <div className="relative min-h-screen">
            {/* Search Grid */}
            <div className="max-w-6xl mx-auto p-4">
                <h1
                    className={`text-2xl mb-6 font-['Press_Start_2P'] text-center transform transition-all duration-300 ${
                        titleVisible ? 'translate-y-0 opacity-100' : '-translate-y-5 opacity-100'
                    } ${transitioningCard ? 'opacity-0' : ''}`}
                >
                    Beatmap Browser
                </h1>
                {isLoading && (
                    <div className="flex flex-col items-center justify-center py-8">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-black mb-4"></div>
                        <div className="text-center font-['Press_Start_2P']">Loading beatmaps...</div>
                    </div>
                )}
                {error && (
                    <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4">
                        <strong className="font-['Press_Start_2P'] block mb-2">Error:</strong>
                        <p className="font-['Press_Start_2P'] text-sm">{error}</p>
                    </div>
                )}
                {!isLoading && !error && (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {results.length === 0 && (
                            <div className="col-span-full text-center font-['Press_Start_2P'] py-8">
                                No beatmaps found. Try refreshing the page.
                            </div>
                        )}
                        {results.map((map) => (
                            <button
                                key={map.id}
                                ref={(el) => { cardRefs.current[map.id] = el; }}
                                onClick={() => handleCardClick(map)}
                                className={getCardClassName(map.id)}
                            >
                                <img
                                    src={map.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : default_image}
                                    alt={map.song}
                                    className="absolute inset-0 w-full h-full object-cover"
                                />

                                <div className="absolute inset-0 bg-black/60" />

                                <div 
                                    ref={(el) => { textRefs.current[map.id] = el; }}
                                    className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-6"
                                >
                                    <div className="flex-1 min-w-0">
                                        <h2 className="text-xl mb-1 line-clamp-1">{map.song}</h2>
                                        <p className="text-sm">by {map.artist}</p>
                                        <p className="text-sm">Charter: {map.charter}</p>
                                    </div>
                                    <div className={`absolute bottom-4 right-4 flex items-center gap-2 transition-opacity duration-200 ${
                                        transitioningCard === map.id ? 'opacity-0' : 'opacity-100'
                                    }`}>
                                        <div className="group relative">
                                            <a
                                                href={`https://beatmap-browser.s3.amazonaws.com/${map.id}.zip`}
                                                className="inline-flex items-center justify-center p-2 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors"
                                                onClick={(e) => e.stopPropagation()}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                                </svg>
                                            </a>
                                            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1 bg-black/90 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 whitespace-nowrap font-['Press_Start_2P'] pointer-events-none">
                                                Download Map
                                                <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 rotate-45"></div>
                                            </div>
                                        </div>
                                        <div className="group relative">
                                            <button
                                                className="inline-flex items-center justify-center p-2 bg-green-500 text-white rounded-full hover:bg-green-600 transition-colors"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    // TODO: Implement one-click install
                                                }}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                                </svg>
                                            </button>
                                            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1 bg-black/90 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 whitespace-nowrap font-['Press_Start_2P'] pointer-events-none">
                                                One-Click Install
                                                <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 rotate-45"></div>
                                            </div>
                                        </div>
                                        <div className="bg-black/40 backdrop-blur-sm px-3 py-1 rounded-full">
                                            <span className="text-sm">↑ {map.upvotes}</span>
                                        </div>
                                    </div>
                                </div>
                            </button>
                        ))}
                    </div>
                )}
            </div>

            {/* Song Page Overlay */}
            {showSongPage && <SongPage skipEntranceAnimation={!!transitioningCard} />}
        </div>
    );
}