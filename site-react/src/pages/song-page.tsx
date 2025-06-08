import React from "react";
import { motion } from "framer-motion";
import { useStore } from "@/lib/store";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';

export default function SongPage() {
    const { selectedMap, setSelectedMap } = useStore();

    if (!selectedMap) return null;

    return (
        <div className="min-h-screen bg-white">
            {/* Banner - Morphs from the card */}
            <motion.div
                layoutId={`card-${selectedMap.id}`}
                className="relative w-full bg-white"
                style={{
                    height: 'calc(100vw * 9 / 32)',
                    maxHeight: '400px',
                    willChange: 'transform'
                }}
                transition={{
                    layout: { duration: 0.6, ease: [0.25, 0.1, 0.25, 1] }
                }}
            >
                <motion.button
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.6, duration: 0.3 }}
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    className="absolute top-4 left-4 z-30 bg-white/90 hover:bg-white text-black p-2 rounded-full shadow-lg transition-colors"
                    onClick={() => setSelectedMap(null)}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                </motion.button>

                {/* Use regular img for performance */}
                <img
                    src={selectedMap.image ? `https://beatmap-browser.s3.amazonaws.com/${selectedMap.id}.png` : default_image}
                    alt={selectedMap.song}
                    className="absolute inset-0 w-full h-full object-cover"
                />

                <div className="absolute inset-0 bg-black/60" />

                <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.4, duration: 0.5 }}
                    className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-8"
                >
                    <div className="flex-1 min-w-0">
                        <h1 className="text-4xl mb-2 line-clamp-1">{selectedMap.song}</h1>
                        <p className="text-xl mb-1">by {selectedMap.artist}</p>
                        <p className="text-xl">Charter: {selectedMap.charter}</p>
                        <div className="flex items-center gap-2 mt-2">
                            <span className="text-lg">Upvotes:</span>
                            <span className="text-2xl">{selectedMap.upvotes}</span>
                        </div>
                    </div>
                </motion.div>
            </motion.div>

            {/* Main Content */}
            <motion.div
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.5, duration: 0.5 }}
                className="max-w-6xl mx-auto p-4"
            >
                <div className="flex gap-6">
                    {/* Left Sidebar */}
                    <div className="w-64 flex-shrink-0">
                        <div className="border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                            <h2 className="text-xl font-['Press_Start_2P'] mb-4">Details</h2>
                            <div className="space-y-2">
                                <p className="font-['Press_Start_2P'] text-sm">
                                    <span className="text-gray-600">Upvotes:</span> {selectedMap.upvotes}
                                </p>
                                {selectedMap.difficulties && (
                                    <p className="font-['Press_Start_2P'] text-sm">
                                        <span className="text-gray-600">Difficulties:</span> {selectedMap.difficulties.join(", ")}
                                    </p>
                                )}
                            </div>
                        </div>

                        <div className="mt-4 border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                            <h2 className="text-xl font-['Press_Start_2P'] mb-4">Download</h2>
                            <div className="space-y-4">
                                <a
                                    href={`https://beatmap-browser.s3.amazonaws.com/${selectedMap.id}.zip`}
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
            </motion.div>
        </div>
    );
}