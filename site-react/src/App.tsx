import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import NotFound from './pages/not-found';
import {SearchCacheProvider} from "@/context/SearchCache.tsx";

export default function App() {
    return (
        <SearchCacheProvider>
            <Router>
                <Routes>
                    <Route path="/" element={<HomePage />} />
                    <Route path="/song/:id" element={<SongPage />} />
                    <Route path="*" element={<NotFound />} />
                </Routes>
            </Router>
        </SearchCacheProvider>
    );
}