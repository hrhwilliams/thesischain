import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import wasm from 'vite-plugin-wasm';

// https://vite.dev/config/
export default defineConfig({
    plugins: [vue(), wasm()],
    server: {
        port: 8080,
        proxy: {
            '/api': {
                target: 'http://localhost:8081',
                changeOrigin: true
            },
            '/ws': {
                target: 'ws://localhost:8081',
                ws: true
            }
        }
    }
})
