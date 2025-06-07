import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { BeatMap } from "@/schema";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';

export default function SongPage() {
    const { id } = useParams();
    const [map, setMap] = useState<BeatMap | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchMap = async () => {
            setLoading(true);
            setError(null);
            try {
                const res = await fetch(`/api/map/${id}`);
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                const data = await res.json();
                setMap(data);
            } catch (e: any) {
                setError(e.message || "Unknown error");
            } finally {
                setLoading(false);
            }
        };
        fetchMap();
    }, [id]);

    if (loading) return <div className="text-center font-['Press_Start_2P']">Loading...</div>;
    if (error) return <div className="text-red-500 text-center font-['Press_Start_2P']">Error: {error}</div>;
    if (!map) return <div className="text-center font-['Press_Start_2P']">Map not found</div>;

    return (
        <div className="min-h-screen bg-gray-100">
            {/* Banner */}
            <div className="relative h-64 w-full">
                <img 
                    src={map.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : default_image}
                    alt={map.song}
                    className="absolute inset-0 w-full h-full object-cover"
                />
                <div className="absolute inset-0 bg-black/60" />
                <div className="absolute inset-0 flex flex-col justify-end p-8 text-white">
                    <h1 className="text-4xl font-['Press_Start_2P'] mb-2">{map.song}</h1>
                    <p className="text-xl font-['Press_Start_2P'] mb-1">by {map.artist}</p>
                    <p className="text-xl font-['Press_Start_2P']">Charter: {map.charter}</p>
                </div>
            </div>

            {/* Main Content */}
            <div className="max-w-7xl mx-auto p-4">
                <div className="flex gap-6">
                    {/* Left Sidebar */}
                    <div className="w-64 flex-shrink-0">
                        <div className="border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                            <h2 className="text-xl font-['Press_Start_2P'] mb-4">Details</h2>
                            <div className="space-y-2">
                                <p className="font-['Press_Start_2P'] text-sm">
                                    <span className="text-gray-600">Upvotes:</span> {map.upvotes}
                                </p>
                                {map.difficulties && (
                                    <p className="font-['Press_Start_2P'] text-sm">
                                        <span className="text-gray-600">Difficulties:</span> {map.difficulties.join(", ")}
                                    </p>
                                )}
                            </div>
                        </div>

                        <div className="mt-4 border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                            <h2 className="text-xl font-['Press_Start_2P'] mb-4">Download</h2>
                            <div className="space-y-4">
                                <a 
                                    href={`https://beatmap-browser.s3.amazonaws.com/${map.id}.zip`}
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
    );
} 