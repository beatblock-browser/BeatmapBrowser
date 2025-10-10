import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { SearchCacheProvider } from './context/SearchCache';
import './index.css';

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <SearchCacheProvider>
      <App />
    </SearchCacheProvider>
  </React.StrictMode>,
);
