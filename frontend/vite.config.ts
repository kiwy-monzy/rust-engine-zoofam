import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Dev server proxies /api to the gateway so the SPA talks to :8080 without
// CORS. In production the gateway itself serves this dist/ folder, so the
// same relative /api/v1 paths work in both modes.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8080",
        changeOrigin: false,
      },
    },
  },
  build: {
    outDir: "dist",
    sourcemap: false,
    commonjsOptions: { transformMixedEsModules: true },
  },
  optimizeDeps: {
    include: ["leaflet", "react-leaflet"],
  },
  resolve: {
    dedupe: ["leaflet", "react-leaflet"],
  },
});
