import { defineConfig } from "vite";
import monacoEditorPluginImport from "vite-plugin-monaco-editor";

const monacoEditorPlugin =
  (monacoEditorPluginImport as unknown as { default?: unknown }).default ??
  monacoEditorPluginImport;

export default defineConfig({
  root: "src",
  clearScreen: false,
  plugins: [
    (monacoEditorPlugin as any)({
      languageWorkers: ["editorWorkerService", "typescript", "json", "html"],
    }),
  ],
  server: {
    port: 15000,
    strictPort: true
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    sourcemap: true,
    outDir: "../dist",
    emptyOutDir: true
  }
});
