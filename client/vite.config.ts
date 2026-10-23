import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  plugins: [react(), wasmPack()],
  worker: { format: 'es' },
  server: {
    port: 3000,
    host: '0.0.0.0',
  },
});