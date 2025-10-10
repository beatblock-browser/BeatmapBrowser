import React, { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, useLocation } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import NotFound from './pages/not-found';
import Topbar from './components/Topbar';
import AccountPage from './pages/account-page';
import UploadPage from './pages/upload-page';
import { initAuthSession } from '@/lib/auth_session';

function AppRoutes() {
    const location = useLocation();
    const state = location.state as { backgroundLocation?: unknown } | undefined;

    return (
        <>
            <Topbar />
            {/* Render the underlying routes at the background location to keep the previous page mounted */}
            <div className="pt-14">
                <Routes location={state?.backgroundLocation || location}>
                    <Route path="/" element={<HomePage />} />
                    <Route path="/account" element={<AccountPage />} />
                    <Route path="/upload" element={<UploadPage />} />
                    <Route path="/song/:id" element={<SongPage />} />
                    <Route path="/404" element={<NotFound />} />
                    <Route path="*" element={<NotFound />} />
                </Routes>
            </div>

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
    useEffect(() => {
        initAuthSession();
    }, []);
    return (
        <Router>
            <AppRoutes />
        </Router>
    );
}