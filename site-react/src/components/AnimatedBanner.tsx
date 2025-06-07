import React, { useEffect, useRef } from 'react';
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
    const navigate = useNavigate();

    useEffect(() => {
        if (!bannerRef.current) return;

        const banner = bannerRef.current;
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

        // Force reflow
        banner.offsetHeight;

        // Start transition
        banner.style.transition = 'all 0.5s cubic-bezier(0.4, 0, 0.2, 1)';
        requestAnimationFrame(() => {
            banner.style.top = '0';
            banner.style.left = '0';
            banner.style.width = '100%';
            banner.style.height = '256px';
        });

        // Navigate after animation completes
        const timeout = setTimeout(() => {
            navigate(`/song/${map.id}`);
            onAnimationComplete();
        }, 500);

        return () => {
            clearTimeout(timeout);
        };
    }, [cardPosition, onAnimationComplete, map.id, navigate]);

    return createPortal(
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
        </div>,
        document.body
    );
} 