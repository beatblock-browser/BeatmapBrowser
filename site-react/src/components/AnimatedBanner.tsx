import React, { useEffect, useRef, useState } from 'react';
import { BeatMap } from '@/schema';
import { useNavigate } from 'react-router-dom';
import { createPortal } from 'react-dom';
// @ts-ignore IDE doesn't recognize image imports.
import default_image from '../public/beatblocks.jpg';

interface AnimatedBannerProps {
    map: BeatMap;
    cardPosition: DOMRect;
    onAnimationComplete: () => void;
}

export function AnimatedBanner({ map, cardPosition, onAnimationComplete }: AnimatedBannerProps) {
    const bannerRef = useRef<HTMLDivElement>(null);
    const panelRef = useRef<HTMLDivElement>(null);
    const navigate = useNavigate();
    const [isNavigating, setIsNavigating] = useState(false);

    useEffect(() => {
        if (!bannerRef.current || !panelRef.current) return;

        const banner = bannerRef.current;
        const panel = panelRef.current;
        const startX = cardPosition.left;
        const startY = cardPosition.top;
        const startWidth = cardPosition.width;
        const startHeight = cardPosition.height;

        // Set initial position
        banner.style.position = 'fixed';
        banner.style.top = `${startY}px`;
        banner.style.left = `${startX}px`;
        banner.style.width = `${startWidth}px`;
        banner.style.height = `${startHeight}px`;
        banner.style.zIndex = '9999';
        banner.style.transition = 'none';

        panel.style.position = 'fixed';
        panel.style.top = `${startY + startHeight}px`;
        panel.style.left = `${startX}px`;
        panel.style.width = `${startWidth}px`;
        panel.style.height = '0';
        panel.style.zIndex = '9998';
        panel.style.transition = 'none';

        // Force reflow
        banner.offsetHeight;
        panel.offsetHeight;

        // Start transition
        banner.style.transition = 'all 0.5s cubic-bezier(0.4, 0, 0.2, 1)';
        panel.style.transition = 'all 0.5s cubic-bezier(0.4, 0, 0.2, 1)';
        
        requestAnimationFrame(() => {
            banner.style.top = '0';
            banner.style.left = '0';
            banner.style.width = '100%';
            banner.style.height = '256px';

            panel.style.top = '256px';
            panel.style.left = '0';
            panel.style.width = '100%';
            panel.style.height = '100vh';
        });

        // Navigate after animation completes
        const timeout = setTimeout(() => {
            setIsNavigating(true);
            navigate(`/song/${map.id}`, {
                state: {
                    initialImage: map.image ? `https://beatmap-browser.s3.amazonaws.com/${map.id}.png` : default_image,
                    initialSong: map.song,
                    initialArtist: map.artist,
                    initialCharter: map.charter
                }
            });
        }, 500);

        return () => {
            clearTimeout(timeout);
        };
    }, [cardPosition, map, navigate]);

    // Only call onAnimationComplete when we're done navigating
    useEffect(() => {
        if (isNavigating) {
            const timeout = setTimeout(() => {
                onAnimationComplete();
            }, 100);
            return () => clearTimeout(timeout);
        }
    }, [isNavigating, onAnimationComplete]);

    return createPortal(
        <>
            <div
                ref={bannerRef}
                className="fixed overflow-hidden"
                style={{
                    backgroundColor: 'white',
                    border: '2px solid black',
                    willChange: 'transform'
                }}
            >
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
            <div
                ref={panelRef}
                className="fixed bg-gray-100"
                style={{
                    willChange: 'transform'
                }}
            />
        </>,
        document.body
    );
} 