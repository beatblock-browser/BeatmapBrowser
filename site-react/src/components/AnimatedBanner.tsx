import React, { useEffect, useRef } from 'react';
import { BeatMap } from '@/schema';
import { createPortal } from 'react-dom';
// @ts-ignore IDE doesn't recognize image imports.
import default_image from '../public/beatblocks.jpg';

interface AnimatedBannerProps {
    map: BeatMap;
    cardPosition: DOMRect;
    onAnimationComplete: () => void;
    isReversing?: boolean;
}

export function AnimatedBanner({ map, cardPosition, onAnimationComplete, isReversing = false }: AnimatedBannerProps) {
    const bannerRef = useRef<HTMLDivElement>(null);
    const panelRef = useRef<HTMLDivElement>(null);

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
        banner.style.top = '0';
        banner.style.left = '0';
        banner.style.width = '100%';
        banner.style.height = '256px';
        banner.style.zIndex = '9999';
        banner.style.transformOrigin = 'top left';
        banner.style.transition = 'none';
        banner.style.transform = isReversing 
            ? 'none'
            : `translate(${startX}px, ${startY}px) scale(${startWidth / window.innerWidth}, ${startHeight / 256})`;

        panel.style.position = 'fixed';
        panel.style.top = '256px';
        panel.style.left = '0';
        panel.style.width = '100%';
        panel.style.height = '100vh';
        panel.style.zIndex = '9998';
        panel.style.transition = 'none';
        panel.style.transform = isReversing
            ? 'none'
            : `translate(${startX}px, ${startY + startHeight}px) scale(${startWidth / window.innerWidth}, 0)`;

        // Force reflow
        banner.offsetHeight;
        panel.offsetHeight;

        // Start transition
        banner.style.transition = 'transform 0.5s cubic-bezier(0.4, 0, 0.2, 1)';
        panel.style.transition = 'transform 0.5s cubic-bezier(0.4, 0, 0.2, 1)';
        
        requestAnimationFrame(() => {
            if (isReversing) {
                banner.style.transform = `translate(${startX}px, ${startY}px) scale(${startWidth / window.innerWidth}, ${startHeight / 256})`;
                panel.style.transform = `translate(${startX}px, ${startY + startHeight}px) scale(${startWidth / window.innerWidth}, 0)`;
            } else {
                banner.style.transform = 'none';
                panel.style.transform = 'none';
            }
        });

        // Call onAnimationComplete after animation
        const timeout = setTimeout(() => {
            onAnimationComplete();
        }, 500);

        return () => {
            clearTimeout(timeout);
        };
    }, [cardPosition, map, onAnimationComplete, isReversing]);

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