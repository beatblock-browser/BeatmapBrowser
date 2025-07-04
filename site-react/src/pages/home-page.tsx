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
    const [shouldFadeOut, setShouldFadeOut] = useState(false);
    const [hideButtons, setHideButtons] = useState<string | null>(null);
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
                
                // Hide buttons in the animated clone more aggressively
                const allDivs = animatedCard.querySelectorAll('div');
                let buttonsFound = 0;
                allDivs.forEach(div => {
                    const classes = div.className || '';
                    if (classes.includes('bottom-4') && classes.includes('right-4')) {
                        console.log('Found button container, hiding it');
                        div.style.opacity = '0';
                        div.style.transform = 'scale(0.9)';
                        div.style.pointerEvents = 'none';
                        buttonsFound++;
                    }
                });
                console.log(`Found ${buttonsFound} button containers in clone`);
                
                // Add the animated card to the document
                document.body.appendChild(animatedCard);
                
                // Create a temporary element with exact same CSS as the banner to get precise dimensions
                const tempBanner = document.createElement('div');
                tempBanner.style.position = 'absolute';
                tempBanner.style.top = '0';
                tempBanner.style.left = '0';
                tempBanner.style.right = '0';
                tempBanner.style.height = 'calc(100vw * 9 / 32)';
                tempBanner.style.maxHeight = '320px';
                tempBanner.style.visibility = 'hidden';
                tempBanner.style.pointerEvents = 'none';
                document.body.appendChild(tempBanner);
                
                const bannerRect = tempBanner.getBoundingClientRect();
                const viewportWidth = bannerRect.width;
                const bannerHeight = bannerRect.height;
                
                // Clean up temp element
                document.body.removeChild(tempBanner);
                
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
                animatedCard.id = 'animated-banner-card'; // Add specific ID for tracking
                
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
            // Keep song page visible during reverse animation to allow fade out
            // setShowSongPage(false); // Removed - let the fade out handle this
            
            const originalRect = originalCardPosition.current[transitioningCard];
            
            // Find the existing animated card that's acting as the banner
            const existingAnimatedCard = document.getElementById('animated-banner-card') as HTMLButtonElement;
            
            if (existingAnimatedCard && originalRect) {
                const animatedText = existingAnimatedCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement;
                
                // Get current banner dimensions
                const currentRect = existingAnimatedCard.getBoundingClientRect();
                const viewportWidth = currentRect.width;
                const bannerHeight = currentRect.height;
                
                // Start animation
                requestAnimationFrame(() => {
                    existingAnimatedCard.style.transition = 'transform 400ms ease-out';
                    if (animatedText) {
                        animatedText.style.transition = 'transform 400ms ease-out';
                    }
                    
                    requestAnimationFrame(() => {
                        // Animate back to original position (no transform - element is already positioned at original location)
                        const scaleBackX = originalRect.width / viewportWidth;
                        const scaleBackY = originalRect.height / bannerHeight;
                        
                        existingAnimatedCard.style.transform = `translate(0px, 0px) scale(1, 1)`;
                        if (animatedText) {
                            animatedText.style.transform = `scale(1, 1)`;
                        }
                        
                        // Fallback cleanup in case fade out handler doesn't complete properly
                        setTimeout(() => {
                            // Clean up animated card if it still exists
                            const remainingCard = document.getElementById('animated-banner-card');
                            if (remainingCard && remainingCard.parentNode) {
                                remainingCard.parentNode.removeChild(remainingCard);
                            }
                        }, 450); // Slightly longer than animation to be safe
                    });
                });
            }
        }
    }, [isReverseAnimation, transitioningCard, results]);

    // Handle song page visibility based on selectedMap
    useEffect(() => {
        if (selectedMap && !showSongPage && cardTransformed && !isReverseAnimation) {
            // Card transition completes, show song page (keep animated card as banner)
            const timer = setTimeout(() => {
                setShowSongPage(true);
                // Don't remove animated clones - they stay as the banner
            }, 400);
            return () => clearTimeout(timer);
        } else if (!selectedMap && showSongPage && !isReverseAnimation) {
            // Hide song page and reset transition state (only if not doing reverse animation)
                            setShowSongPage(false);
                setTransitioningCard(null);
                setCardTransformed(false);
                setShouldFadeOut(false);
                setHideButtons(null);
        }
    }, [selectedMap, showSongPage, cardTransformed, isReverseAnimation]);

    // Handle window resize for animated card
    useEffect(() => {
        const handleResize = () => {
            const animatedCard = document.getElementById('animated-banner-card') as HTMLButtonElement;
            if (animatedCard && transitioningCard && cardTransformed && !isReverseAnimation) {
                // Recalculate banner dimensions
                const tempBanner = document.createElement('div');
                tempBanner.style.position = 'absolute';
                tempBanner.style.top = '0';
                tempBanner.style.left = '0';
                tempBanner.style.right = '0';
                tempBanner.style.height = 'calc(100vw * 9 / 32)';
                tempBanner.style.maxHeight = '320px';
                tempBanner.style.visibility = 'hidden';
                tempBanner.style.pointerEvents = 'none';
                document.body.appendChild(tempBanner);
                
                const bannerRect = tempBanner.getBoundingClientRect();
                const newViewportWidth = bannerRect.width;
                const newBannerHeight = bannerRect.height;
                
                document.body.removeChild(tempBanner);
                
                // Get original card dimensions
                const originalRect = originalCardPosition.current[transitioningCard];
                if (originalRect) {
                    // Update animated card to new banner size
                    animatedCard.style.width = `${newViewportWidth}px`;
                    animatedCard.style.height = `${newBannerHeight}px`;
                    
                    // Recalculate and apply transforms
                    const scaleX = newViewportWidth / originalRect.width;
                    const scaleY = newBannerHeight / originalRect.height;
                    
                    animatedCard.style.transform = `translate(${-originalRect.left}px, ${-originalRect.top}px) scale(${scaleX}, ${scaleY})`;
                    
                    // Update text scaling
                    const animatedText = animatedCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement;
                    if (animatedText) {
                        animatedText.style.transform = `scale(${1/scaleX}, ${1/scaleY})`;
                    }
                }
            }
        };

        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, [transitioningCard, cardTransformed, isReverseAnimation]);

    const handleCardClick = (map: BeatMap) => {
        if (transitioningCard || isReverseAnimation) return; // Prevent clicks during any animation
        
        // Hide buttons first
        setHideButtons(map.id);
        
        // Small delay to let button fade start before cloning the card
        setTimeout(() => {
            setTransitioningCard(map.id);
            setSelectedMap(map);
            setShouldFadeOut(false); // Reset fade out state
        }, 50); // Small delay for React to update DOM
    };

    const handleCloseSongPage = () => {
        console.log('Close song page clicked');
        
        if (selectedMap && transitioningCard) {
            // Start both fade out and reverse animation simultaneously
            setShouldFadeOut(true);
            setIsReverseAnimation(true);
            
            // Cleanup after animations finish (reverse animation takes 400ms)
            setTimeout(() => {
                console.log('Cleanup after animations complete');
                
                // First hide the song page to prevent "Song not found" flicker
                setShowSongPage(false);
                
                                 // Then clean up all other states
                 setTimeout(() => {
                     setShouldFadeOut(false);
                     setSelectedMap(null);
                     setTransitioningCard(null);
                     setIsReverseAnimation(false);
                     setCardTransformed(false);
                     
                     // Clean up any remaining animated cards
                     const animatedCard = document.getElementById('animated-banner-card');
                     if (animatedCard && animatedCard.parentNode) {
                         animatedCard.parentNode.removeChild(animatedCard);
                     }
                     
                     // Show buttons again quickly
                     setTimeout(() => {
                         setHideButtons(null);
                     }, 10); // Almost immediate
                 }, 50); // Small delay to ensure song page is hidden first
            }, 450); // After reverse animation completes (400ms) + small buffer
        } else {
            // Fallback for direct URL access
            setShouldFadeOut(true);
            
            // Cleanup for direct URL access after fade completes
            setTimeout(() => {
                // Hide song page first to prevent flicker
                setShowSongPage(false);
                
                                 // Then clean up states
                 setTimeout(() => {
                     setShouldFadeOut(false);
                     setSelectedMap(null);
                     setHideButtons(null); // Show buttons again
                 }, 50);
            }, 350); // After fade out completes (300ms) + buffer
        }
    };

    const handleFadeOutComplete = () => {
        // This handler isn't working reliably, so we rely on the timeout-based cleanup
        console.log('Fade out complete called but using timeout-based cleanup instead');
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
                                    <div className={`absolute bottom-4 right-4 flex items-center gap-2 ease-in-out ${
                                        hideButtons === map.id 
                                            ? 'opacity-0 scale-90 transition-all duration-300' 
                                            : 'opacity-100 scale-100 transition-all duration-150'
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
            {showSongPage && <SongPage skipEntranceAnimation={!!transitioningCard} onClose={handleFadeOutComplete} shouldFadeOut={shouldFadeOut} />}
            
            {/* Back Button - Rendered outside SongPage to avoid z-index inheritance */}
            {showSongPage && !shouldFadeOut && (
                <button
                    className={`fixed top-4 left-4 z-[60] text-white p-2 bg-black/20 rounded-full backdrop-blur-sm 
                               transform transition-all duration-300 hover:scale-110 active:scale-90 
                               opacity-100 scale-100`}
                    onClick={handleCloseSongPage}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                </button>
            )}
        </div>
    );
}