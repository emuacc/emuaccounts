import { defineConfig } from "vite";

export default defineConfig({
  server: {
    port: 1420,
    strictPort: true,
    // Listen on all interfaces so a phone on the same LAN/Tailnet can load the
    // dev server with live reload. Data calls are proxied to emuacc-web.
    host: true,
    watch: {
      // Cargo's target tree is huge and changes heavily during Rust builds.
      // Watching it can retain substantial filesystem state in the dev server.
      ignored: ["**/target/**"],
    },
    proxy: {
      "/api": "http://127.0.0.1:8787",
    },
  },
  clearScreen: false,
});
