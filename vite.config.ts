import { defineConfig } from "vite";

export default defineConfig({
  root: "frontend",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    sourcemap: true,
    // The wallet-only Anchor client is lazy-loaded as a separate chunk. Its
    // generated IDL and Solana codecs are intentionally absent from demo boot.
    chunkSizeWarningLimit: 550,
  },
  server: {
    port: 4173,
  },
});
