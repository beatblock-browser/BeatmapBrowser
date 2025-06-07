import React from 'react';
import PageTransition from './components/PageTransition';
import { SearchCacheProvider } from './context/SearchCache';

export default function App() {
    return (
        <SearchCacheProvider>
            <PageTransition />
        </SearchCacheProvider>
    );
}