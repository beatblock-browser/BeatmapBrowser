import { StrictMode } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';

export default function App() {
    console.log('App rendering');
    return (
        <StrictMode>
            <Router>
                <Routes>
                    <Route path="/" element={<HomePage />} />
                    <Route path="/song/:id" element={<SongPage />} />
                </Routes>
            </Router>
        </StrictMode>
    );
}