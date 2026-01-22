import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import wasm from 'vite-plugin-wasm';

// https://vite.dev/config/
export default defineConfig({
    plugins: [vue(), wasm()],
    server: {
        host: '0.0.0.0',
        port: 8081,
        allowedHosts: ['chat.fiatlux.dev'],
        strictPort: true,
        hmr: {
            host: 'chat.fiatlux.dev',
            clientPort: 443,
            protocol: 'wss'
        }
    }
})
