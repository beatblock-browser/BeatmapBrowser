import React, { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { AnimatedBanner } from './AnimatedBanner';
import { BeatMap } from '@/schema';

interface AnimationState {
    map: BeatMap;
    position: DOMRect;
}

export function AnimatedLayout() {
    const [animationState, setAnimationState] = useState<AnimationState | null>(null);

    const startAnimation = (map: BeatMap, position: DOMRect) => {
        setAnimationState({ map, position });
    };

    const handleAnimationComplete = () => {
        setAnimationState(null);
    };

    return (
        <>
            <Outlet context={{ startAnimation }} />
            {animationState && (
                <AnimatedBanner
                    map={animationState.map}
                    cardPosition={animationState.position}
                    onAnimationComplete={handleAnimationComplete}
                />
            )}
        </>
    );
} 