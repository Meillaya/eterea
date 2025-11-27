import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit()
  ],
  
  // Tauri expects a fixed port
  server: {
    port: 1420,
    strictPort: true,
    host: true
  },
  
  build: {
    target: 'esnext'
  },
  
  // Optimize for Tauri
  clearScreen: false
});

