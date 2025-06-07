import { StrictMode } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import HomePage from './pages/home-page';
import SongPage from './pages/song-page';
import { AnimatedLayout } from './components/AnimatedLayout';

export default function App() {
    console.log('App rendering');
    return (
        <StrictMode>
            <Router>
                <Routes>
                    <Route element={<AnimatedLayout />}>
                        <Route path="/" element={<HomePage />} />
                        <Route path="/song/:id" element={<SongPage />} />
                    </Route>
                </Routes>
            </Router>
        </StrictMode>
    );
}