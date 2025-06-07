import React, { useEffect, useState } from "react";
import { BeatMap } from "@/schema";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';

interface SongPageProps {
    map: BeatMap;
    cardPosition: DOMRect;
    onBack: () => void;
}

export default function SongPage({ map, cardPosition, onBack }: SongPageProps) {
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [contentVisible, setContentVisible] = useState(false);

    useEffect(() => {
        // Delay showing content until after the banner animation
        setTimeout(() => {
            setContentVisible(true);
        }, 600);
    }, []);

    if (error) return <div className="text-red-500 text-center font-['Press_Start_2P']">Error: {error}</div>;

    return (
        <div className="min-h-screen bg-gray-100">
            {/* Static Banner */}
            <div className="relative h-64 w-full">
                <button
                    onClick={onBack}
                    className="absolute top-4 left-4 z-10 bg-white/90 hover:bg-white text-black p-2 rounded-full shadow-lg transition-colors"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                </button>
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
            <div className="max-w-6xl mx-auto p-4">
                <div 
                    className={`transition-all duration-500 ease-out ${
                        contentVisible 
                            ? 'opacity-100 translate-y-0' 
                            : 'opacity-0 translate-y-8'
                    }`}
                >
                    {loading ? (
                        <div className="text-center font-['Press_Start_2P']">Loading details...</div>
                    ) : (
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
                    )}
                </div>
            </div>
        </div>
    );
} 