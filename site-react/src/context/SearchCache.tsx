import React, { createContext, useContext, useState } from 'react';
import { BeatMap } from '@/schema';

interface SearchCacheContextType {
    results: BeatMap[];
    setResults: (results: BeatMap[]) => void;
    isLoading: boolean;
    setIsLoading: (isLoading: boolean) => void;
}

const SearchCacheContext = createContext<SearchCacheContextType | undefined>(undefined);

export function SearchCacheProvider({ children }: { children: React.ReactNode }) {
    const [results, setResults] = useState<BeatMap[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    return (
        <SearchCacheContext.Provider value={{ results, setResults, isLoading, setIsLoading }}>
            {children}
        </SearchCacheContext.Provider>
    );
}

export function useSearchCache() {
    const context = useContext(SearchCacheContext);
    if (context === undefined) {
        throw new Error('useSearchCache must be used within a SearchCacheProvider');
    }
    return context;
} 