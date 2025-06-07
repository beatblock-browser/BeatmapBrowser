import React, { useEffect, useState } from "react";
import { useParams, useLocation, useNavigate } from "react-router-dom";
import { BeatMap } from "@/schema";
import { AnimatedBanner } from "@/components/AnimatedBanner";
import ReactDOM from "react-dom/client";
// @ts-ignore IDE doesn't recognize image imports.
import default_image from './../public/beatblocks.jpg';

interface LocationState {
    initialImage: string;
    initialSong: string;
    initialArtist: string;
    initialCharter: string;
    cardPosition?: DOMRect;
}

export default function SongPage() {
    const { id } = useParams();
    const location = useLocation();
    const navigate = useNavigate();
    const state = location.state as LocationState;
    const [map, setMap] = useState<BeatMap | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [contentVisible, setContentVisible] = useState(false);
    const [isAnimating, setIsAnimating] = useState(false);

    useEffect(() => {
        const fetchMap = async () => {
            setLoading(true);
            setError(null);
            try {
                const res = await fetch(`/api/map/${id}`);
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                const data = await res.json();
                setMap(data);
                // Delay showing content until after the banner animation
                setTimeout(() => {
                    setContentVisible(true);
                }, 600);
            } catch (e: any) {
                setError(e.message || "Unknown error");
            } finally {
                setLoading(false);
            }
        };
        fetchMap();
    }, [id]);

    const handleBack = () => {
        if (!map || isAnimating) return;
        
        setIsAnimating(true);
        setContentVisible(false);

        // Create a DOMRect for the animation
        let cardPosition: DOMRect;
        if (!state?.cardPosition) {
            cardPosition = new DOMRect(0, 0, 300, 200);
        } else {
            cardPosition = new DOMRect(
                state.cardPosition.left,
                state.cardPosition.top,
                state.cardPosition.width,
                state.cardPosition.height
            );
        }

        // Navigate immediately
        navigate('/', {
            state: {
                map,
                cardPosition
            }
        });

        // Use requestAnimationFrame to ensure navigation happens before animation
        requestAnimationFrame(() => {
            // Start reverse animation
            const banner = document.createElement('div');
            document.body.appendChild(banner);
            const root = ReactDOM.createRoot(banner);
            
            const cleanup = () => {
                root.unmount();
                document.body.removeChild(banner);
                setIsAnimating(false);
                
                // Scroll to the saved position
                if (state?.cardPosition?.scrollY !== undefined) {
                    window.scrollTo({
                        top: state.cardPosition.scrollY,
                        behavior: 'instant'
                    });
                }
            };

            // Render the AnimatedBanner with reverse animation
            root.render(
                <AnimatedBanner
                    map={map}
                    cardPosition={cardPosition}
                    onAnimationComplete={cleanup}
                    isReversing={true}
                />
            );
        });
    };

    if (error) return <div className="text-red-500 text-center font-['Press_Start_2P']">Error: {error}</div>;

    return (
        <div className="min-h-screen bg-gray-100">
            {/* Static Banner */}
            <div className="relative h-64 w-full">
                <button
                    onClick={handleBack}
                    className="absolute top-4 left-4 z-10 bg-white/90 hover:bg-white text-black p-2 rounded-full shadow-lg transition-colors"
                    disabled={isAnimating}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                </button>
                <img 
                    src={map?.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : state?.initialImage || default_image}
                    alt={map?.song || state?.initialSong || "Loading..."}
                    className="absolute inset-0 w-full h-full object-cover"
                />
                <div className="absolute inset-0 bg-black/60" />
                <div className="absolute inset-0 flex flex-col justify-end p-8 text-white">
                    <h1 className="text-4xl font-['Press_Start_2P'] mb-2">{map?.song || state?.initialSong || "Loading..."}</h1>
                    <p className="text-xl font-['Press_Start_2P'] mb-1">by {map?.artist || state?.initialArtist || "..."}</p>
                    <p className="text-xl font-['Press_Start_2P']">Charter: {map?.charter || state?.initialCharter || "..."}</p>
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
                    ) : map ? (
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
                    ) : (
                        <div className="text-center font-['Press_Start_2P']">Map not found</div>
                    )}
                </div>
            </div>
        </div>
    );
} 