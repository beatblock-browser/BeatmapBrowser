import React, { useEffect, useState, useRef, useCallback, useMemo, startTransition } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { BeatMap } from "@/schema";
import { useSearchCache } from "@/context/SearchCache";
import { apiFetch } from "@/lib/api";
import SongCard from "@/components/SongCard";
import { setCachedMap } from "@/lib/mapCache";

// Preferred difficulty order; also used to seed the available list so all appear
const DIFFICULTY_ORDER: string[] = [
    'Apocraphyia',
    'Challenge',
    'Hard',
    'Easy',
    'Special',
];

function normalizeDifficulty(name: string): string {
    if (!name) return '';
    // Remove zero-width and BOM characters, normalize whitespace
    const cleaned = name
        .replace(/[\u200B-\u200D\uFEFF]/g, '')
        .replace(/\s+/g, ' ')
        .trim();
    const lower = cleaned.toLowerCase();
    // Exact canonical match first
    const exact = DIFFICULTY_ORDER.find((n) => n.toLowerCase() === lower);
    if (exact) return exact;
    // Fuzzy prefix mapping to canonical
    if (lower.startsWith('apoc')) return 'Apocraphyia';
    if (lower.startsWith('chall')) return 'Challenge';
    if (lower.startsWith('hard')) return 'Hard';
    if (lower.startsWith('easy')) return 'Easy';
    if (lower.startsWith('spec')) return 'Special';
    return cleaned;
}

export default function HomePage() {
    const navigate = useNavigate();
    const location = useLocation();
    const { results, setResults } = useSearchCache();
    const [error, setError] = useState<string | null>(null);
    const [jwt, setJwt] = useState<string | null>(null);
    // Search and filter state
    const [query, setQuery] = useState("");
    const [debouncedQuery, setDebouncedQuery] = useState("");
    const [minUpvotes, setMinUpvotes] = useState<number | "">("");
    const [selectedDifficulties, setSelectedDifficulties] = useState<Set<string>>(new Set());
    const [sortBy, setSortBy] = useState<"relevance" | "upvotes" | "newest">("relevance");
    const [page, setPage] = useState(0);
    const [hasMore, setHasMore] = useState(true);
    const [isLoading, setIsLoading] = useState(false);
    const [loadingMore, setLoadingMore] = useState(false);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const availableDifficulties = useMemo<string[]>(() => {
        // Use fixed preset list regardless of what appears in results
        return [...DIFFICULTY_ORDER];
    }, []);
    const [totalCount, setTotalCount] = useState<number>(0);
    const sentinelRef = useRef<HTMLDivElement | null>(null);
    const requestIdRef = useRef(0);
    // Guards for pagination to avoid duplicate requests
    const pagingRef = useRef(false);
    const lastLoadAtRef = useRef(0);
    const firstPageLoadedRef = useRef(false);
    const hasLoadedOnceRef = useRef(false);
    const currentFilterRef = useRef<string>("");
    const lastCompletedFilterRef = useRef<string>("");
    // Debounced empty-state to avoid flicker
    const [showEmpty, setShowEmpty] = useState(false);
    const emptyTimerRef = useRef<number | null>(null);
    const [showFilters, setShowFilters] = useState(false);

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
            // Reset paging lock because we're starting fresh
            pagingRef.current = false;
            // Do not clear results; we'll dim the UI while refreshing to avoid stale flashes
        }
        startTransition(() => {
            setIsLoading(isInitial);
            setIsRefreshing(reset && !isInitial);
            setLoadingMore(!reset);
            setError(null);
        });
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
            try {
                // Debug snapshot: selected filters and numeric difficulties in the payload
                const snapshot = (data.results || []).slice(0, 20).map(m => ({
                    id: m.id,
                    diffs: (m.difficulties || []).map(d => ({ display: (d as any).display, difficulty: (d as any).difficulty }))
                }));
                // eslint-disable-next-line no-console
                console.info('[HomePage] search debug', {
                    selectedDifficulties: Array.from(selectedDifficulties),
                    page: pageToLoad,
                    hasMore: Boolean(data.has_more),
                    totalCount: data.total_count,
                    snapshot,
                });
            } catch {}
            // Drop responses that are stale by id or mismatch current filters
            if (reqId !== requestIdRef.current || filterKey !== currentFilterRef.current) {
                // stale response; ignore
                return;
            }
            startTransition(() => {
                setHasMore(Boolean(data.has_more));
                if (typeof data.total_count === 'number') setTotalCount(data.total_count);
            });
            let cachedResults: BeatMap[] = [];
            if (reset) {
                const next = (data.results || []);
                next.forEach(map => setCachedMap(map.id, map));
                startTransition(() => setResults(next));
                cachedResults = next;
            } else {
                startTransition(() => {
                    setResults(prev => {
                        const next = [...prev, ...(data.results || [])];
                        (data.results || []).forEach(map => setCachedMap(map.id, map));
                        cachedResults = next;
                        return next;
                    });
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
                lastCompletedFilterRef.current = filterKey;
            }
        } catch (e: any) {
            console.error("Failed to fetch results:", e);
            startTransition(() => {
                setError(e.message || "Failed to load beatmaps. Please try again later.");
                if (reset) setResults([]);
            });
        } finally {
            startTransition(() => {
                setIsLoading(false);
                setIsRefreshing(false);
                setLoadingMore(false);
            });
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
        if (diffsParam) setSelectedDifficulties(new Set(
            diffsParam
                .split(',')
                .filter(Boolean)
                .map(normalizeDifficulty)
        ));
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
                        startTransition(() => {
                            setResults(cache.results || []);
                            setTotalCount(cache.totalCount || 0);
                            setHasMore(Boolean(cache.hasMore));
                            setIsLoading(false);
                            setIsRefreshing(false);
                            setLoadingMore(false);
                        });
                        firstPageLoadedRef.current = true;
                        hasLoadedOnceRef.current = true;
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
                    startTransition(() => {
                        setResults(cache.results || []);
                        setTotalCount(cache.totalCount || 0);
                        setHasMore(Boolean(cache.hasMore));
                        setIsLoading(false);
                        setIsRefreshing(false);
                        setLoadingMore(false);
                    });
                    firstPageLoadedRef.current = true;
                    hasLoadedOnceRef.current = true;
                    return;
                } else {
                    localStorage.removeItem(`search:${filterKey}`);
                }
            } catch {}
        }
        fetchPage(0, true);
    }, [debouncedQuery, minUpvotes, selectedDifficulties, sortBy]);

    // Debounce empty-state to prevent brief flashes between network state transitions
    useEffect(() => {
        // Never show empty before first page completes
        if (!hasLoadedOnceRef.current || !firstPageLoadedRef.current) {
            if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
            setShowEmpty(false);
            return;
        }
        // Only consider empty if current filter's first page completed
        if (lastCompletedFilterRef.current !== currentFilterRef.current) {
            if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
            setShowEmpty(false);
            return;
        }
        const noResults = results.length === 0;
        const loadingAny = isLoading || isRefreshing || loadingMore;
        if (noResults && !loadingAny) {
            if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
            emptyTimerRef.current = window.setTimeout(() => setShowEmpty(true), 500);
        } else {
            if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
            setShowEmpty(false);
        }
        return () => {
            if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
            emptyTimerRef.current = null;
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [results.length, isLoading, isRefreshing, loadingMore]);

    // Clear empty-state immediately when filters change, to avoid transient flashes
    useEffect(() => {
        if (emptyTimerRef.current) window.clearTimeout(emptyTimerRef.current);
        setShowEmpty(false);
    }, [debouncedQuery, minUpvotes, selectedDifficulties, sortBy]);

    // No JS resize logic; rely purely on CSS breakpoints to prevent flicker

    useEffect(() => {
        setJwt(localStorage.getItem('jwt'));
    }, []);

    // Refresh search after a map is deleted (from song page)
    useEffect(() => {
        const onDeleted = (evt: Event) => {
            const anyEvt = evt as CustomEvent<{ id?: string }>;
            const deletedId = anyEvt.detail?.id;
            if (!deletedId) return;
            // Optimistically remove from current results
            startTransition(() => {
                setResults(prev => prev.filter(m => m.id === deletedId ? false : true));
            });
            // Trigger a refresh for the current filters
            setPage(0);
            fetchPage(0, true);
        };
        window.addEventListener('map:deleted', onDeleted);
        return () => window.removeEventListener('map:deleted', onDeleted);
        // fetchPage is stable due to useCallback deps; setResults from context
    }, [fetchPage, setResults]);


    const handleCardClick = (map: BeatMap) => {
        navigate(`/song/${map.id}`);
    };

    const handleCardHover = useCallback(async (mapId: string) => {
        try {
            const response = await apiFetch(`/api/map/${mapId}`);
            if (response.ok) {
                const data: BeatMap = await response.json();
                setCachedMap(mapId, data);
            }
        } catch {}
    }, []);

    const getCardClassName = () => {
        return "block w-full aspect-[32/9] border border-black cursor-pointer overflow-hidden relative text-left bg-white rounded-lg shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transition-all duration-100 hover:translate-x-1 hover:translate-y-1 hover:shadow-none hover:scale-[1.02] active:scale-[0.98]";
    };

    const toggleDifficulty = (name: string) => {
        const normalized = normalizeDifficulty(name);
        setSelectedDifficulties(prev => {
            const next = new Set(prev);
            if (next.has(normalized)) next.delete(normalized); else next.add(normalized);
            // eslint-disable-next-line no-console
            console.info('[HomePage] toggleDifficulty', {
                clicked: name,
                normalized,
                next: Array.from(next),
            });
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
            if (pagingRef.current) return;
            if (!firstPageLoadedRef.current) return;
            const now = Date.now();
            if (now - lastLoadAtRef.current < 500) return;
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
    }, [hasMore, isLoading, loadingMore, isRefreshing, fetchPage]);

    // Release pagination lock when network states settle
    useEffect(() => {
        if (!isLoading && !loadingMore && !isRefreshing) {
            pagingRef.current = false;
        }
    }, [isLoading, loadingMore, isRefreshing]);

    return (
        <div className="relative min-h-[calc(100vh-56px)]">
            <div className="max-w-6xl mx-auto p-4">
                <div className="sticky top-0 z-20 mb-6">
                    <div className="pixel-panel rounded-md bg-white/95 backdrop-blur-sm">
                        <div className="max-w-6xl mx-auto px-4 pt-3 pb-2">
                            {/* Row 1: Search */}
                            <div className="flex items-center gap-3 justify-between">
                                <div className="search-wrapper w-full max-w-full flex-1">
                                    <svg className="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
                                        <circle cx="11" cy="11" r="8"/>
                                        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                                    </svg>
                                    <input
                                        value={query}
                                        onChange={(e) => setQuery(e.target.value)}
                                        placeholder="Search songs, artists, or charters…"
                                        className="w-full h-12 text-[24px] leading-none pl-8 pr-10 font-['Press_Start_2P'] pixel-control pixel-focus"
                                        aria-label="Search beatmaps"
                                    />
                                    {query && (
                                        <button
                                            type="button"
                                            className="search-clear pixel-focus"
                                            aria-label="Clear search"
                                            onClick={() => setQuery('')}
                                        >
                                            ✕
                                        </button>
                                    )}
                                </div>
                                <div className="flex items-center gap-2 flex-shrink-0" />
                            </div>

                            {/* Row 2: Filters */}
                            <div className="mt-2 flex items-center justify-between gap-3">
                                {/* Left: Filters button and inline chips when not collapsed */}
                                <div className="flex-1 min-w-0 flex items-center gap-2">
                                    <button
                                        className={`pixel-btn px-3 py-2 text-xs font-['Press_Start_2P'] lg:hidden ${showFilters ? 'bg-purple-600 text-white' : 'bg-white'}`}
                                        type="button"
                                        onClick={() => setShowFilters((v) => !v)}
                                        aria-expanded={showFilters}
                                        aria-controls="filters-panel"
                                    >
                                        Filters{selectedDifficulties.size ? ` (${selectedDifficulties.size})` : ''}
                                    </button>
                                    <div
                                        className="hidden lg:flex flex-1 min-w-0 items-center gap-2 overflow-x-auto whitespace-nowrap py-1 px-1"
                                    >
                                        {[...new Set(availableDifficulties)].map((d) => (
                                            <button
                                                key={d}
                                                onClick={() => toggleDifficulty(d)}
                                                className="pixel-chip"
                                                aria-pressed={selectedDifficulties.has(d)}
                                            >
                                                {d}
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {/* Right group: Min, Sort (visible on lg+) */}
                                <div className="hidden lg:flex items-center gap-2 flex-shrink-0">
                                    <input
                                        inputMode="numeric"
                                        pattern="[0-9]*"
                                        value={minUpvotes === '' ? '' : String(minUpvotes)}
                                        onChange={(e) => {
                                            const v = e.target.value.replace(/\D+/g, '').slice(0, 4);
                                            const n = v === '' ? '' : Math.max(0, Math.min(9999, Number(v)));
                                            setMinUpvotes(n);
                                        }}
                                        maxLength={4}
                                        placeholder="Min ↑"
                                        className="w-24 h-9 px-3 text-[11px] leading-none font-['Press_Start_2P'] pixel-control pixel-focus"
                                        aria-label="Minimum upvotes"
                                    />

                                    <select
                                        value={sortBy}
                                        onChange={(e) => setSortBy(e.target.value as any)}
                                        className="h-9 px-3 text-[11px] leading-none font-['Press_Start_2P'] pixel-control pixel-focus"
                                        aria-label="Sort results"
                                    >
                                        <option value="relevance">Sort: Relevance</option>
                                        <option value="upvotes">Sort: Most Upvoted</option>
                                        <option value="newest">Sort: Newest</option>
                                    </select>
                                </div>
                            </div>

                            {/* Collapsible Filters Panel (small screens) */}
                            {showFilters && (
                                <div id="filters-panel" className="lg:hidden mt-3 p-3 border-t border-black">
                                    <div className="flex flex-wrap items-center gap-2 mb-3">
                                        {[...new Set(availableDifficulties)].map((d) => (
                                            <button
                                                key={d}
                                                onClick={() => toggleDifficulty(d)}
                                                className="pixel-chip"
                                                aria-pressed={selectedDifficulties.has(d)}
                                            >
                                                {d}
                                            </button>
                                        ))}
                                    </div>
                                    <div className="flex items-center gap-2 flex-wrap">
                                        <input
                                            inputMode="numeric"
                                            pattern="[0-9]*"
                                            value={minUpvotes === '' ? '' : String(minUpvotes)}
                                            onChange={(e) => {
                                                const v = e.target.value.replace(/\D+/g, '').slice(0, 4);
                                                const n = v === '' ? '' : Math.max(0, Math.min(9999, Number(v)));
                                                setMinUpvotes(n);
                                            }}
                                            maxLength={4}
                                            placeholder="Min ↑"
                                            className="w-24 h-9 px-3 text-[11px] leading-none font-['Press_Start_2P'] pixel-control pixel-focus"
                                            aria-label="Minimum upvotes"
                                        />
                                        <select
                                            value={sortBy}
                                            onChange={(e) => setSortBy(e.target.value as any)}
                                            className="h-9 px-3 text-[11px] leading-none font-['Press_Start_2P'] pixel-control pixel-focus"
                                            aria-label="Sort results"
                                        >
                                            <option value="relevance">Sort: Relevance</option>
                                            <option value="upvotes">Sort: Most Upvoted</option>
                                            <option value="newest">Sort: Newest</option>
                                        </select>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                {/* Second line outside filter box: Results count (no spinner) */}
                <div className="max-w-6xl mx-auto px-4 mt-2 mb-4 flex items-center justify-end gap-2">
                    <div className="h-6 flex items-center text-[14px] font-['Press_Start_2P'] text-gray-700 select-none tabular-nums whitespace-nowrap">
                        {hasLoadedOnceRef.current ? totalCount.toLocaleString() : '…'} results
                    </div>
                </div>
                {error && (
                    <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4">
                        <strong className="font-['Press_Start_2P'] block mb-2">Error:</strong>
                        <p className="font-['Press_Start_2P'] text-sm">{error}</p>
                    </div>
                )}
                {!error && (
                    <div className={`grid grid-cols-1 md:grid-cols-2 gap-4 ${isRefreshing ? 'opacity-60 pointer-events-none transition-opacity' : ''}`}>
                        {!hasLoadedOnceRef.current ? (
                            <>
                                {Array.from({ length: 6 }).map((_, i) => (
                                    <div key={`skeleton-${i}`} className="block w-full aspect-[32/9] border border-black overflow-hidden relative bg-white rounded-lg shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]">
                                        <div className="absolute inset-0 animate-pulse">
                                            <div className="w-full h-full bg-gray-200" />
                                            <div className="absolute inset-0 p-6 flex items-end">
                                                <div className="w-2/3 h-6 bg-gray-300" />
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </>
                        ) : (
                            <>
                                {showEmpty && (
                                    <div className="col-span-full text-center font-['Press_Start_2P'] py-8">
                                        No beatmaps found. Try refreshing the page.
                                    </div>
                                )}
                                {results.map((map) => (
                                  <SongCard
                                    key={map.id}
                                    map={map}
                                    className={getCardClassName()}
                                    onClick={() => handleCardClick(map)}
                                    onKeyDown={(e) => {
                                      if (e.key === 'Enter' || e.key === ' ') {
                                        e.preventDefault();
                                        handleCardClick(map);
                                      }
                                    }}
                                    onMouseEnter={() => handleCardHover(map.id)}
                                    jwt={jwt}
                                  />
                                ))}
                            </>
                        )}
                        {/* Sentinel for infinite scroll (no spinner) */}
                        <div ref={sentinelRef} className="col-span-full h-6" />
                    </div>
                )}
            </div>

            {/* Song overlay/back button now handled by router-level modal in App.tsx */}
        </div>
    );
}