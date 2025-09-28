import React, { useEffect, useState, useRef, useCallback, useMemo } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { BeatMap } from "@/schema";
import { useSearchCache } from "@/context/SearchCache";
import { useStore } from "@/lib/store";
import { apiFetch } from "@/lib/api";

// Overlay song page is now routed via App.tsx using background location
// @ts-ignore
import default_image from './../public/beatblocks.jpg';

export default function HomePage() {
    const navigate = useNavigate();
    const location = useLocation();
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
    // Search and filter state
    const [query, setQuery] = useState("");
    const [debouncedQuery, setDebouncedQuery] = useState("");
    const [minUpvotes, setMinUpvotes] = useState<number | "">("");
    const [selectedDifficulties, setSelectedDifficulties] = useState<Set<string>>(new Set());
    const [sortBy, setSortBy] = useState<"relevance" | "upvotes" | "newest">("relevance");
    const [page, setPage] = useState(0);
    const [hasMore, setHasMore] = useState(true);
    const [loadingMore, setLoadingMore] = useState(false);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [availableDifficulties, setAvailableDifficulties] = useState<string[]>([]);
    const [totalCount, setTotalCount] = useState<number>(0);
    const cardRefs = useRef<{ [key: string]: HTMLButtonElement | null }>({});
    const textRefs = useRef<{ [key: string]: HTMLDivElement | null }>({});
    const originalCardPosition = useRef<{ [key: string]: DOMRect }>({});
    const sentinelRef = useRef<HTMLDivElement | null>(null);
    const requestIdRef = useRef(0);
    // Guards for pagination to avoid duplicate requests
    const pagingRef = useRef(false);
    const lastLoadAtRef = useRef(0);
    const firstPageLoadedRef = useRef(false);
    const hasLoadedOnceRef = useRef(false);
    const currentFilterRef = useRef<string>("");

    // Fetch a page from the backend worker with filters applied
    const fetchPage = useCallback(async (pageToLoad: number, reset: boolean = false) => {
        const reqId = ++requestIdRef.current;
        const isInitial = reset && !hasLoadedOnceRef.current;
        // Build a stable filter snapshot key for this request
        const filterKey = JSON.stringify({
            q: debouncedQuery,
            min: minUpvotes === "" ? null : Number(minUpvotes),
            diffs: Array.from(selectedDifficulties).sort(),
            sort: sortBy,
        });
        if (reset) {
            // Block IO-triggered load-more for the new filter until page 0 completes
            firstPageLoadedRef.current = false;
            // Update the current filter immediately on reset
            currentFilterRef.current = filterKey;
            // Clear old results synchronously so the UI doesn't show mismatched entries
            setResults([]);
            setTotalCount(0);
            setHasMore(true);
            // Reset paging lock because we're starting fresh
            pagingRef.current = false;
        }
        setIsLoading(isInitial);
        setIsRefreshing(reset && !isInitial);
        setLoadingMore(!reset);
        setError(null);
        try {
            const body = {
                query: debouncedQuery,
                min_upvotes: minUpvotes === "" ? undefined : Number(minUpvotes),
                difficulties: Array.from(selectedDifficulties),
                sort: sortBy,
                page: pageToLoad,
                page_size: 20,
            };
            const res = await apiFetch("/api/search", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(body),
            });
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            const data = await res.json() as { results: BeatMap[]; has_more: boolean; total_count?: number };
            // Drop responses that are stale by id or mismatch current filters
            if (reqId !== requestIdRef.current || filterKey !== currentFilterRef.current) {
                // stale response; ignore
                return;
            }
            setHasMore(Boolean(data.has_more));
            if (typeof data.total_count === 'number') setTotalCount(data.total_count);
            let cachedResults: BeatMap[] = [];
            if (reset) {
                const next = (data.results || []);
                setResults(next);
                cachedResults = next;
            } else {
                setResults(prev => {
                    const next = [...prev, ...(data.results || [])];
                    cachedResults = next;
                    return next;
                });
            }
            try {
                localStorage.setItem(`search:${filterKey}`,
                    JSON.stringify({
                        results: cachedResults,
                        hasMore: Boolean(data.has_more),
                        totalCount: typeof data.total_count === 'number' ? data.total_count : totalCount,
                        savedAt: Date.now(),
                    })
                );
            } catch {}
            if (pageToLoad === 0) {
                firstPageLoadedRef.current = true;
                hasLoadedOnceRef.current = true;
            }
        } catch (e: any) {
            console.error("Failed to fetch results:", e);
            setError(e.message || "Failed to load beatmaps. Please try again later.");
            if (reset) setResults([]);
        } finally {
            setIsLoading(false);
            setIsRefreshing(false);
            setLoadingMore(false);
        }
    }, [debouncedQuery, minUpvotes, selectedDifficulties, sortBy, setResults, setIsLoading]);

    // Initial load
    useEffect(() => {
        // Read filters from URL on first mount
        const params = new URLSearchParams(location.search);
        const q = params.get('q') || '';
        const minParam = params.get('min');
        const diffsParam = params.get('diffs');
        const sortParam = params.get('sort') as any;

        if (q) setQuery(q);
        if (minParam !== null) setMinUpvotes(minParam === '' ? '' : Math.max(0, Number(minParam)));
        if (diffsParam) setSelectedDifficulties(new Set(diffsParam.split(',').filter(Boolean)));
        if (sortParam === 'relevance' || sortParam === 'upvotes' || sortParam === 'newest') setSortBy(sortParam);

        if (results.length === 0) {
            setPage(0);
            // Try hydrate from cache before fetching
            const filterKey = JSON.stringify({
                q: q || '',
                min: minParam === null || minParam === '' ? null : Number(minParam),
                diffs: diffsParam ? diffsParam.split(',').filter(Boolean).sort() : [],
                sort: sortParam || sortBy,
            });
            const cacheRaw = localStorage.getItem(`search:${filterKey}`);
            if (cacheRaw) {
                try {
                    const cache = JSON.parse(cacheRaw);
                    const expiryMs = 10 * 60 * 1000; // 10 minutes
                    if (cache.savedAt && (Date.now() - cache.savedAt) < expiryMs) {
                        currentFilterRef.current = filterKey;
                        setResults(cache.results || []);
                        setTotalCount(cache.totalCount || 0);
                        setHasMore(Boolean(cache.hasMore));
                        firstPageLoadedRef.current = true;
                        hasLoadedOnceRef.current = true;
                        setIsLoading(false);
                        setIsRefreshing(false);
                        setLoadingMore(false);
                        return;
                    } else {
                        // stale cache
                        localStorage.removeItem(`search:${filterKey}`);
                    }
                } catch {}
            }
            fetchPage(0, true);
        }
    }, []);

    // Debounce user input for smoother filtering
    useEffect(() => {
        const t = setTimeout(() => setDebouncedQuery(query.trim()), 150);
        return () => clearTimeout(t);
    }, [query]);

    // Refetch when filters change, but hydrate from cache if available; also sync URL
    useEffect(() => {
        // Sync URL params
        const params = new URLSearchParams();
        if (debouncedQuery) params.set('q', debouncedQuery);
        if (minUpvotes !== '') params.set('min', String(minUpvotes));
        const diffsArr = Array.from(selectedDifficulties);
        if (diffsArr.length) params.set('diffs', diffsArr.sort().join(','));
        if (sortBy) params.set('sort', sortBy);
        const search = params.toString();
        const nextUrl = `${location.pathname}${search ? `?${search}` : ''}`;
        if (nextUrl !== `${location.pathname}${location.search}`) {
            navigate(nextUrl, { replace: true });
        }

        // Hydrate from cache or fetch
        setPage(0);
        const filterKey = JSON.stringify({
            q: debouncedQuery,
            min: minUpvotes === '' ? null : Number(minUpvotes),
            diffs: Array.from(selectedDifficulties).sort(),
            sort: sortBy,
        });
        currentFilterRef.current = filterKey;
        const cacheRaw = localStorage.getItem(`search:${filterKey}`);
        if (results.length === 0 && cacheRaw) {
            try {
                const cache = JSON.parse(cacheRaw);
                const expiryMs = 10 * 60 * 1000; // 10 minutes
                if (cache.savedAt && (Date.now() - cache.savedAt) < expiryMs) {
                    setResults(cache.results || []);
                    setTotalCount(cache.totalCount || 0);
                    setHasMore(Boolean(cache.hasMore));
                    firstPageLoadedRef.current = true;
                    hasLoadedOnceRef.current = true;
                    setIsLoading(false);
                    setIsRefreshing(false);
                    setLoadingMore(false);
                    return;
                } else {
                    localStorage.removeItem(`search:${filterKey}`);
                }
            } catch {}
        }
        fetchPage(0, true);
    }, [debouncedQuery, minUpvotes, selectedDifficulties, sortBy]);

    useEffect(() => {
        // Trigger title animation after component mounts
        setTitleVisible(true);
    }, []);

    // Handle body overflow when overlay is open
    useEffect(() => {
        // Create or update the style element for hiding scrollbars
        let styleElement = document.getElementById('hide-scrollbar-style') as HTMLStyleElement;

        if (selectedMap) {
            // Compute scrollbar width and pad body to avoid layout shift
            const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
            if (scrollbarWidth > 0) {
                document.body.style.paddingRight = `${scrollbarWidth}px`;
            }

            document.body.classList.add('no-scrollbar');
            document.body.style.overflow = 'hidden';
            document.body.style.scrollbarWidth = 'none'; // Firefox
            (document.body.style as any).msOverflowStyle = 'none'; // IE

            // Create style element to forcefully hide webkit scrollbars
            if (!styleElement) {
                styleElement = document.createElement('style');
                styleElement.id = 'hide-scrollbar-style';
                document.head.appendChild(styleElement);
            }

            styleElement.textContent = `
                html::-webkit-scrollbar, body::-webkit-scrollbar {
                    display: none !important;
                    width: 0 !important;
                    height: 0 !important;
                }
                html, body {
                    scrollbar-width: none !important;
                    -ms-overflow-style: none !important;
                }
            `;
        } else {
            document.body.classList.remove('no-scrollbar');
            document.body.style.overflow = '';
            document.body.style.scrollbarWidth = 'thin'; // Firefox
            (document.body.style as any).msOverflowStyle = ''; // IE
            document.body.style.paddingRight = '';

            // Remove the hide scrollbar style
            if (styleElement) {
                styleElement.remove();
            }
        }

        // Cleanup function to restore scroll when component unmounts
        return () => {
            document.body.classList.remove('no-scrollbar');
            document.body.style.overflow = '';
            document.body.style.scrollbarWidth = 'thin';
            (document.body.style as any).msOverflowStyle = '';
            document.body.style.paddingRight = '';

            const cleanupStyleElement = document.getElementById('hide-scrollbar-style');
            if (cleanupStyleElement) {
                cleanupStyleElement.remove();
            }
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

                // Hide difficulty chips to match the banner layout (they don't appear on the song-page banner)
                const difficultyContainers = animatedCard.querySelectorAll('div.mt-2');
                difficultyContainers.forEach((el) => {
                    (el as HTMLElement).style.display = 'none';
                });
                
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
                    // Capture original paddings so we can compensate during inverse scaling
                    const cs = getComputedStyle(animatedText);
                    (animatedText as any).dataset.padLeftOriginal = cs.paddingLeft || '0px';
                    (animatedText as any).dataset.padRightOriginal = cs.paddingRight || '0px';
                    (animatedText as any).dataset.padTopOriginal = cs.paddingTop || '0px';
                    (animatedText as any).dataset.padBottomOriginal = cs.paddingBottom || '0px';
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
                    
                    // Apply inverse scaling to text to keep it at original size,
                    // translate vertically to keep centered as height grows,
                    // and compensate for padding by scaling padding on all sides
                    if (animatedText) {
                        const deltaY = (bannerHeight - rect.height) / 2;
                        const padLeftOriginal = parseFloat((animatedText as any).dataset.padLeftOriginal || '0') || 0;
                        const padRightOriginal = parseFloat((animatedText as any).dataset.padRightOriginal || '0') || 0;
                        const padTopOriginal = parseFloat((animatedText as any).dataset.padTopOriginal || '0') || 0;
                        const padBottomOriginal = parseFloat((animatedText as any).dataset.padBottomOriginal || '0') || 0;
                        // Scale paddings so that after inverse scale the perceived padding matches the original
                        animatedText.style.paddingLeft = `${padLeftOriginal * scaleX}px`;
                        animatedText.style.paddingRight = `${padRightOriginal * scaleX}px`;
                        animatedText.style.paddingTop = `${padTopOriginal * scaleY}px`;
                        animatedText.style.paddingBottom = `${padBottomOriginal * scaleY}px`;
                        animatedText.style.transform = `translate(0px, ${deltaY}px) scale(${1/scaleX}, ${1/scaleY})`;
                    }
                    
                    setCardTransformed(true);

                    // One more frame to measure and correct horizontal alignment to match banner padding
                    requestAnimationFrame(() => {
                        const remainingCard = document.getElementById('animated-banner-card') as HTMLButtonElement | null;
                        if (!remainingCard) return;
                        const remainingText = remainingCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement | null;
                        if (!remainingText) return;
                        const targetLeftPadding = parseFloat(getComputedStyle(remainingText).paddingLeft || '24') || 24; // p-6 ~ 24px
                        const textRect = remainingText.getBoundingClientRect();
                        const currentLeft = textRect.left;
                        const bannerLeft = 0; // animated card is translated to viewport left
                        const desiredLeft = bannerLeft + targetLeftPadding;
                        const correction = desiredLeft - currentLeft;
                        // Preserve existing translateY and scale while adding X correction
                        const existing = remainingText.style.transform;
                        const match = existing.match(/translate\(([^,]+)px,\s*([^\)]+)px\)\s*scale\(([^,]+),\s*([^\)]+)\)/);
                        if (match) {
                            const ty = parseFloat(match[2]);
                            const sx = parseFloat(match[3]);
                            const sy = parseFloat(match[4]);
                            remainingText.style.transform = `translate(${correction}px, ${ty}px) scale(${sx}, ${sy})`;
                        } else {
                            // Fallback: reapply with zero Y if parse fails
                            remainingText.style.transform = `translate(${correction}px, 0px)`;
                        }
                    });
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

                // Re-enable truncation immediately to avoid snapping as we shrink
                existingAnimatedCard.classList.add('reclamp');

                // Get current banner dimensions
                const currentRect = existingAnimatedCard.getBoundingClientRect();
                const viewportWidth = currentRect.width;
                const bannerHeight = currentRect.height;

                // Do not force width/height, preserve inset-0 for proper centering during reverse
                if (animatedText) {
                    animatedText.style.width = '';
                    animatedText.style.height = '';
                }

                // Start animation
                requestAnimationFrame(() => {
                    existingAnimatedCard.style.transition = 'transform 400ms ease-out';
                    if (animatedText) {
                        animatedText.style.transition = 'transform 400ms ease-out';
                    }

                    requestAnimationFrame(() => {
                        // Animate back to original position
                        existingAnimatedCard.style.transform = `translate(0px, 0px) scale(1, 1)`;
                        if (animatedText) {
                            animatedText.style.transform = `scale(1, 1)`;
                        }

                        // Fallback cleanup in case fade out handler doesn't complete properly
                        setTimeout(() => {
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
                    
                    // Update text scaling and centering; re-apply scaled paddings for consistency on resize
                    const animatedText = animatedCard.querySelector('[class*="absolute inset-0 flex"]') as HTMLDivElement;
                    if (animatedText) {
                        const deltaY = (newBannerHeight - originalRect.height) / 2;
                        const padLeftOriginal = parseFloat((animatedText as any).dataset.padLeftOriginal || '0') || 0;
                        const padRightOriginal = parseFloat((animatedText as any).dataset.padRightOriginal || '0') || 0;
                        const padTopOriginal = parseFloat((animatedText as any).dataset.padTopOriginal || '0') || 0;
                        const padBottomOriginal = parseFloat((animatedText as any).dataset.padBottomOriginal || '0') || 0;
                        animatedText.style.paddingLeft = `${padLeftOriginal * scaleX}px`;
                        animatedText.style.paddingRight = `${padRightOriginal * scaleX}px`;
                        animatedText.style.paddingTop = `${padTopOriginal * scaleY}px`;
                        animatedText.style.paddingBottom = `${padBottomOriginal * scaleY}px`;
                        animatedText.style.transform = `translate(0px, ${deltaY}px) scale(${1/scaleX}, ${1/scaleY})`;
                        animatedText.style.width = '';
                        animatedText.style.height = '';

                        // Measure and correct horizontal alignment to match banner padding
                        const targetLeftPadding = parseFloat(getComputedStyle(animatedText).paddingLeft || '24') || 24;
                        const textRect = animatedText.getBoundingClientRect();
                        const currentLeft = textRect.left;
                        const bannerLeft = 0;
                        const desiredLeft = bannerLeft + targetLeftPadding;
                        const correction = desiredLeft - currentLeft;
                        animatedText.style.transform = `translate(${correction}px, ${deltaY}px) scale(${1/scaleX}, ${1/scaleY})`;
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

            // Navigate to song route with background location so Home stays mounted
            navigate(`/song/${map.id}`, { state: { backgroundLocation: location } });
        }, 50); // Small delay for React to update DOM
    };

    const handleCloseSongPage = useCallback(() => {
        console.log('Close song page clicked');
        
        if (selectedMap && transitioningCard) {
            // Start both fade out and reverse animation simultaneously
            setShouldFadeOut(true);
            setIsReverseAnimation(true);
            
            // Cleanup after animations finish (reverse animation takes ~400ms)
            setTimeout(() => {
                console.log('Cleanup after animations complete');

                // Unhide the original card first so there is never a gap
                setTransitioningCard(null);
                setIsReverseAnimation(false);
                setCardTransformed(false);

                // On the next frame, hide the song page so the card is already visible underneath
                requestAnimationFrame(() => {
                    setShowSongPage(false);

                    // Then clean up remaining state and remove the clone after another frame
                    setTimeout(() => {
                        setShouldFadeOut(false);
                        setSelectedMap(null);

                        requestAnimationFrame(() => {
                            const animatedCard = document.getElementById('animated-banner-card');
                            if (animatedCard && animatedCard.parentNode) {
                                animatedCard.parentNode.removeChild(animatedCard);
                            }
                            setHideButtons(null);
                        });
                    }, 20);
                });
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
    }, [selectedMap, transitioningCard, setSelectedMap]);

    const handleFadeOutComplete = () => {
        // This handler isn't working reliably, so we rely on the timeout-based cleanup
        console.log('Fade out complete called but using timeout-based cleanup instead');
    };

    // Listen for song closing triggered from SongPage (router back)
    useEffect(() => {
        const listener = () => {
            // Mirror clicking the back button overlay
            handleCloseSongPage();
        };
        window.addEventListener('song:closing', listener as EventListener);
        return () => window.removeEventListener('song:closing', listener as EventListener);
    }, [handleCloseSongPage]);

    const getCardClassName = (mapId: string) => {
        const baseClass = "block w-full aspect-[32/9] border border-black cursor-pointer overflow-hidden relative text-left bg-white rounded-lg";
        const shadowClass = "shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]";
        
        // Hide the transitioning card during both forward and reverse animations (clone handles the animation)
        if (transitioningCard === mapId) {
            return `${baseClass} transform-gpu invisible`;
        }
        
        // Hide other cards during forward animation only
        if (transitioningCard && transitioningCard !== mapId && !isReverseAnimation) {
            return `${baseClass} ${shadowClass} opacity-0 transition-opacity duration-200`;
        }
        
        return `${baseClass} ${shadowClass} transition-all duration-100 hover:translate-x-1 hover:translate-y-1 hover:shadow-none hover:scale-[1.02] active:scale-[0.98]`;
    };

    // Update available difficulties based on results without flicker
    useEffect(() => {
        const set = new Set<string>(availableDifficulties);
        for (const m of results) {
            (m.difficulties || []).forEach(d => set.add(d.display));
        }
        setAvailableDifficulties(Array.from(set).sort((a, b) => a.localeCompare(b)));
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [results]);

    const toggleDifficulty = (name: string) => {
        setSelectedDifficulties(prev => {
            const next = new Set(prev);
            if (next.has(name)) next.delete(name); else next.add(name);
            return next;
        });
    };

    // Infinite scroll: observe sentinel
    useEffect(() => {
        const el = sentinelRef.current;
        if (!el) return;

        // Shared guard + load function so both IO and scroll fallback use identical logic
        const maybeLoadMore = () => {
            if (!hasMore) return;
            if (isLoading || loadingMore || isRefreshing) return;
            if (transitioningCard || isReverseAnimation) return;
            if (pagingRef.current) return;
            if (!firstPageLoadedRef.current) return; // wait for initial page to complete
            const now = Date.now();
            if (now - lastLoadAtRef.current < 500) return; // cooldown 500ms
            setPage((prev) => {
                const next = prev + 1;
                pagingRef.current = true;
                lastLoadAtRef.current = now;
                fetchPage(next, false);
                return next;
            });
        };

        // IntersectionObserver for primary infinite scroll
        const observer = new IntersectionObserver(
            (entries) => {
                const first = entries[0];
                if (first && first.isIntersecting) {
                    maybeLoadMore();
                }
            },
            {
                root: null,
                // Reasonable eagerness; prevents constant firing far away from viewport
                rootMargin: '400px 0px 400px 0px',
                threshold: 0,
            }
        );
        observer.observe(el);

        // No window scroll fallback; IO is sufficient and avoids duplicate trigger spam

        // If sentinel is already visible, only try after initial page is done
        const initialRect = el.getBoundingClientRect();
        if (firstPageLoadedRef.current && !isLoading && !loadingMore && !isRefreshing && initialRect.top < window.innerHeight + 200) {
            maybeLoadMore();
        }

        return () => {
            observer.disconnect();
        };
    }, [hasMore, isLoading, loadingMore, isRefreshing, fetchPage, transitioningCard, isReverseAnimation]);

    // Release pagination lock when network states settle
    useEffect(() => {
        if (!isLoading && !loadingMore && !isRefreshing) {
            pagingRef.current = false;
        }
    }, [isLoading, loadingMore, isRefreshing]);

    return (
        <div className="relative min-h-screen">
            {/* Search Grid */}
            <div className="max-w-6xl mx-auto p-4">
                <h1
                    className={`text-2xl mb-6 font-['Press_Start_2P'] pixel-title tracking-wide text-center transform transition-all duration-300 ${
                        titleVisible ? 'translate-y-0 opacity-100' : '-translate-y-5 opacity-100'
                    } ${transitioningCard && !isReverseAnimation ? 'opacity-0' : ''}`}
                >
                    Beatmap Browser
                </h1>
                {/* Pixel-styled search with filters: search on its own line, filters on one row below */}
                <div className={`sticky top-0 z-20 ${transitioningCard && !isReverseAnimation ? 'opacity-0 pointer-events-none' : 'opacity-100'} transition-opacity mb-6`}>
                    <div className="pixel-panel rounded-md bg-white/95 backdrop-blur-sm">
                        <div className="max-w-6xl mx-auto p-4 pt-3">
                            {/* Row 1: Search */}
                            <div className="flex items-center gap-2">
                                <input
                                    value={query}
                                    onChange={(e) => setQuery(e.target.value)}
                                    placeholder="Search song, artist, or charter..."
                                    className="w-full px-3 py-3 font-['Press_Start_2P'] text-sm border border-black bg-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] focus:outline-none"
                                />
                            </div>

                            {/* Row 2: Filters (all on one row) */}
                            <div className="mt-2 flex items-center gap-2">
                                {/* Difficulty chips - horizontally scrollable and takes remaining space */}
                                <div className="flex-1 flex items-center gap-2 overflow-x-auto whitespace-nowrap py-1 px-1">
                                    {availableDifficulties.map((d) => (
                                        <button
                                            key={d}
                                            onClick={() => toggleDifficulty(d)}
                                            className={`px-2 py-1 text-xs font-['Press_Start_2P'] border border-black transition-all ${
                                                selectedDifficulties.has(d)
                                                    ? 'bg-purple-500 text-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]'
                                                    : 'bg-white text-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:bg-gray-100'
                                            }`}
                                            aria-pressed={selectedDifficulties.has(d)}
                                        >
                                            {d}
                                        </button>
                                    ))}
                                </div>

                                {/* Min upvotes compact input with placeholder and no arrows */}
                                <input
                                    inputMode="numeric"
                                    pattern="[0-9]*"
                                    value={minUpvotes === '' ? '' : String(minUpvotes)}
                                    onChange={(e) => {
                                        const v = e.target.value.replace(/\D+/g, '');
                                        setMinUpvotes(v === '' ? '' : Math.max(0, Number(v)));
                                    }}
                                    placeholder="Min ↑"
                                    className="w-24 px-2 py-2 font-['Press_Start_2P'] text-xs border border-black bg-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] focus:outline-none"
                                />

                                {/* Sort selector */}
                                <select
                                    value={sortBy}
                                    onChange={(e) => setSortBy(e.target.value as any)}
                                    className="px-3 py-3 font-['Press_Start_2P'] text-xs border border-black bg-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] focus:outline-none"
                                    aria-label="Sort results"
                                >
                                    <option value="relevance">Sort: Relevance</option>
                                    <option value="upvotes">Sort: Most Upvoted</option>
                                    <option value="newest">Sort: Newest</option>
                                </select>

                                {/* Results count and inline spinner */}
                                <div className="flex items-center gap-2">
                                    {isRefreshing && (
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-black" />
                                    )}
                                    <div className="font-['Press_Start_2P'] text-xs text-gray-700">{totalCount} results</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
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

                                <div className="absolute inset-0 card-overlay-gradient" />

                                <div 
                                    ref={(el) => { textRefs.current[map.id] = el; }}
                                    className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-6"
                                >
                                    <div className="flex-1 min-w-0">
                                        <h2 className="text-xl mb-1 line-clamp-1">{map.song}</h2>
                                        <p className="text-sm">by {map.artist}</p>
                                        <p className="text-sm">Charter: {map.charter}</p>
                                        {map.difficulties && map.difficulties.length > 0 && (
                                            <div className="mt-2 flex flex-wrap gap-2">
                                                {map.difficulties.slice(0, 3).map((d) => (
                                                    <span key={d.display} className="px-2 py-0.5 text-[10px] font-['Press_Start_2P'] border border-white/60 bg-black/40 rounded-sm">
                                                        {d.display}
                                                    </span>
                                                ))}
                                                {map.difficulties.length > 3 && (
                                                    <span className="px-2 py-0.5 text-[10px] font-['Press_Start_2P'] border border-white/40 bg-black/20 rounded-sm">+{map.difficulties.length - 3}</span>
                                                )}
                                            </div>
                                        )}
                                    </div>
                                    <div className={`absolute bottom-4 right-4 flex items-center gap-2 ease-in-out ${
                                        hideButtons === map.id 
                                            ? 'opacity-0 scale-90 transition-all duration-300' 
                                            : 'opacity-100 scale-100 transition-all duration-150'
                                    }`}>
                                        <div className="group relative">
                                            <a
                                                href={`https://beatmap-browser.s3.amazonaws.com/${map.id}.zip`}
                                                className="inline-flex items-center justify-center px-3 py-2 pixel-btn bg-blue-500 text-white rounded-sm hover:bg-blue-600"
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
                                                className="inline-flex items-center justify-center px-3 py-2 pixel-btn bg-green-500 text-white rounded-sm hover:bg-green-600"
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
                                        <div className="pixel-panel bg-white/90 px-3 py-1 rounded-sm">
                                            <span className="text-sm text-black">↑ {map.upvotes}</span>
                                        </div>
                                    </div>
                                </div>
                            </button>
                        ))}
                        {/* Sentinel for infinite scroll */}
                        <div ref={sentinelRef} className="col-span-full h-6" />
                        {loadingMore && (
                            <div className="col-span-full flex justify-center py-4">
                                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-black"></div>
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Song overlay/back button now handled by router-level modal in App.tsx */}
        </div>
    );
}