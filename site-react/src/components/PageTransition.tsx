import React, { useState, useEffect } from 'react';
import { BeatMap } from '@/schema';
import HomePage from '@/pages/home-page';
import SongPage from '@/pages/song-page';
import { AnimatedBanner } from './AnimatedBanner';

export default function PageTransition() {
    const [currentMap, setCurrentMap] = useState<BeatMap | null>(null);
    const [cardPosition, setCardPosition] = useState<DOMRect | null>(null);
    const [isAnimating, setIsAnimating] = useState(false);
    const [isReversing, setIsReversing] = useState(false);
    const [pendingMap, setPendingMap] = useState<BeatMap | null>(null);
    const [animatingMap, setAnimatingMap] = useState<BeatMap | null>(null);

    // Handle card click from home page
    const handleCardClick = (map: BeatMap, cardElement: HTMLElement) => {
        const rect = cardElement.getBoundingClientRect();
        const position = new DOMRect(
            rect.left,
            rect.top,
            rect.width,
            rect.height
        );
        
        setPendingMap(map);
        setCardPosition(position);
        setIsAnimating(true);
        setIsReversing(false);
    };

    // Handle back button from song page
    const handleBack = () => {
        // Store the current map for animation
        setAnimatingMap(currentMap);
        // Change page state immediately
        setCurrentMap(null);
        // Start reverse animation
        setIsAnimating(true);
        setIsReversing(true);
    };

    // Handle animation completion
    const handleAnimationComplete = () => {
        setIsAnimating(false);
        if (isReversing) {
            // Clear animation state after reverse animation completes
            setAnimatingMap(null);
            setCardPosition(null);
        } else if (pendingMap) {
            // Set the map after opening animation completes
            setCurrentMap(pendingMap);
            setPendingMap(null);
        }
    };

    return (
        <>
            {currentMap ? (
                <SongPage
                    map={currentMap}
                    cardPosition={cardPosition!}
                    onBack={handleBack}
                />
            ) : (
                <HomePage onCardClick={handleCardClick} />
            )}
            {isAnimating && cardPosition && (animatingMap || pendingMap) && (
                <AnimatedBanner
                    map={animatingMap || pendingMap!}
                    cardPosition={cardPosition}
                    onAnimationComplete={handleAnimationComplete}
                    isReversing={isReversing}
                />
            )}
        </>
    );
} 