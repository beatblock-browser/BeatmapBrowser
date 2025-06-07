import React, { createContext, useContext, useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface AnimationContextType {
    cardPosition: DOMRect | null;
    setCardPosition: (rect: DOMRect | null) => void;
    isAnimating: boolean;
    startAnimation: (rect: DOMRect, targetPath: string) => void;
}

const AnimationContext = createContext<AnimationContextType | undefined>(undefined);

export function AnimationProvider({ children }: { children: React.ReactNode }) {
    const [cardPosition, setCardPosition] = useState<DOMRect | null>(null);
    const [isAnimating, setIsAnimating] = useState(false);
    const navigate = useNavigate();

    const startAnimation = (rect: DOMRect, targetPath: string) => {
        setCardPosition(rect);
        setIsAnimating(true);
        
        // Navigate after animation completes
        setTimeout(() => {
            navigate(targetPath);
        }, 500);
    };

    return (
        <AnimationContext.Provider value={{ 
            cardPosition, 
            setCardPosition, 
            isAnimating,
            startAnimation 
        }}>
            {children}
        </AnimationContext.Provider>
    );
}

export function useAnimation() {
    const context = useContext(AnimationContext);
    if (context === undefined) {
        throw new Error('useAnimation must be used within an AnimationProvider');
    }
    return context;
} 