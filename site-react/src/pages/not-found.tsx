import React from "react";
import { useNavigate } from "react-router-dom";

export default function NotFound() {
    const navigate = useNavigate();

    const handleGoHome = () => {
        navigate('/');
    };

    return (
        <div className="min-h-screen bg-white flex items-center justify-center">
            <div className="text-center">
                <h1 className="text-6xl font-['Press_Start_2P'] mb-4">404</h1>
                <h2 className="text-2xl font-['Press_Start_2P'] mb-8">Page Not Found</h2>
                <p className="text-lg font-['Press_Start_2P'] mb-8 text-gray-600">
                    The page you're looking for doesn't exist.
                </p>
                <button
                    onClick={handleGoHome}
                    className="px-6 py-3 bg-black text-white font-['Press_Start_2P'] text-sm hover:bg-gray-800 transition-colors border border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:translate-x-1 hover:translate-y-1 hover:shadow-none"
                >
                    Back to Home
                </button>
            </div>
        </div>
    );
}
