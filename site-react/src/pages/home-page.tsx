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
    const [isReverseAnimation, setIsReverseAnimation] = useState(false);
    const cardRefs = useRef<{ [key: string]: HTMLButtonElement | null }>({});
    const textRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
    const originalCardPosition = useRef<{ [key: string]: DOMRect }>({});

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
        if (transitioningCard && selectedMap && !isReverseAnimation) {
            const originalCardElement = cardRefs.current[transitioningCard];
            const selectedMapData = results.find(map => map.id === transitioningCard);
            
            if (originalCardElement && selectedMapData) {
                // Get the card's current position and store it for reverse animation
                const rect = originalCardElement.getBoundingClientRect();
                originalCardPosition.current[transitioningCard] = rect;
                
                // Create a clone of the card for animation
                const animatedCard = originalCardElement.cloneNode(true) as HTMLButtonElement;
                const animatedText = animatedCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement;
                
                // Remove any refs and event listeners from the clone
                animatedCard.removeAttribute('data-*');
                animatedCard.onclick = null;
                
                // Add the animated card to the document
                document.body.appendChild(animatedCard);
                
                const viewportWidth = window.innerWidth;
                const bannerHeight = Math.min(viewportWidth * 9 / 32, 320);
                
                // Position clone exactly at original card location
                animatedCard.style.position = 'fixed';
                animatedCard.style.zIndex = '50';
                animatedCard.style.top = `${rect.top}px`;
                animatedCard.style.left = `${rect.left}px`;
                animatedCard.style.width = `${rect.width}px`;
                animatedCard.style.height = `${rect.height}px`;
                animatedCard.style.transformOrigin = 'top left';
                animatedCard.style.transition = 'transform 400ms ease-out';
                animatedCard.style.visibility = 'visible'; // Override any invisible class
                animatedCard.style.opacity = '1'; // Ensure it's fully visible
                
                if (animatedText) {
                    animatedText.style.transformOrigin = 'left center';
                    animatedText.style.transition = 'transform 400ms ease-out';
                }
                
                // Trigger the transforms on next frame
                requestAnimationFrame(() => {
                    // Calculate translation and scaling for card
                    const translateX = -rect.left;
                    const translateY = -rect.top;
                    const scaleX = viewportWidth / rect.width;
                    const scaleY = bannerHeight / rect.height;
                    
                    // Apply translation and scaling to animated card
                    animatedCard.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scaleX}, ${scaleY})`;
                    
                    // Apply inverse scaling to text to keep it at original size
                    if (animatedText) {
                        animatedText.style.transform = `scale(${1/scaleX}, ${1/scaleY})`;
                    }
                    
                    setCardTransformed(true);
                });
            }
        }
    }, [transitioningCard, selectedMap, isReverseAnimation, results]);

    // Handle reverse animation when closing
    useEffect(() => {
        if (isReverseAnimation && transitioningCard) {
            // Hide song page immediately to show home page underneath
            setShowSongPage(false);
            
            const originalCardElement = cardRefs.current[transitioningCard];
            const originalRect = originalCardPosition.current[transitioningCard];
            const selectedMapData = results.find(map => map.id === transitioningCard);
            
            if (originalCardElement && originalRect && selectedMapData) {
                // Create a clone of the card for animation instead of modifying the original
                const animatedCard = originalCardElement.cloneNode(true) as HTMLButtonElement;
                const animatedText = animatedCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement;
                
                // Remove any refs and event listeners from the clone
                animatedCard.removeAttribute('data-*');
                animatedCard.onclick = null;
                
                // Add the animated card to the document
                document.body.appendChild(animatedCard);
                
                // Position animated card exactly over the banner
                const viewportWidth = window.innerWidth;
                const bannerHeight = Math.min(viewportWidth * 9 / 32, 320);
                
                animatedCard.style.position = 'fixed';
                animatedCard.style.zIndex = '60';
                animatedCard.style.top = '0px';
                animatedCard.style.left = '0px';
                animatedCard.style.width = `${viewportWidth}px`;
                animatedCard.style.height = `${bannerHeight}px`;
                animatedCard.style.transformOrigin = 'top left';
                animatedCard.style.transition = '';
                animatedCard.style.transform = '';
                
                if (animatedText) {
                    animatedText.style.transformOrigin = 'left center';
                    animatedText.style.transition = '';
                    animatedText.style.transform = '';
                }
                
                // Start animation
                requestAnimationFrame(() => {
                    animatedCard.style.transition = 'transform 400ms ease-out';
                    if (animatedText) {
                        animatedText.style.transition = 'transform 400ms ease-out';
                    }
                    
                    requestAnimationFrame(() => {
                        // Animate back to original position
                        const translateX = originalRect.left;
                        const translateY = originalRect.top;
                        const scaleBackX = originalRect.width / viewportWidth;
                        const scaleBackY = originalRect.height / bannerHeight;
                        
                        animatedCard.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scaleBackX}, ${scaleBackY})`;
                        if (animatedText) {
                            animatedText.style.transform = `scale(${1/scaleBackX}, ${1/scaleBackY})`;
                        }
                        
                        // After animation completes, clean up
                        setTimeout(() => {
                            // Remove the animated card
                            if (animatedCard.parentNode) {
                                animatedCard.parentNode.removeChild(animatedCard);
                            }
                            
                            setSelectedMap(null);
                            setTransitioningCard(null);
                            setIsReverseAnimation(false);
                            setCardTransformed(false);
                        }, 400);
                    });
                });
            }
        }
    }, [isReverseAnimation, transitioningCard, results]);

    // Handle song page visibility based on selectedMap
    useEffect(() => {
        if (selectedMap && !showSongPage && cardTransformed && !isReverseAnimation) {
            // Card transition completes, show song page (only if not reverse animating)
            const timer = setTimeout(() => {
                setShowSongPage(true);
                
                // Clean up any animated clones from forward animation
                const animatedClones = document.querySelectorAll('button[style*="position: fixed"][style*="z-index: 50"]');
                animatedClones.forEach(clone => {
                    if (clone.parentNode) {
                        clone.parentNode.removeChild(clone);
                    }
                });
            }, 400);
            return () => clearTimeout(timer);
        } else if (!selectedMap && showSongPage && !isReverseAnimation) {
            // Hide song page and reset transition state (only if not doing reverse animation)
            setShowSongPage(false);
            setTransitioningCard(null);
            setCardTransformed(false);
        }
    }, [selectedMap, showSongPage, cardTransformed, isReverseAnimation]);

    const handleCardClick = (map: BeatMap) => {
        if (transitioningCard || isReverseAnimation) return; // Prevent clicks during any animation
        
        setTransitioningCard(map.id);
        setSelectedMap(map);
    };

    const handleCloseSongPage = () => {
        if (selectedMap && transitioningCard) {
            // Start reverse animation
            setIsReverseAnimation(true);
        } else {
            // Fallback for direct URL access
            setSelectedMap(null);
        }
    };

    const getCardClassName = (mapId: string) => {
        const baseClass = "block w-full aspect-[32/9] border border-black cursor-pointer overflow-hidden relative text-left bg-white";
        const shadowClass = "shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]";
        
        // Hide the transitioning card during forward animation (clone handles the animation)
        if (transitioningCard === mapId && !isReverseAnimation) {
            return `${baseClass} transform-gpu invisible`;
        }
        
        // Hide other cards during forward animation only
        if (transitioningCard && transitioningCard !== mapId && !isReverseAnimation) {
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
                    } ${transitioningCard && !isReverseAnimation ? 'opacity-0' : ''}`}
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
            {showSongPage && <SongPage skipEntranceAnimation={!!transitioningCard} onClose={handleCloseSongPage} />}
        </div>
    );
}