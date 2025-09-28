import React from 'react';
import { BrowserRouter as Router, Routes, Route, useLocation } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import NotFound from './pages/not-found';
import { SearchCacheProvider } from "@/context/SearchCache.tsx";

function AppRoutes() {
    const location = useLocation();
    const state = location.state as { backgroundLocation?: unknown } | undefined;

    return (
        <>
            {/* Render the underlying routes at the background location to keep the previous page mounted */}
            <Routes location={state?.backgroundLocation || location}>
                <Route path="/" element={<HomePage />} />
                <Route path="/song/:id" element={<SongPage />} />
                <Route path="*" element={<NotFound />} />
            </Routes>

            {/* If we have a background location, render the song route again as an overlay/modal */}
            {state?.backgroundLocation && (
                <Routes>
                    <Route path="/song/:id" element={<SongPage />} />
                </Routes>
            )}
        </>
    );
}

export default function App() {
    return (
        <SearchCacheProvider>
            <Router>
                <AppRoutes />
            </Router>
        </SearchCacheProvider>
    );
}