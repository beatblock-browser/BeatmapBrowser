import React, { useEffect, useState } from "react";
import { SearchRequest, SearchResult } from "@/schema/search";
import { BeatMap } from "@/schema";

export default function HomePage() {
    const [results, setResults] = useState<BeatMap[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchResults = async () => {
            setLoading(true);
            setError(null);
            try {
                const res = await fetch("/api/search", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ query: "" } as SearchRequest)
                });
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                const data: SearchResult = await res.json();
                setResults(data.results || []);
            } catch (e: any) {
                setError(e.message || "Unknown error");
            } finally {
                setLoading(false);
            }
        };
        fetchResults();
    }, []);

    return (
        <div style={{ maxWidth: 800, margin: "2rem auto", padding: "1rem" }}>
            <h1>Search</h1>
            {loading && <div>Loading...</div>}
            {error && <div style={{ color: "red" }}>Error: {error}</div>}
            {!loading && !error && (
                <ul>
                    {results.length === 0 && <li>No results found.</li>}
                    {results.map((map) => (
                        <li key={map.id} style={{ marginBottom: "1rem", borderBottom: "1px solid #eee", paddingBottom: "0.5rem" }}>
                            <strong>{map.song}</strong> by {map.artist}<br />
                            Charter: {map.charter}<br />
                            Upvotes: {map.upvotes}
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
