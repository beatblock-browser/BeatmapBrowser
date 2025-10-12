import React, { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import NotFound from './pages/not-found';
import Topbar from './components/Topbar';
import AccountPage from './pages/account-page';
import UploadPage from './pages/upload-page';
import { initAuthSession } from '@/lib/auth_session';

function AppRoutes() {
    return (
        <>
            <Topbar />
            <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/account" element={<AccountPage />} />
                <Route path="/upload" element={<UploadPage />} />
                <Route path="/song/:id" element={<SongPage />} />
                <Route path="/404" element={<NotFound />} />
                <Route path="*" element={<NotFound />} />
            </Routes>
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