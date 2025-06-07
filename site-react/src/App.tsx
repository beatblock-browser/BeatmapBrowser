import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import { SearchCacheProvider } from './context/SearchCache';

export default function App() {
    return (
        <SearchCacheProvider>
            <Router>
                <Routes>
                    <Route path="/" element={<HomePage />} />
                    <Route path="/song/:id" element={<SongPage />} />
                </Routes>
            </Router>
        </SearchCacheProvider>
    );
}