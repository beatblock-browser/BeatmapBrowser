import React, { useEffect } from "react";
import { SearchRequest, SearchResult } from "@/schema/search";
import { BeatMap } from "@/schema";
import { useSearchCache } from "@/context/SearchCache";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';

interface HomePageProps {
    onCardClick: (map: BeatMap, cardElement: HTMLElement) => void;
}

export default function HomePage({ onCardClick }: HomePageProps) {
    const { results, setResults, isLoading, setIsLoading } = useSearchCache();
    const [error, setError] = React.useState<string | null>(null);

    useEffect(() => {
        const fetchResults = async () => {
            // Only fetch if we don't have results
            if (results.length > 0) return;

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
                setError(e.message || "Unknown error");
            } finally {
                setIsLoading(false);
            }
        };
        fetchResults();
    }, [results.length, setResults, setIsLoading]);

    const handleCardClick = (map: BeatMap, e: React.MouseEvent<HTMLButtonElement>) => {
        const rect = e.currentTarget.getBoundingClientRect();
        onCardClick(map, e.currentTarget);
    };

    return (
        <div className="max-w-6xl mx-auto p-4">
            <h1 className="text-2xl mb-6 font-['Press_Start_2P'] text-center">Beatmap Browser</h1>
            {isLoading && <div className="text-center font-['Press_Start_2P']">Loading...</div>}
            {error && <div className="text-red-500 text-center font-['Press_Start_2P']">Error: {error}</div>}
            {!isLoading && !error && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {results.length === 0 && (
                        <div className="col-span-full text-center font-['Press_Start_2P']">No results found.</div>
                    )}
                    {results.map((map) => (
                        <button 
                            key={map.id}
                            onClick={(e) => handleCardClick(map, e)}
                            className="block w-full aspect-[32/9] border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] 
                                     hover:translate-x-1 hover:translate-y-1 hover:shadow-none 
                                     transition-all duration-100 cursor-pointer overflow-hidden relative text-left"
                        >
                            <img 
                                src={map.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : default_image}
                                alt={map.song}
                                className="absolute inset-0 w-full h-full object-cover"
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
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}
