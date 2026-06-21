import { resolve } from "node:path";
import { defineConfig } from "vite";
import terajsPlugin from "@terajs/app/vite";

export default defineConfig({
  plugins: [terajsPlugin()],
  server: {
    host: "127.0.0.1",
    port: 4174,
    proxy: {
      "/truth": "http://127.0.0.1:3030",
      "/repositories": "http://127.0.0.1:3030",
      "/memory": "http://127.0.0.1:3030",
      "/context-runs": "http://127.0.0.1:3030",
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 4174,
  },
  build: {
    manifest: true,
    rollupOptions: {
      input: {
        app: resolve(__dirname, "index.html"),
      },
    },
  },
});
