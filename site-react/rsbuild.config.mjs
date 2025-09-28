import {defineConfig} from '@rsbuild/core';
import {pluginReact} from '@rsbuild/plugin-react';

export default defineConfig({
    source: {
        define: {
            'process.env.API_BASE': JSON.stringify(''),
        },
    },
    plugins: [pluginReact()],
    resolve: {
        alias: {
            "@": "./src",
            "@shared": "shared"
        }
    },
    server: {
        // Proxy API requests during dev to Wrangler's local worker
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:8787',
                changeOrigin: true,
            },
        },
    },
});