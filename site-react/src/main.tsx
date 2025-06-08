import { createRoot } from "react-dom/client";
import App from "./App";
import { SearchCacheProvider } from "./context/SearchCache";
import "./index.css";

createRoot(document.getElementById("root")!).render(
    <SearchCacheProvider>
        <App />
    </SearchCacheProvider>
);
