import React from 'react';
import HomePage from './pages/home-page';
import {SearchCacheProvider} from "@/context/SearchCache.tsx";

export default function App() {
    return <SearchCacheProvider>
        <HomePage/>
    </SearchCacheProvider>;
}