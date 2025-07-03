import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useStore } from "@/lib/store";
import default_image from './../public/beatblocks.jpg';

export default function SongPage() {
    const { selectedMap, setSelectedMap, upvoteMap, unvoteMap } = useStore();

    if (!selectedMap) return null;

    const handleBack = () => {
        setSelectedMap(null);
    };

    const handleUpvote = async () => {
        await upvoteMap(selectedMap.id);
    };

    return (
        <motion.div
            className="fixed inset-0 z-10"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
        >
            {/* Background Panel */}
            <motion.div
                className="absolute inset-0 bg-white"
                initial={{ y: "-100%" }}
                animate={{ y: 0 }}
                exit={{ y: "-100%" }}
                transition={{
                    duration: 0.4,
                    ease: [0.25, 0.1, 0.25, 1]
                }}
            />

            {/* Banner - Fixed at top */}
            <motion.div
                layoutId={`card-${selectedMap.id}`}
                className="absolute top-0 left-0 right-0 overflow-hidden"
                style={{
                    height: 'calc(100vw * 9 / 32)',
                    maxHeight: '400px'
                }}
                transition={{
                    layout: {
                        duration: 0.4,
                        ease: [0.25, 0.1, 0.25, 1]
                    }
                }}
            >
                <motion.button
                    initial={{ opacity: 0, scale: 0.8 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.8 }}
                    transition={{ duration: 0.2, delay: 0.2 }}
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    className="absolute top-4 left-4 z-30 text-white p-2 bg-black/20 rounded-full backdrop-blur-sm"
                    onClick={handleBack}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                </motion.button>

                <motion.div
                    layoutId={`image-${selectedMap.id}`}
                    className="absolute inset-0"
                >
                    <img
                        src={selectedMap.image ? `https://beatmap-browser.s3.amazonaws.com/${selectedMap.id}.png` : default_image}
                        alt={selectedMap.song}
                        className="w-full h-full object-cover"
                    />
                </motion.div>

                <motion.div
                    layoutId={`overlay-${selectedMap.id}`}
                    className="absolute inset-0 bg-black/60"
                />

                <motion.div
                    layoutId={`content-${selectedMap.id}`}
                    className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-8"
                >
                    <div className="flex-1 min-w-0">
                        <h1 className="text-6xl mb-2 line-clamp-1">{selectedMap.song}</h1>
                        <p className="text-2xl mb-1">by {selectedMap.artist}</p>
                        <p className="text-2xl">Charter: {selectedMap.charter}</p>
                    </div>
                </motion.div>
            </motion.div>

            {/* Main Content - Scrollable below banner */}
            <motion.div
                initial={{ opacity: 0, y: 50 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 50 }}
                transition={{
                    delay: 0.3,
                    duration: 0.3,
                    ease: "easeOut"
                }}
                className="absolute top-0 left-0 right-0 bottom-0 overflow-y-auto pt-[400px]"
            >
                <div className="max-w-6xl mx-auto p-4">
                    <div className="flex gap-6">
                        {/* Left Sidebar */}
                        <div className="w-64 flex-shrink-0">
                            <div className="border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 bg-white">
                                <h2 className="text-xl font-['Press_Start_2P'] mb-4">Details</h2>
                                <div className="space-y-2">
                                    <button 
                                        onClick={handleUpvote}
                                        className="w-full text-left font-['Press_Start_2P'] text-sm hover:bg-gray-100 p-2 rounded transition-colors"
                                    >
                                        <span className="text-gray-600">Upvotes:</span> {selectedMap.upvotes}
                                    </button>
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
                </div>
            </motion.div>
        </motion.div>
    );
}