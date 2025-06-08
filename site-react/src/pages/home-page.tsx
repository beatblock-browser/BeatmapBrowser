import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { SearchRequest, SearchResult } from "@/schema/search";
import { BeatMap } from "@/schema";
import { useSearchCache } from "@/context/SearchCache";
import { useStore } from "@/lib/store";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';
import SongPage from "./song-page";

export default function HomePage() {
    const { results, setResults, isLoading, setIsLoading } = useSearchCache();
    const { selectedMap, setSelectedMap } = useStore();
    const [error, setError] = useState<string | null>(null);

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

    const handleCardClick = (map: BeatMap) => {
        setSelectedMap(map);
    };

    return (
        <div className="relative min-h-screen bg-white">
            {/* Search Grid */}
            <motion.div
                className="max-w-6xl mx-auto p-4"
                animate={{
                    opacity: selectedMap ? 0.3 : 1,
                    scale: selectedMap ? 0.9 : 1
                }}
                transition={{ duration: 0.4, ease: "easeInOut" }}
                style={{ pointerEvents: selectedMap ? 'none' : 'auto' }}
            >
                <motion.h1
                    initial={{ y: -20 }}
                    animate={{ y: 0 }}
                    className="text-2xl mb-6 font-['Press_Start_2P'] text-center"
                >
                    Beatmap Browser
                </motion.h1>

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
                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="grid grid-cols-1 md:grid-cols-2 gap-4"
                    >
                        {results.length === 0 && (
                            <div className="col-span-full text-center font-['Press_Start_2P'] py-8">
                                No beatmaps found. Try refreshing the page.
                            </div>
                        )}
                        {results.map((map) => (
                            <motion.button
                                key={map.id}
                                layoutId={`card-${map.id}`}
                                onClick={() => handleCardClick(map)}
                                whileHover={{ scale: 1.02 }}
                                whileTap={{ scale: 0.98 }}
                                className="block w-full aspect-[32/9] border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]
                                         hover:translate-x-1 hover:translate-y-1 hover:shadow-none
                                         transition-all duration-100 cursor-pointer overflow-hidden relative text-left bg-white"
                                style={{
                                    willChange: selectedMap?.id === map.id ? 'transform' : 'auto'
                                }}
                                transition={{
                                    layout: { duration: 0.6, ease: [0.25, 0.1, 0.25, 1] }
                                }}
                            >
                                {/* Use regular img for better performance */}
                                <img
                                    src={map.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : default_image}
                                    alt={map.song}
                                    className="absolute inset-0 w-full h-full object-cover"
                                    loading="lazy"
                                    style={{ willChange: 'auto' }}
                                />
                                <div className="absolute inset-0 bg-black/60" />
                                <div className="relative h-full flex flex-row items-center font-['Press_Start_2P'] text-xs text-white p-2">
                                    <div className="flex-1 min-w-0">
                                        <h2 className="text-sm mb-0.5 line-clamp-1">{map.song}</h2>
                                        <p className="text-gray-200 mb-0.5 text-[10px]">by {map.artist}</p>
                                        <p className="text-gray-200 text-[10px]">Charter: {map.charter}</p>
                                        <div className="flex items-center gap-1 mt-1">
                                            <span className="text-[10px]">Upvotes:</span>
                                            <span className="text-sm">{map.upvotes}</span>
                                        </div>
                                    </div>
                                </div>
                            </motion.button>
                        ))}
                    </motion.div>
                )}
            </motion.div>

            {/* Song Page Overlay */}
            {selectedMap && (
                <div className="absolute inset-0 z-10">
                    <SongPage />
                </div>
            )}
        </div>
    );
}